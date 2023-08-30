use std::path::Path;
use std::time::Duration;

use crate::config::Config;
use crate::router::state::AppState;
use axum::error_handling::HandleErrorLayer;
use axum::{
    http::{StatusCode, Uri},
    middleware,
    routing::get,
    Extension, Router,
};
use tower::ServiceBuilder;

mod auth;
mod debug;
mod error;
mod fapi;
mod spa;
mod state;

pub fn router(static_folder: &Path, config: &Config) -> anyhow::Result<shuttle_axum::AxumService> {
    let router = Router::new()
        .route("/", get(spa::welcome_handler))
        .nest("/auth", auth::router())
        .route("/*path", get(spa::spa_handler))
        .layer(Extension(spa::static_services(static_folder)))
        .layer(middleware::from_fn(debug::log_request_response))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(error::handle))
                .timeout(Duration::from_secs(10)),
        )
        .fallback(fallback)
        .with_state(AppState::new(config)?);

    Ok(router.into())
}

#[allow(clippy::unused_async)]
async fn fallback(uri: Uri) -> (StatusCode, String) {
    let message = format!("I couldn't find '{}'. Try something else?", uri.path());
    (StatusCode::NOT_FOUND, message)
}
