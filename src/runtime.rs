#[cfg(feature = "hydrate")]
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use leptos::config::LeptosOptions;

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
pub fn gate_ws_meta_content() -> String {
    std::env::var("PRIOR_GATE_WS_URL").unwrap_or_default()
}

#[cfg(feature = "hydrate")]
pub fn gate_ws_url() -> String {
    if let Some(url) = configured_gate_ws_url() {
        return url;
    }
    fallback_gate_ws_url()
}

#[cfg(feature = "hydrate")]
fn configured_gate_ws_url() -> Option<String> {
    let window = web_sys::window()?;
    let document = window.document()?;
    let meta = document
        .query_selector("meta[name='prior-gate-ws-url']")
        .ok()
        .flatten()?;
    let content = meta.get_attribute("content")?;
    if content.is_empty() {
        return None;
    }
    Some(content)
}

#[cfg(feature = "hydrate")]
fn fallback_gate_ws_url() -> String {
    let Some(window) = web_sys::window() else {
        return "ws://127.0.0.1:7071/".into();
    };
    let location = window.location();
    let protocol = match location.protocol() {
        Ok(value) if value == "https:" => "wss",
        Ok(_) => "ws",
        Err(_) => "ws",
    };
    let host = match location.hostname() {
        Ok(value) if !value.is_empty() => value,
        _ => "127.0.0.1".into(),
    };
    format!("{protocol}://{host}:7071/")
}
