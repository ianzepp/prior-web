#![cfg(feature = "ssr")]

use std::path::PathBuf;

use axum::Router;
use leptos::config::get_configuration;
use leptos_axum::{LeptosRoutes, generate_route_list};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    if let Err(error) = run().await {
        tracing::error!(%error, "prior-web failed");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    let conf = get_configuration(None).map_err(|error| format!("leptos config: {error}"))?;
    let options = conf.leptos_options;
    let routes = generate_route_list(prior_web::app::App);
    let site_root = PathBuf::from(options.site_root.as_ref());
    let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("public");
    let site_addr = prior_web::runtime::site_addr(&options);

    let app = Router::new()
        .leptos_routes(&options, routes, {
            let options = options.clone();
            move || prior_web::app::shell(options.clone())
        })
        .nest_service("/pkg", ServeDir::new(site_root.join("pkg")))
        .nest_service("/", ServeDir::new(assets).append_index_html_on_directories(true))
        .with_state(options.clone());

    let listener = tokio::net::TcpListener::bind(&site_addr)
        .await
        .map_err(|error| format!("bind {site_addr}: {error}"))?;
    tracing::info!(addr = %site_addr, "prior-web listening");
    axum::serve(listener, app)
        .await
        .map_err(|error| format!("serve: {error}"))?;
    Ok(())
}
