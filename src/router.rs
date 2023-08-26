use std::path::PathBuf;

use crate::config::Config;
use crate::router::state::AppState;
use axum::{
    http::{StatusCode, Uri},
    routing::get,
    Extension, Router,
};

mod auth;
mod spa;
mod state;

pub fn router(static_folder: PathBuf, config: &Config) -> shuttle_axum::AxumService {
    let app_state = AppState::new(config);

    let router = Router::new()
        .route("/", get(spa::welcome_handler))
        .nest("/auth", auth::router())
        .route("/*path", get(spa::spa_handler))
        .layer(Extension(spa::static_services(&static_folder)))
        .fallback(fallback)
        .with_state(app_state);

    router.into()
}

async fn fallback(uri: Uri) -> (StatusCode, String) {
    let message = format!("I couldn't find '{}'. Try something else?", uri.path());
    (StatusCode::NOT_FOUND, message)
}
