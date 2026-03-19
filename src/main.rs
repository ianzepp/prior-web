#![cfg(feature = "ssr")]

use std::path::PathBuf;

use axum::Router;
use leptos::config::get_configuration;
use leptos_axum::{LeptosRoutes, generate_route_list};
use tower_http::services::ServeDir;

use prior_web::runtime::AppState;

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
    let gate = prior_web::runtime::prior_gate_config();
    let auth = prior_web::auth::config::load_auth_runtime().await?;
    let state = AppState {
        leptos_options: options.clone(),
        gate,
        auth,
    };

    let app = Router::new()
        .route(
            "/auth/login",
            axum::routing::get(prior_web::auth::routes::login),
        )
        .route(
            "/auth/callback",
            axum::routing::get(prior_web::auth::routes::callback),
        )
        .route(
            "/auth/logout",
            axum::routing::get(prior_web::auth::routes::logout)
                .post(prior_web::auth::routes::logout),
        )
        .leptos_routes_with_context(&state, routes, || {}, {
            let options = options.clone();
            move || prior_web::app::shell(options.clone())
        })
        .nest_service("/pkg", ServeDir::new(site_root.join("pkg")))
        .fallback_service(ServeDir::new(assets).append_index_html_on_directories(true))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&site_addr)
        .await
        .map_err(|error| format!("bind {site_addr}: {error}"))?;
    tracing::info!(addr = %site_addr, "prior-web listening");
    axum::serve(listener, app)
        .await
        .map_err(|error| format!("serve: {error}"))?;
    Ok(())
}
