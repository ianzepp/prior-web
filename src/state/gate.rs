use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ConnectionStatus {
    #[default]
    Connecting,
    Connected,
    Disconnected,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GateUiState {
    pub connection: ConnectionStatus,
    pub gate_url: String,
    pub server_name: Option<String>,
    pub status: String,
    pub rooms: Vec<String>,
    pub last_event: Option<String>,
}

impl GateUiState {
    #[must_use]
    pub fn loading(status: &str) -> Self {
        Self {
            connection: ConnectionStatus::Connecting,
            gate_url: "server-owned transport".into(),
            server_name: None,
            status: status.into(),
            rooms: Vec::new(),
            last_event: None,
        }
    }

    #[must_use]
    pub fn disconnected(gate_url: &str, status: String) -> Self {
        Self {
            connection: ConnectionStatus::Disconnected,
            gate_url: gate_url.into(),
            server_name: None,
            status,
            rooms: Vec::new(),
            last_event: None,
        }
    }
}
