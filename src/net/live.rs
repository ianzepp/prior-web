use serde::{Deserialize, Serialize};

use crate::net::factory::FactoryEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LiveClientMessage {
    JoinRoom { room: String },
    SendRoomMessage { room: String, content: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LiveServerMessage {
    Connected {
        actor: String,
        session_id: String,
    },
    RoomEvent {
        room: Option<String>,
        topic: String,
        actor: Option<String>,
        content: String,
    },
    RoomTurnDone {
        room: String,
    },
    FactoryEvent {
        event: FactoryEvent,
    },
    Error {
        message: String,
    },
}

#[cfg(feature = "ssr")]
mod ssr {
    use std::collections::HashMap;
    use std::sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    };

    use axum::{
        extract::{
            State,
            ws::{Message, WebSocket, WebSocketUpgrade},
        },
        http::HeaderMap,
        response::{IntoResponse, Response},
    };
    use futures_util::{SinkExt, StreamExt};
    use prost::Message as ProstMessage;
    use prost_types::{Struct, Value, value::Kind};
    use tokio::sync::{Mutex, mpsc, oneshot};
    use tokio_tungstenite::{connect_async, tungstenite::Message as GateWsMessage};

    use crate::{
        auth::user::current_session_from_headers,
        net::{
            factory::FactoryEvent,
            prior_gate_proto::{
                ClientEnvelope, ClientHello, GateEvent, GateRequest, GateResponse, ResponseItem,
                ResponseOp, ServerEnvelope, ServerHello, client_envelope, server_envelope,
            },
        },
        runtime::{AppState, PriorGateConfig},
    };

    use super::{LiveClientMessage, LiveServerMessage};

    const MAX_SAFE_INTEGER_F64: f64 = 9_007_199_254_740_991.0;

    pub async fn ws_handler(
        ws: WebSocketUpgrade,
        State(state): State<AppState>,
        headers: HeaderMap,
    ) -> Response {
        let Some(session) = current_session_from_headers(&headers, &state) else {
            return axum::http::StatusCode::UNAUTHORIZED.into_response();
        };

        ws.on_upgrade(move |socket| handle_socket(socket, state.gate, session.user.sub))
    }

    async fn handle_socket(socket: WebSocket, config: PriorGateConfig, actor: String) {
        let (mut browser_tx, mut browser_rx) = socket.split();
        let (server_tx, mut server_rx) = mpsc::channel::<LiveServerMessage>(64);

        let browser_writer = tokio::spawn(async move {
            while let Some(message) = server_rx.recv().await {
                let Ok(json) = serde_json::to_string(&message) else {
                    continue;
                };
                if browser_tx.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
        });

        let gate = match LiveGateClient::connect(config, actor.clone(), server_tx.clone()).await {
            Ok(gate) => gate,
            Err(error) => {
                let _ = server_tx
                    .send(LiveServerMessage::Error { message: error })
                    .await;
                browser_writer.abort();
                return;
            }
        };

        let _ = server_tx
            .send(LiveServerMessage::Connected {
                actor,
                session_id: gate.session_id.clone(),
            })
            .await;

        if let Err(error) = gate.subscribe_factory_events(None).await {
            let _ = server_tx
                .send(LiveServerMessage::Error { message: error })
                .await;
        }

        while let Some(message) = browser_rx.next().await {
            let Ok(message) = message else {
                break;
            };

            match message {
                Message::Text(text) => {
                    let parsed = serde_json::from_str::<LiveClientMessage>(&text);
                    match parsed {
                        Ok(LiveClientMessage::JoinRoom { room }) => {
                            if let Err(error) = gate.join_room(&room).await {
                                let _ = server_tx
                                    .send(LiveServerMessage::Error { message: error })
                                    .await;
                            }
                        }
                        Ok(LiveClientMessage::SendRoomMessage { room, content }) => {
                            let result = gate.send_room_message(&room, &content).await;
                            let message = match result {
                                Ok(()) => LiveServerMessage::RoomTurnDone { room },
                                Err(error) => LiveServerMessage::Error { message: error },
                            };
                            let _ = server_tx.send(message).await;
                        }
                        Err(error) => {
                            let _ = server_tx
                                .send(LiveServerMessage::Error {
                                    message: format!("invalid live message: {error}"),
                                })
                                .await;
                        }
                    }
                }
                Message::Close(_) => break,
                Message::Binary(_) | Message::Ping(_) | Message::Pong(_) => {}
            }
        }

        if let Err(error) = gate.disconnect().await {
            let _ = server_tx
                .send(LiveServerMessage::Error { message: error })
                .await;
        }

        gate.shutdown().await;
        drop(server_tx);
        let _ = browser_writer.await;
    }

    enum PendingRequest {
        Collect {
            syscall: String,
            responses: Vec<GateResponse>,
            tx: oneshot::Sender<Result<Vec<GateResponse>, String>>,
        },
        FactoryStream,
    }

    struct LiveGateClient {
        outbound: mpsc::Sender<GateWsMessage>,
        pending: Arc<Mutex<HashMap<String, PendingRequest>>>,
        next_request_id: AtomicU64,
        session_id: String,
    }

    impl LiveGateClient {
        async fn connect(
            config: PriorGateConfig,
            actor: String,
            server_tx: mpsc::Sender<LiveServerMessage>,
        ) -> Result<Self, String> {
            let (mut socket, _) = connect_async(&config.ws_url)
                .await
                .map_err(|error| format!("connect {}: {error}", config.ws_url))?;
            gate_hello(&mut socket, config.service_token, actor.clone()).await?;
            let (outbound, outbound_rx) = mpsc::channel::<GateWsMessage>(64);
            let pending = Arc::new(Mutex::new(HashMap::new()));
            let io_server_tx = server_tx.clone();
            let io_pending = Arc::clone(&pending);

            tokio::spawn(async move {
                run_gate_io(socket, outbound_rx, io_pending, io_server_tx).await;
            });

            let client = Self {
                outbound,
                pending,
                next_request_id: AtomicU64::new(1),
                session_id: String::new(),
            };

            let session_id = client.connect_session(&actor).await?;

            Ok(Self {
                session_id,
                ..client
            })
        }

        async fn connect_session(&self, actor: &str) -> Result<String, String> {
            let responses = self
                .request_collect(
                    "door:connect",
                    None,
                    struct_from_vec(vec![("from", string_value(actor))]),
                    None,
                )
                .await?;
            responses
                .iter()
                .filter_map(|response| response.item.as_ref())
                .find_map(|item| extract_string_field(item, "session"))
                .ok_or_else(|| "door:connect did not return a session id".into())
        }

        async fn subscribe_factory_events(&self, target_repo: Option<&str>) -> Result<(), String> {
            let request_id = self.allocate_request_id();
            self.pending
                .lock()
                .await
                .insert(request_id.clone(), PendingRequest::FactoryStream);
            self.send_request(
                request_id,
                "factory:event:subscribe",
                None,
                struct_from_vec(vec![(
                    "target_repo",
                    target_repo.map_or(
                        Value {
                            kind: Some(Kind::NullValue(0)),
                        },
                        string_value,
                    ),
                )]),
                None,
            )
            .await
        }

        async fn join_room(&self, room: &str) -> Result<(), String> {
            let room_join = self
                .request_collect("room:join", Some(room), Struct::default(), None)
                .await;
            if let Err(error) = room_join {
                if !error.contains("actor already joined") {
                    return Err(error);
                }
            }

            self.request_collect(
                "door:join",
                None,
                struct_from_vec(vec![
                    ("session", string_value(&self.session_id)),
                    ("room", string_value(room)),
                ]),
                None,
            )
            .await
            .map(|_| ())
        }

        async fn send_room_message(&self, room: &str, content: &str) -> Result<(), String> {
            self.request_collect(
                "door:message",
                None,
                struct_from_vec(vec![
                    ("session", string_value(&self.session_id)),
                    ("room", string_value(room)),
                    ("content", string_value(content)),
                ]),
                None,
            )
            .await
            .map(|_| ())
        }

        async fn disconnect(&self) -> Result<(), String> {
            self.request_collect(
                "door:disconnect",
                None,
                struct_from_vec(vec![("session", string_value(&self.session_id))]),
                None,
            )
            .await
            .map(|_| ())
        }

        async fn shutdown(&self) {
            let _ = self.outbound.send(GateWsMessage::Close(None)).await;
        }

        async fn request_collect(
            &self,
            syscall: &str,
            room: Option<&str>,
            data: Struct,
            secrets: Option<Struct>,
        ) -> Result<Vec<GateResponse>, String> {
            let request_id = self.allocate_request_id();
            let (tx, rx) = oneshot::channel();
            self.pending.lock().await.insert(
                request_id.clone(),
                PendingRequest::Collect {
                    syscall: syscall.to_string(),
                    responses: Vec::new(),
                    tx,
                },
            );
            if let Err(error) = self
                .send_request(request_id.clone(), syscall, room, data, secrets)
                .await
            {
                self.pending.lock().await.remove(&request_id);
                return Err(error);
            }
            rx.await
                .map_err(|error| format!("{syscall} failed waiting for response: {error}"))?
        }

        async fn send_request(
            &self,
            request_id: String,
            syscall: &str,
            room: Option<&str>,
            data: Struct,
            secrets: Option<Struct>,
        ) -> Result<(), String> {
            self.send_envelope(ClientEnvelope {
                body: Some(client_envelope::Body::Request(GateRequest {
                    request_id,
                    syscall: syscall.into(),
                    room: room.map(ToOwned::to_owned),
                    from: None,
                    timeout_ms: None,
                    data: Some(data),
                    trace: None,
                    secrets,
                })),
            })
            .await
        }

        async fn send_envelope(&self, envelope: ClientEnvelope) -> Result<(), String> {
            let mut bytes = Vec::new();
            envelope
                .encode(&mut bytes)
                .map_err(|error| format!("encode envelope: {error}"))?;
            self.outbound
                .send(GateWsMessage::Binary(bytes.into()))
                .await
                .map_err(|error| format!("queue gate envelope: {error}"))
        }

        fn allocate_request_id(&self) -> String {
            let id = self.next_request_id.fetch_add(1, Ordering::Relaxed);
            format!("prior-web-live-{id}")
        }
    }

    async fn run_gate_io(
        mut socket: tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        mut outbound_rx: mpsc::Receiver<GateWsMessage>,
        pending: Arc<Mutex<HashMap<String, PendingRequest>>>,
        server_tx: mpsc::Sender<LiveServerMessage>,
    ) {
        loop {
            tokio::select! {
                outbound = outbound_rx.recv() => {
                    let Some(message) = outbound else {
                        break;
                    };
                    if socket.send(message).await.is_err() {
                        break;
                    }
                }
                inbound = socket.next() => {
                    let Some(message) = inbound else {
                        break;
                    };
                    let Ok(message) = message else {
                        let _ = server_tx.send(LiveServerMessage::Error {
                            message: "gate websocket read failed".into(),
                        }).await;
                        break;
                    };
                    match message {
                        GateWsMessage::Ping(payload) => {
                            if socket.send(GateWsMessage::Pong(payload)).await.is_err() {
                                break;
                            }
                        }
                        other => {
                            if !handle_gate_message(other, &pending, &server_tx).await {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    async fn handle_gate_message(
        message: GateWsMessage,
        pending: &Arc<Mutex<HashMap<String, PendingRequest>>>,
        server_tx: &mpsc::Sender<LiveServerMessage>,
    ) -> bool {
        match message {
            GateWsMessage::Binary(bytes) => {
                let Ok(envelope) = ServerEnvelope::decode(bytes.as_ref()) else {
                    let _ = server_tx
                        .send(LiveServerMessage::Error {
                            message: "failed to decode gate envelope".into(),
                        })
                        .await;
                    return false;
                };
                handle_server_envelope(envelope, pending, server_tx).await;
                true
            }
            GateWsMessage::Close(_) => false,
            GateWsMessage::Text(_)
            | GateWsMessage::Pong(_)
            | GateWsMessage::Frame(_)
            | GateWsMessage::Ping(_) => true,
        }
    }

    async fn handle_server_envelope(
        envelope: ServerEnvelope,
        pending: &Arc<Mutex<HashMap<String, PendingRequest>>>,
        server_tx: &mpsc::Sender<LiveServerMessage>,
    ) {
        match envelope.body {
            Some(server_envelope::Body::Event(event)) => {
                if let Some(message) = gate_event_to_live_message(&event) {
                    let _ = server_tx.send(message).await;
                }
            }
            Some(server_envelope::Body::Response(response)) => {
                handle_gate_response(response, pending, server_tx).await;
            }
            Some(server_envelope::Body::Hello(_) | server_envelope::Body::Pong(_)) | None => {}
        }
    }

    async fn handle_gate_response(
        response: GateResponse,
        pending: &Arc<Mutex<HashMap<String, PendingRequest>>>,
        server_tx: &mpsc::Sender<LiveServerMessage>,
    ) {
        let mut guard = pending.lock().await;
        let Some(entry) = guard.get_mut(&response.request_id) else {
            return;
        };

        match entry {
            PendingRequest::Collect {
                syscall,
                responses,
                tx: _,
            } => {
                if response.op == ResponseOp::Error as i32 {
                    let message = response.error.as_ref().map_or_else(
                        || "request failed".to_string(),
                        |body| format!("{}: {}", body.code, body.message),
                    );
                    let request_id = response.request_id.clone();
                    let syscall = syscall.clone();
                    let tx = guard.remove(&request_id).and_then(|entry| match entry {
                        PendingRequest::Collect { tx, .. } => Some(tx),
                        PendingRequest::FactoryStream => None,
                    });
                    if let Some(tx) = tx {
                        let _ = tx.send(Err(format!("{syscall} failed: {message}")));
                    }
                    return;
                }

                responses.push(response.clone());
                let terminal = matches!(
                    ResponseOp::try_from(response.op),
                    Ok(ResponseOp::Done | ResponseOp::Cancel)
                );
                if terminal {
                    let request_id = response.request_id.clone();
                    let tx = guard.remove(&request_id).and_then(|entry| match entry {
                        PendingRequest::Collect { responses, tx, .. } => Some((responses, tx)),
                        PendingRequest::FactoryStream => None,
                    });
                    if let Some((responses, tx)) = tx {
                        let _ = tx.send(Ok(responses));
                    }
                }
            }
            PendingRequest::FactoryStream => {
                if response.op == ResponseOp::Item as i32 {
                    if let Some(item) = response.item.as_ref() {
                        match item_to_typed::<FactoryEvent>(item) {
                            Ok(event) => {
                                let _ = server_tx
                                    .send(LiveServerMessage::FactoryEvent { event })
                                    .await;
                            }
                            Err(error) => {
                                let _ = server_tx
                                    .send(LiveServerMessage::Error { message: error })
                                    .await;
                            }
                        }
                    }
                }

                let terminal = matches!(
                    ResponseOp::try_from(response.op),
                    Ok(ResponseOp::Done | ResponseOp::Cancel | ResponseOp::Error)
                );
                if terminal {
                    guard.remove(&response.request_id);
                }
            }
        }
    }

    fn gate_event_to_live_message(event: &GateEvent) -> Option<LiveServerMessage> {
        let record = gate_event_to_record(event)?;
        Some(LiveServerMessage::RoomEvent {
            room: record.room,
            topic: record.topic,
            actor: record.actor,
            content: record.content,
        })
    }

    #[derive(Debug)]
    struct GateEventRecord {
        room: Option<String>,
        topic: String,
        actor: Option<String>,
        content: String,
    }

    fn gate_event_to_record(event: &GateEvent) -> Option<GateEventRecord> {
        let data = event.data.as_ref()?;
        let topic = event.topic.clone();
        let actor = data.fields.get("from").and_then(value_as_string);
        let room = data.fields.get("room").and_then(value_as_string);
        let content = match topic.as_str() {
            "door:thought" | "door:chat" => data.fields.get("content").and_then(value_as_string)?,
            "door:tool" => format_tool_event(data),
            "door:tool_result" => format_tool_result_event(data),
            _ => return None,
        };
        Some(GateEventRecord {
            room,
            topic,
            actor,
            content,
        })
    }

    fn format_tool_event(data: &Struct) -> String {
        let syscall = data
            .fields
            .get("syscall")
            .and_then(value_as_string)
            .unwrap_or_else(|| "tool".to_string());
        let args = data
            .fields
            .get("args")
            .map(prost_value_to_json)
            .or_else(|| data.fields.get("input").map(prost_value_to_json))
            .map(pretty_json)
            .unwrap_or_default();
        if args.is_empty() {
            format!("Tool call: `{syscall}`")
        } else {
            format!("Tool call: `{syscall}`\n```json\n{args}\n```")
        }
    }

    fn format_tool_result_event(data: &Struct) -> String {
        let body = data
            .fields
            .get("content")
            .map(prost_value_to_json)
            .map(|value| match value {
                serde_json::Value::String(text) => text,
                other => pretty_json(other),
            })
            .unwrap_or_default();
        let label = if data
            .fields
            .get("is_error")
            .and_then(value_as_bool)
            .unwrap_or(false)
        {
            "Tool error"
        } else {
            "Tool result"
        };
        if body.is_empty() {
            label.to_string()
        } else {
            format!("{label}\n```\n{body}\n```")
        }
    }

    fn item_to_typed<T: serde::de::DeserializeOwned>(item: &ResponseItem) -> Result<T, String> {
        let value = item.data.as_ref().map_or_else(
            || serde_json::Value::Object(serde_json::Map::default()),
            struct_to_json_object,
        );
        let value = normalize_json_numbers(value);
        serde_json::from_value(value).map_err(|error| format!("decode response item: {error}"))
    }

    fn extract_string_field(item: &ResponseItem, field: &str) -> Option<String> {
        item.data
            .as_ref()
            .and_then(|data| data.fields.get(field))
            .and_then(value_as_string)
    }

    fn struct_from_vec(pairs: Vec<(&str, Value)>) -> Struct {
        Struct {
            fields: pairs
                .into_iter()
                .map(|(key, value)| (key.to_string(), value))
                .collect(),
        }
    }

    fn string_value(value: &str) -> Value {
        Value {
            kind: Some(Kind::StringValue(value.to_string())),
        }
    }

    fn value_as_string(value: &Value) -> Option<String> {
        match value.kind.as_ref() {
            Some(Kind::StringValue(text)) => Some(text.clone()),
            _ => None,
        }
    }

    fn value_as_bool(value: &Value) -> Option<bool> {
        match value.kind.as_ref() {
            Some(Kind::BoolValue(value)) => Some(*value),
            _ => None,
        }
    }

    fn struct_to_json_object(data: &Struct) -> serde_json::Value {
        serde_json::Value::Object(
            data.fields
                .iter()
                .map(|(key, value)| (key.clone(), prost_value_to_json(value)))
                .collect(),
        )
    }

    fn prost_value_to_json(value: &Value) -> serde_json::Value {
        match value.kind.as_ref() {
            Some(Kind::NullValue(_)) | None => serde_json::Value::Null,
            Some(Kind::NumberValue(number)) => serde_json::Value::from(*number),
            Some(Kind::StringValue(text)) => serde_json::Value::String(text.clone()),
            Some(Kind::BoolValue(value)) => serde_json::Value::Bool(*value),
            Some(Kind::StructValue(object)) => struct_to_json_object(object),
            Some(Kind::ListValue(list)) => {
                serde_json::Value::Array(list.values.iter().map(prost_value_to_json).collect())
            }
        }
    }

    fn normalize_json_numbers(value: serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Array(values) => {
                serde_json::Value::Array(values.into_iter().map(normalize_json_numbers).collect())
            }
            serde_json::Value::Object(map) => serde_json::Value::Object(
                map.into_iter()
                    .map(|(key, value)| (key, normalize_json_numbers(value)))
                    .collect(),
            ),
            serde_json::Value::Number(number) => normalize_json_number(number),
            other => other,
        }
    }

    fn normalize_json_number(number: serde_json::Number) -> serde_json::Value {
        let fallback = number.clone();
        number.as_f64().map_or_else(
            || serde_json::Value::Number(fallback),
            |value| {
                if value.fract() == 0.0 && value.abs() <= MAX_SAFE_INTEGER_F64 {
                    value
                        .to_string()
                        .parse::<i64>()
                        .map(|integer| serde_json::Value::Number(serde_json::Number::from(integer)))
                        .unwrap_or(serde_json::Value::Number(number))
                } else {
                    serde_json::Value::Number(number)
                }
            },
        )
    }

    fn pretty_json(value: serde_json::Value) -> String {
        serde_json::to_string_pretty(&value).unwrap_or_default()
    }

    async fn gate_hello(
        socket: &mut tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        service_token: Option<String>,
        actor: String,
    ) -> Result<ServerHello, String> {
        let envelope = ClientEnvelope {
            body: Some(client_envelope::Body::Hello(ClientHello {
                protocol_version: "prior.gate.v1".into(),
                client_name: "prior-web".into(),
                metadata: HashMap::from([("surface".into(), "prior-web".into())]),
                service_token,
                user_id: Some(actor),
            })),
        };
        let mut bytes = Vec::new();
        envelope
            .encode(&mut bytes)
            .map_err(|error| format!("encode envelope: {error}"))?;
        socket
            .send(GateWsMessage::Binary(bytes.into()))
            .await
            .map_err(|error| format!("write gate hello: {error}"))?;

        loop {
            let message = socket
                .next()
                .await
                .ok_or_else(|| "gate hello read: websocket closed".to_string())?
                .map_err(|error| format!("gate hello read: {error}"))?;
            match message {
                GateWsMessage::Binary(bytes) => {
                    let envelope = ServerEnvelope::decode(bytes.as_ref())
                        .map_err(|error| format!("decode envelope: {error}"))?;
                    match envelope.body {
                        Some(server_envelope::Body::Hello(hello)) => return Ok(hello),
                        _ => return Err("expected server hello".into()),
                    }
                }
                GateWsMessage::Ping(payload) => {
                    socket
                        .send(GateWsMessage::Pong(payload))
                        .await
                        .map_err(|error| format!("write gate pong: {error}"))?;
                }
                GateWsMessage::Close(_) => return Err("gate hello read: websocket closed".into()),
                GateWsMessage::Text(_) | GateWsMessage::Pong(_) | GateWsMessage::Frame(_) => {}
            }
        }
    }
}

#[cfg(feature = "ssr")]
pub use ssr::ws_handler;
