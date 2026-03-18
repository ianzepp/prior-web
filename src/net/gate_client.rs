#[cfg(feature = "hydrate")]
use crate::state::gate::{ConnectionStatus, GateUiState};

#[cfg(feature = "hydrate")]
use futures::channel::mpsc;
#[cfg(feature = "hydrate")]
use futures::{SinkExt, StreamExt};
#[cfg(feature = "hydrate")]
use gloo_net::websocket::{Message, futures::WebSocket};
#[cfg(feature = "hydrate")]
use leptos::prelude::*;

#[derive(Clone, Debug)]
pub enum GateCommand {
    RefreshRooms,
}

#[cfg(feature = "hydrate")]
pub fn spawn_gate_client(gate: RwSignal<GateUiState>) -> mpsc::UnboundedSender<GateCommand> {
    let (tx, rx) = mpsc::unbounded();
    leptos::task::spawn_local(async move {
        if let Err(error) = gate_client_loop(gate, rx).await {
            gate.update(|state| {
                state.connection = ConnectionStatus::Disconnected;
                state.status = error;
            });
        }
    });
    tx
}

#[cfg(feature = "hydrate")]
async fn gate_client_loop(
    gate: RwSignal<GateUiState>,
    mut rx: mpsc::UnboundedReceiver<GateCommand>,
) -> Result<(), String> {
    gate.update(|state| {
        state.connection = ConnectionStatus::Connecting;
        state.status = "connecting to gate".into();
    });

    let url = gate_ws_url();
    let socket = WebSocket::open(&url).map_err(|error| format!("open websocket: {error}"))?;
    let (mut write, mut read) = socket.split();

    gate.update(|state| {
        state.connection = ConnectionStatus::Connected;
        state.gate_url = url.clone();
        state.status = "connected to gate websocket".into();
    });

    while let Some(command) = rx.next().await {
        match command {
            GateCommand::RefreshRooms => {
                let message = "prior-web gate client scaffold: room refresh wiring pending";
                write
                    .send(Message::Text(message.into()))
                    .await
                    .map_err(|error| format!("send: {error}"))?;
                gate.update(|state| state.status = "refresh requested; websocket client scaffold active".into());
            }
        }

        if let Some(frame) = read.next().await {
            let frame = frame.map_err(|error| format!("recv: {error}"))?;
            if let Message::Text(text) = frame {
                gate.update(|state| state.last_event = Some(text));
            }
        }
    }

    Ok(())
}

#[cfg(feature = "hydrate")]
fn gate_ws_url() -> String {
    crate::runtime::gate_ws_url()
}
