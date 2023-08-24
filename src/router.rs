use std::path::PathBuf;

use axum::{
    http::{StatusCode, Uri},
    routing::get,
    Extension, Router,
};

mod spa;

pub fn router(static_folder: PathBuf) -> shuttle_axum::AxumService {
    let spa_router = get(spa::spa_handler);

    let router = Router::new()
        .route("/", spa_router.clone())
        .route("/*path", spa_router)
        .layer(Extension(spa::spa_service(&static_folder)))
        .fallback(fallback);

    router.into()
}

async fn fallback(uri: Uri) -> (StatusCode, String) {
    let message = format!("I couldn't find '{}'. Try something else?", uri.path());
    (StatusCode::NOT_FOUND, message)
}
