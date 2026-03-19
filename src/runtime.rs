#[cfg(feature = "ssr")]
use axum::extract::FromRef;
#[cfg(feature = "ssr")]
use axum_extra::extract::cookie::Key;
#[cfg(feature = "ssr")]
use leptos::config::LeptosOptions;

#[cfg(feature = "ssr")]
use crate::auth::config::AuthRuntime;

#[cfg(feature = "ssr")]
#[derive(Clone, Debug)]
pub struct PriorGateConfig {
    pub ws_url: String,
    pub service_token: Option<String>,
}

#[cfg(feature = "ssr")]
#[derive(Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub gate: PriorGateConfig,
    pub auth: Option<AuthRuntime>,
}

#[cfg(feature = "ssr")]
pub fn site_addr(options: &LeptosOptions) -> String {
    if let Ok(addr) = std::env::var("PRIOR_WEB_SITE_ADDR") {
        return addr;
    }
    if let Ok(port) = std::env::var("PORT") {
        return format!("0.0.0.0:{port}");
    }
    options.site_addr.to_string()
}

#[cfg(feature = "ssr")]
pub fn prior_gate_config() -> PriorGateConfig {
    PriorGateConfig {
        ws_url: std::env::var("PRIOR_GATE_WS_URL")
            .unwrap_or_else(|_| "ws://127.0.0.1:7071/".into()),
        service_token: std::env::var("PRIOR_GATE_SERVICE_TOKEN")
            .ok()
            .filter(|value| !value.is_empty()),
    }
}

pub fn prior_web_gate_actor() -> String {
    std::env::var("PRIOR_WEB_GATE_ACTOR").unwrap_or_else(|_| "prior-web".into())
}

#[cfg(feature = "ssr")]
impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.leptos_options.clone()
    }
}

#[cfg(feature = "ssr")]
impl FromRef<AppState> for PriorGateConfig {
    fn from_ref(state: &AppState) -> Self {
        state.gate.clone()
    }
}

#[cfg(feature = "ssr")]
impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state
            .auth
            .as_ref()
            .expect("cookie key requested without auth runtime")
            .cookie_key
            .clone()
    }
}
