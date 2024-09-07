#![deny(clippy::pedantic)]

use axum::{middleware, routing::get, Router};

use error::Result;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;
mod error;
mod http_tracing;
mod model;
mod project_auth_cookie;
mod proxy;

const BIND_ADDRESS: &str = "0.0.0.0:3000";
#[tokio::main]
async fn main() -> Result<()> {
    // Setup logs
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let app_state = model::AppState::from_env()?;

    let app_router = Router::new()
        .route("/", get(proxy::handler))
        .route("/*path", get(proxy::handler))
        .with_state(app_state.clone())
        .layer(middleware::map_request_with_state(
            app_state.clone(),
            project_auth_cookie::auth_layer,
        ))
        .layer(middleware::from_fn(http_tracing::log_requests));

    let listener = tokio::net::TcpListener::bind(BIND_ADDRESS).await?;

    axum::serve(listener, app_router).await?;
    println!(" 🔌 Server started at {BIND_ADDRESS}");

    Ok(())
}
