#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionStatus {
    #[default]
    Connecting,
    Connected,
    Disconnected,
}

#[derive(Debug, Clone, Default)]
pub struct GateUiState {
    pub connection: ConnectionStatus,
    pub gate_url: String,
    pub server_name: Option<String>,
    pub status: String,
    pub rooms: Vec<String>,
    pub last_event: Option<String>,
}
