use std::path::Path;

use axum::{
    http::{Request, Response, StatusCode},
    Extension,
};
use tower_http::{
    services::{fs::ServeFileSystemResponseBody, ServeDir, ServeFile},
    set_status::SetStatus,
};
use tower_service::Service;

pub fn spa_service(static_folder: &Path) -> ServeDir<SetStatus<ServeFile>> {
    let dashboard_path = static_folder.join("dashboard.html");

    let serve_dir = ServeDir::new(&static_folder).fallback(SetStatus::new(
        ServeFile::new(&dashboard_path),
        StatusCode::OK,
    ));
    serve_dir
}

pub async fn spa_handler<ReqBody>(
    mut serve_dir: Extension<ServeDir<SetStatus<ServeFile>>>,
    req: Request<ReqBody>,
) -> Response<ServeFileSystemResponseBody>
where
    ReqBody: 'static + Send,
{
    serve_dir.0.call(req).await.unwrap()
}
