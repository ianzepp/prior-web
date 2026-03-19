use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentUser {
    pub sub: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}

impl CurrentUser {
    #[must_use]
    pub fn label(&self) -> String {
        self.display_name
            .clone()
            .or_else(|| self.email.clone())
            .unwrap_or_else(|| self.sub.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthState {
    Anonymous,
    Authenticated(CurrentUser),
}
