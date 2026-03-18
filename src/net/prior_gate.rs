use leptos::prelude::*;
use server_fn::error::ServerFnError;

use crate::state::gate::GateUiState;

#[server]
pub async fn refresh_dashboard() -> Result<GateUiState, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return Ok(ssr::refresh_dashboard().await);
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new("refresh_dashboard is only available on the server"))
}

#[cfg(feature = "ssr")]
mod ssr {
    use std::collections::HashMap;

    use futures_util::{SinkExt, StreamExt};
    use prost::Message;
    use prost_types::{Struct, Value, value::Kind};
    use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};

    use crate::net::prior_gate_proto::{
        ClientEnvelope, ClientHello, GateRequest, ResponseItem, ResponseOp, ServerEnvelope, ServerHello,
        client_envelope, server_envelope,
    };
    use crate::runtime::{prior_gate_config, prior_web_gate_actor};
    use crate::state::gate::{ConnectionStatus, GateUiState};

    pub async fn refresh_dashboard() -> GateUiState {
        let config = prior_gate_config();

        match load_snapshot().await {
            Ok(state) => state,
            Err(error) => GateUiState {
                connection: ConnectionStatus::Disconnected,
                gate_url: config.ws_url,
                server_name: None,
                status: format!("server-side gate refresh failed: {error}"),
                rooms: Vec::new(),
                last_event: None,
            },
        }
    }

    async fn load_snapshot() -> Result<GateUiState, String> {
        let config = prior_gate_config();
        let actor = prior_web_gate_actor();
        let mut client = PriorGateClient::connect(&config.ws_url).await?;
        let hello = client
            .hello(config.service_token.clone(), actor.clone())
            .await?;
        let mut last_event = None;
        let session_id = client.connect_session(&actor, &mut last_event).await?;
        let rooms_result = client.list_rooms(&mut last_event).await;
        let disconnect_result = client.disconnect(&session_id, &mut last_event).await;

        let rooms = rooms_result?;
        disconnect_result?;

        Ok(GateUiState {
            connection: ConnectionStatus::Connected,
            gate_url: config.ws_url,
            server_name: Some(hello.server_name),
            status: format!("server-owned gate round trip ok for actor {actor}"),
            rooms,
            last_event,
        })
    }

    struct PriorGateClient {
        socket: tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
        next_request_id: u64,
    }

    impl PriorGateClient {
        async fn connect(url: &str) -> Result<Self, String> {
            let (socket, _) = connect_async(url)
                .await
                .map_err(|error| format!("connect {url}: {error}"))?;
            Ok(Self { socket, next_request_id: 1 })
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
                    metadata: HashMap::from([("surface".into(), "server-dashboard".into())]),
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

        async fn connect_session(
            &mut self,
            actor: &str,
            last_event: &mut Option<String>,
        ) -> Result<String, String> {
            let responses = self
                .request(
                    "door:connect",
                    struct_from_pairs([("from", string_value(actor))]),
                    last_event,
                )
                .await?;

            responses
                .iter()
                .find_map(|response| response.item.as_ref())
                .and_then(response_item_session)
                .ok_or_else(|| "door:connect did not return a session id".into())
        }

        async fn list_rooms(&mut self, last_event: &mut Option<String>) -> Result<Vec<String>, String> {
            let responses = self.request("door:rooms", Struct::default(), last_event).await?;
            let mut rooms = responses
                .iter()
                .filter_map(|response| response.item.as_ref())
                .filter_map(response_item_room)
                .collect::<Vec<_>>();
            rooms.sort();
            rooms.dedup();
            Ok(rooms)
        }

        async fn disconnect(
            &mut self,
            session_id: &str,
            last_event: &mut Option<String>,
        ) -> Result<(), String> {
            self.request(
                "door:disconnect",
                struct_from_pairs([("session", string_value(session_id))]),
                last_event,
            )
            .await
            .map(|_| ())
        }

        async fn request(
            &mut self,
            syscall: &str,
            data: Struct,
            last_event: &mut Option<String>,
        ) -> Result<Vec<crate::net::prior_gate_proto::GateResponse>, String> {
            let request_id = self.next_request_id();
            let envelope = ClientEnvelope {
                body: Some(client_envelope::Body::Request(GateRequest {
                    request_id: request_id.clone(),
                    syscall: syscall.into(),
                    room: None,
                    from: None,
                    timeout_ms: None,
                    data: Some(data),
                    trace: None,
                    secrets: None,
                })),
            };
            self.send(&envelope).await?;

            let mut responses = Vec::new();
            loop {
                let envelope = self.read().await?;
                match envelope.body {
                    Some(server_envelope::Body::Response(response)) if response.request_id == request_id => {
                        if response.op == ResponseOp::Error as i32 {
                            let message = response
                                .error
                                .as_ref()
                                .map_or_else(|| "request failed".to_string(), |body| {
                                    format!("{}: {}", body.code, body.message)
                                });
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
                    Some(server_envelope::Body::Pong(_)) => {}
                    Some(server_envelope::Body::Hello(_)) => {}
                    Some(server_envelope::Body::Response(_)) | None => {}
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
                    WsMessage::Close(_) => return Err("read envelope payload: websocket closed".into()),
                    WsMessage::Pong(_) => {}
                    WsMessage::Text(_) => {}
                    WsMessage::Frame(_) => {}
                }
            }
        }
    }

    fn struct_from_pairs<const N: usize>(pairs: [(&str, Value); N]) -> Struct {
        Struct {
            fields: pairs
                .into_iter()
                .map(|(key, value)| (key.to_string(), value))
                .collect(),
        }
    }

    fn string_value(value: &str) -> Value {
        Value { kind: Some(Kind::StringValue(value.to_string())) }
    }

    fn response_item_session(item: &ResponseItem) -> Option<String> {
        item.data
            .as_ref()
            .and_then(|data| data.fields.get("session"))
            .and_then(value_as_string)
    }

    fn response_item_room(item: &ResponseItem) -> Option<String> {
        item.data
            .as_ref()
            .and_then(|data| data.fields.get("room"))
            .and_then(value_as_string)
    }

    fn value_as_string(value: &Value) -> Option<String> {
        match value.kind.as_ref() {
            Some(Kind::StringValue(value)) => Some(value.clone()),
            _ => None,
        }
    }
}
