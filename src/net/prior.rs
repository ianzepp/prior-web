use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use server_fn::error::ServerFnError;

use crate::state::gate::GateUiState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomActor {
    pub from: String,
    pub config: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomHistoryEntry {
    pub id: u64,
    pub ts: i64,
    pub from: String,
    pub content: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomMessageEntry {
    pub actor: Option<String>,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoEntry {
    pub id: i64,
    pub ts: i64,
    pub owner: String,
    pub name: String,
    pub room: String,
    pub clone_url: String,
    pub bare_path: String,
    pub default_base_branch: Option<String>,
    pub last_sync_ts: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoImportResult {
    pub room: String,
    pub bare_path: String,
}

#[server]
pub async fn refresh_dashboard() -> Result<GateUiState, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return client::refresh_dashboard()
            .await
            .map_err(ServerFnError::new);
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "refresh_dashboard is only available on the server",
    ))
}

#[server]
pub async fn list_known_rooms() -> Result<Vec<String>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return client::list_known_rooms().await.map_err(ServerFnError::new);
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "list_known_rooms is only available on the server",
    ))
}

#[server]
pub async fn list_room_actors(room: String) -> Result<Vec<RoomActor>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return client::list_room_actors(&room)
            .await
            .map_err(ServerFnError::new);
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "list_room_actors is only available on the server",
    ))
}

#[server]
pub async fn list_room_history(
    room: String,
    before: Option<i64>,
    limit: Option<usize>,
) -> Result<Vec<RoomHistoryEntry>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return client::list_room_history(&room, before, limit)
            .await
            .map_err(ServerFnError::new);
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "list_room_history is only available on the server",
    ))
}

#[server]
pub async fn send_room_message(
    room: String,
    content: String,
) -> Result<Vec<RoomMessageEntry>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return client::send_room_message(&room, &content)
            .await
            .map_err(ServerFnError::new);
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "send_room_message is only available on the server",
    ))
}

#[server]
pub async fn list_repos() -> Result<Vec<RepoEntry>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return client::list_repos().await.map_err(ServerFnError::new);
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "list_repos is only available on the server",
    ))
}

#[server]
pub async fn import_repo(
    clone_url: String,
    owner: String,
    name: String,
    auth_token: Option<String>,
) -> Result<RepoImportResult, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return client::import_repo(&clone_url, &owner, &name, auth_token.as_deref())
            .await
            .map_err(ServerFnError::new);
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "import_repo is only available on the server",
    ))
}

#[cfg(feature = "ssr")]
pub(crate) mod client {
    use std::collections::HashMap;

    use futures_util::{SinkExt, StreamExt};
    use prost::Message;
    use prost_types::{Struct, Value, value::Kind};
    use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};

    use crate::auth::user::{require_current_user, require_github_access_token};
    use crate::net::prior_gate_proto::{
        ClientEnvelope, ClientHello, GateRequest, ResponseItem, ResponseOp, ServerEnvelope,
        ServerHello, client_envelope, server_envelope,
    };
    use crate::runtime::prior_gate_config;
    use crate::state::gate::{ConnectionStatus, GateUiState};

    use super::{RepoEntry, RepoImportResult, RoomActor, RoomHistoryEntry, RoomMessageEntry};

    const SECRET_AUTH_TOKEN: &str = "auth_token";
    const MAX_SAFE_INTEGER_F64: f64 = 9_007_199_254_740_991.0;

    pub async fn refresh_dashboard() -> Result<GateUiState, String> {
        let actor = require_current_user()?.sub;

        let result = async {
            let mut session = connect_session(&actor, None).await?;
            let mut last_event = None;
            let rooms = session.list_known_rooms(&mut last_event).await?;
            let gate_url = session.config.ws_url.clone();
            let server_name = session.server_name.clone();
            disconnect_with_result(
                &mut session,
                Ok(GateUiState {
                    connection: ConnectionStatus::Connected,
                    gate_url,
                    server_name: Some(server_name),
                    status: format!("server-owned gate round trip ok for user {actor}"),
                    rooms,
                    last_event,
                }),
            )
            .await
        }
        .await;

        result.or_else(|error| {
            let config = prior_gate_config();
            Ok(GateUiState {
                connection: ConnectionStatus::Disconnected,
                gate_url: config.ws_url,
                server_name: None,
                status: format!("server-side gate refresh failed: {error}"),
                rooms: Vec::new(),
                last_event: None,
            })
        })
    }

    pub async fn list_known_rooms() -> Result<Vec<String>, String> {
        let actor = require_current_user()?.sub;
        let mut session = connect_session(&actor, None).await?;
        let mut last_event = None;
        let result = session.list_known_rooms(&mut last_event).await;
        disconnect_with_result(&mut session, result).await
    }

    pub async fn list_room_actors(room: &str) -> Result<Vec<RoomActor>, String> {
        let actor = require_current_user()?.sub;
        let mut session = connect_session(&actor, None).await?;
        let result = session
            .request_items("room:list", Some(room), Struct::default(), None)
            .await;
        disconnect_with_result(&mut session, result).await
    }

    pub async fn list_room_history(
        room: &str,
        before: Option<i64>,
        limit: Option<usize>,
    ) -> Result<Vec<RoomHistoryEntry>, String> {
        let actor = require_current_user()?.sub;
        let mut session = connect_session(&actor, None).await?;
        let mut pairs = Vec::new();
        if let Some(before) = before {
            pairs.push(("before", i64_number_value(before)?));
        }
        if let Some(limit) = limit {
            pairs.push(("limit", usize_number_value(limit)?));
        }
        let result = session
            .request_items("room:history", Some(room), struct_from_vec(pairs), None)
            .await;
        disconnect_with_result(&mut session, result).await
    }

    pub async fn send_room_message(
        room: &str,
        content: &str,
    ) -> Result<Vec<RoomMessageEntry>, String> {
        let actor = require_current_user()?.sub;
        let mut session = connect_session(&actor, Some(room)).await?;
        let result = session.send_message(room, content).await;
        disconnect_with_result(&mut session, result).await
    }

    pub async fn list_repos() -> Result<Vec<RepoEntry>, String> {
        let actor = require_current_user()?.sub;
        let mut session = connect_session(&actor, None).await?;
        let result = session
            .request_items::<RepoEntry>("repo:list", None, Struct::default(), None)
            .await;
        disconnect_with_result(&mut session, result).await
    }

    pub async fn import_repo(
        clone_url: &str,
        owner: &str,
        name: &str,
        auth_token: Option<&str>,
    ) -> Result<RepoImportResult, String> {
        let actor = require_current_user()?.sub;
        let mut session = connect_session(&actor, None).await?;
        let session_token = require_github_access_token()
            .ok()
            .map(|token| token.access_token);
        let effective_token = auth_token.map(ToOwned::to_owned).or(session_token);
        let secrets = effective_token
            .as_deref()
            .map(|token| struct_from_vec(vec![(SECRET_AUTH_TOKEN, string_value(token))]));
        let data = struct_from_vec(vec![
            ("clone_url", string_value(clone_url)),
            ("owner", string_value(owner)),
            ("name", string_value(name)),
        ]);
        let result = session
            .request_one("repo:import", None, data, secrets)
            .await;
        disconnect_with_result(&mut session, result).await
    }

    pub(crate) async fn connect_session(
        actor: &str,
        room: Option<&str>,
    ) -> Result<PriorSession, String> {
        let config = prior_gate_config();
        let mut client = PriorGateClient::connect(&config.ws_url).await?;
        let hello = client
            .hello(config.service_token.clone(), actor.to_string())
            .await?;
        let session_id = client.connect_session(actor).await?;
        let mut session = PriorSession {
            client,
            config,
            server_name: hello.server_name,
            session_id,
        };

        if let Some(room) = room {
            session.join_room(room).await?;
        }

        Ok(session)
    }

    pub(crate) async fn disconnect_with_result<T>(
        session: &mut PriorSession,
        result: Result<T, String>,
    ) -> Result<T, String> {
        let disconnect_result = session.disconnect().await;
        match (result, disconnect_result) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(error), _) | (Ok(_), Err(error)) => Err(error),
        }
    }

    pub(crate) struct PriorSession {
        pub(crate) client: PriorGateClient,
        pub(crate) config: crate::runtime::PriorGateConfig,
        pub(crate) server_name: String,
        pub(crate) session_id: String,
    }

    impl PriorSession {
        pub(crate) async fn list_known_rooms(
            &mut self,
            last_event: &mut Option<String>,
        ) -> Result<Vec<String>, String> {
            let responses = self
                .client
                .request_raw("door:rooms", None, Struct::default(), None, last_event)
                .await?;
            let mut rooms = responses
                .iter()
                .filter_map(|response| response.item.as_ref())
                .filter_map(|item| extract_string_field(item, "room"))
                .collect::<Vec<_>>();
            rooms.sort();
            rooms.dedup();
            Ok(rooms)
        }

        pub(crate) async fn join_room(&mut self, room: &str) -> Result<(), String> {
            let data = struct_from_vec(vec![
                ("session", string_value(&self.session_id)),
                ("room", string_value(room)),
            ]);
            self.client
                .request_done("door:join", None, data, None, &mut None)
                .await
        }

        pub(crate) async fn send_message(
            &mut self,
            room: &str,
            content: &str,
        ) -> Result<Vec<RoomMessageEntry>, String> {
            let data = struct_from_vec(vec![
                ("session", string_value(&self.session_id)),
                ("room", string_value(room)),
                ("content", string_value(content)),
            ]);
            self.request_items("door:message", None, data, None).await
        }

        pub(crate) async fn request_one<T: serde::de::DeserializeOwned>(
            &mut self,
            syscall: &str,
            room: Option<&str>,
            data: Struct,
            secrets: Option<Struct>,
        ) -> Result<T, String> {
            let mut items = self.request_items(syscall, room, data, secrets).await?;
            items
                .pop()
                .ok_or_else(|| format!("{syscall} returned no items"))
        }

        pub(crate) async fn request_items<T: serde::de::DeserializeOwned>(
            &mut self,
            syscall: &str,
            room: Option<&str>,
            data: Struct,
            secrets: Option<Struct>,
        ) -> Result<Vec<T>, String> {
            let responses = self
                .client
                .request_raw(syscall, room, data, secrets, &mut None)
                .await?;

            responses
                .iter()
                .filter_map(|response| response.item.as_ref())
                .map(item_to_typed)
                .collect()
        }

        pub(crate) async fn disconnect(&mut self) -> Result<(), String> {
            let data = struct_from_vec(vec![("session", string_value(&self.session_id))]);
            self.client
                .request_done("door:disconnect", None, data, None, &mut None)
                .await
        }
    }

    pub(crate) struct PriorGateClient {
        socket: tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        next_request_id: u64,
    }

    impl PriorGateClient {
        async fn connect(url: &str) -> Result<Self, String> {
            let (socket, _) = connect_async(url)
                .await
                .map_err(|error| format!("connect {url}: {error}"))?;
            Ok(Self {
                socket,
                next_request_id: 1,
            })
        }

        async fn hello(
            &mut self,
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
            self.send(&envelope).await?;

            let server = self.read().await?;
            match server.body {
                Some(server_envelope::Body::Hello(hello)) => Ok(hello),
                _ => Err("expected server hello".into()),
            }
        }

        async fn connect_session(&mut self, actor: &str) -> Result<String, String> {
            let responses = self
                .request_raw(
                    "door:connect",
                    None,
                    struct_from_vec(vec![("from", string_value(actor))]),
                    None,
                    &mut None,
                )
                .await?;

            responses
                .iter()
                .filter_map(|response| response.item.as_ref())
                .find_map(|item| extract_string_field(item, "session"))
                .ok_or_else(|| "door:connect did not return a session id".into())
        }

        async fn request_done(
            &mut self,
            syscall: &str,
            room: Option<&str>,
            data: Struct,
            secrets: Option<Struct>,
            last_event: &mut Option<String>,
        ) -> Result<(), String> {
            self.request_raw(syscall, room, data, secrets, last_event)
                .await
                .map(|_| ())
        }

        async fn request_raw(
            &mut self,
            syscall: &str,
            room: Option<&str>,
            data: Struct,
            secrets: Option<Struct>,
            last_event: &mut Option<String>,
        ) -> Result<Vec<crate::net::prior_gate_proto::GateResponse>, String> {
            let request_id = self.next_request_id();
            let envelope = ClientEnvelope {
                body: Some(client_envelope::Body::Request(GateRequest {
                    request_id: request_id.clone(),
                    syscall: syscall.into(),
                    room: room.map(ToOwned::to_owned),
                    from: None,
                    timeout_ms: None,
                    data: Some(data),
                    trace: None,
                    secrets,
                })),
            };
            self.send(&envelope).await?;

            let mut responses = Vec::new();
            loop {
                let envelope = self.read().await?;
                match envelope.body {
                    Some(server_envelope::Body::Response(response))
                        if response.request_id == request_id =>
                    {
                        if response.op == ResponseOp::Error as i32 {
                            let message = response.error.as_ref().map_or_else(
                                || "request failed".to_string(),
                                |body| format!("{}: {}", body.code, body.message),
                            );
                            return Err(format!("{syscall} failed: {message}"));
                        }

                        let terminal = matches!(
                            ResponseOp::try_from(response.op),
                            Ok(ResponseOp::Done | ResponseOp::Cancel)
                        );
                        responses.push(response);
                        if terminal {
                            return Ok(responses);
                        }
                    }
                    Some(server_envelope::Body::Event(event)) => {
                        *last_event = Some(format!("{}: {}", event.topic, event.event_id));
                    }
                    Some(
                        server_envelope::Body::Pong(_)
                        | server_envelope::Body::Hello(_)
                        | server_envelope::Body::Response(_),
                    )
                    | None => {}
                }
            }
        }

        fn next_request_id(&mut self) -> String {
            let request_id = format!("prior-web-{}", self.next_request_id);
            self.next_request_id += 1;
            request_id
        }

        async fn send(&mut self, envelope: &ClientEnvelope) -> Result<(), String> {
            let mut bytes = Vec::new();
            envelope
                .encode(&mut bytes)
                .map_err(|error| format!("encode envelope: {error}"))?;
            self.socket
                .send(WsMessage::Binary(bytes.into()))
                .await
                .map_err(|error| format!("write envelope payload: {error}"))
        }

        async fn read(&mut self) -> Result<ServerEnvelope, String> {
            loop {
                let message = self
                    .socket
                    .next()
                    .await
                    .ok_or_else(|| "read envelope payload: websocket closed".to_string())?
                    .map_err(|error| format!("read envelope payload: {error}"))?;

                match message {
                    WsMessage::Binary(bytes) => {
                        return ServerEnvelope::decode(bytes.as_ref())
                            .map_err(|error| format!("decode envelope: {error}"));
                    }
                    WsMessage::Ping(payload) => {
                        self.socket
                            .send(WsMessage::Pong(payload))
                            .await
                            .map_err(|error| format!("write pong: {error}"))?;
                    }
                    WsMessage::Close(_) => {
                        return Err("read envelope payload: websocket closed".into());
                    }
                    WsMessage::Pong(_) | WsMessage::Text(_) | WsMessage::Frame(_) => {}
                }
            }
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

    pub(crate) fn struct_from_vec(pairs: Vec<(&str, Value)>) -> Struct {
        Struct {
            fields: pairs
                .into_iter()
                .map(|(key, value)| (key.to_string(), value))
                .collect(),
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

    #[allow(clippy::cast_possible_truncation)]
    fn normalize_json_number(number: serde_json::Number) -> serde_json::Value {
        let Some(value) = number.as_f64() else {
            return serde_json::Value::Number(number);
        };

        if !value.is_finite() || value.fract() != 0.0 {
            return serde_json::Value::Number(number);
        }

        if !(-MAX_SAFE_INTEGER_F64..=MAX_SAFE_INTEGER_F64).contains(&value) {
            return serde_json::Value::Number(number);
        }

        let integer = value as i64;
        serde_json::Value::Number(serde_json::Number::from(integer))
    }

    pub(crate) fn string_value(value: &str) -> Value {
        Value {
            kind: Some(Kind::StringValue(value.to_string())),
        }
    }

    pub(crate) fn number_value(value: f64) -> Value {
        Value {
            kind: Some(Kind::NumberValue(value)),
        }
    }

    #[allow(clippy::cast_precision_loss)]
    pub(crate) fn i64_number_value(value: i64) -> Result<Value, String> {
        const MAX_SAFE_INTEGER: i64 = 9_007_199_254_740_991;

        if !(-MAX_SAFE_INTEGER..=MAX_SAFE_INTEGER).contains(&value) {
            return Err(format!(
                "integer {value} exceeds protobuf Struct safe numeric range"
            ));
        }

        Ok(number_value(value as f64))
    }

    #[allow(clippy::cast_precision_loss)]
    pub(crate) fn usize_number_value(value: usize) -> Result<Value, String> {
        const MAX_SAFE_INTEGER: usize = 9_007_199_254_740_991;

        if value > MAX_SAFE_INTEGER {
            return Err(format!(
                "integer {value} exceeds protobuf Struct safe numeric range"
            ));
        }

        Ok(number_value(value as f64))
    }

    fn value_as_string(value: &Value) -> Option<String> {
        match value.kind.as_ref() {
            Some(Kind::StringValue(value)) => Some(value.clone()),
            _ => None,
        }
    }
}
