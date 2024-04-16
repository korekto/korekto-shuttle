use std::time::Duration;

use crate::router::state::AppState;
use axum::error_handling::HandleErrorLayer;
use axum::{
    http::{StatusCode, Uri},
    routing::get,
    Extension, Router,
};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

mod auth;
mod error;
mod fapi;
mod spa;
pub mod state;

pub fn router(state: AppState) -> shuttle_axum::AxumService {
    let router = Router::new()
        .route("/", get(spa::welcome_handler))
        .nest("/auth", auth::router())
        .nest("/fapi", fapi::router())
        .route("/*path", get(spa::spa_handler))
        .layer(Extension(spa::static_services()))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(HandleErrorLayer::new(error::handle))
                .timeout(Duration::from_secs(10)),
        )
        .fallback(fallback)
        .with_state(state);

    router.into()
}

#[allow(clippy::unused_async)]
async fn fallback(uri: Uri) -> (StatusCode, String) {
    let message = format!("I couldn't find '{}'. Try something else?", uri.path());
    (StatusCode::NOT_FOUND, message)
}
