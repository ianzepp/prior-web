#[cfg(feature = "ssr")]
use leptos::config::LeptosOptions;

#[cfg(feature = "ssr")]
#[derive(Clone, Debug)]
pub struct PriorGateConfig {
    pub tcp_addr: String,
    pub service_token: Option<String>,
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
        tcp_addr: std::env::var("PRIOR_GATE_TCP_ADDR").unwrap_or_else(|_| "127.0.0.1:7070".into()),
        service_token: std::env::var("PRIOR_GATE_SERVICE_TOKEN").ok().filter(|value| !value.is_empty()),
    }
}

pub fn prior_web_gate_actor() -> String {
    std::env::var("PRIOR_WEB_GATE_ACTOR").unwrap_or_else(|_| "prior-web".into())
}
