use std::path::Path;

use axum::response::Redirect;
use axum::{
    http::{Request, Response, StatusCode},
    response::IntoResponse,
    Extension,
};
use tower_http::{
    services::{fs::ServeFileSystemResponseBody, ServeDir, ServeFile},
    set_status::SetStatus,
};
use tower_service::Service;
use tracing::info;

use crate::router::auth::AuthenticatedUser;

#[derive(Clone)]
pub struct StaticServices {
    welcome: ServeFile,
    spa: ServeDir<SetStatus<ServeFile>>,
}

pub fn static_services(static_folder: &Path) -> StaticServices {
    StaticServices {
        welcome: welcome_service(static_folder),
        spa: spa_service(static_folder),
    }
}

fn spa_service(static_folder: &Path) -> ServeDir<SetStatus<ServeFile>> {
    let dashboard_path = static_folder.join("dashboard.html");

    let serve_dir = ServeDir::new(&static_folder).fallback(SetStatus::new(
        ServeFile::new(&dashboard_path),
        StatusCode::OK,
    ));
    serve_dir
}

fn welcome_service(static_folder: &Path) -> ServeFile {
    ServeFile::new(static_folder.join("welcome.html"))
}

pub async fn spa_handler<ReqBody>(
    user: Option<AuthenticatedUser>,
    mut static_services: Extension<StaticServices>,
    req: Request<ReqBody>,
) -> Result<Response<ServeFileSystemResponseBody>, Redirect>
where
    ReqBody: 'static + Send,
{
    if let Some(user) = user {
        Ok(static_services.0.spa.call(req).await.unwrap())
    } else {
        Err(Redirect::temporary("/"))
    }
}

pub async fn welcome_handler<ReqBody>(
    user: Option<AuthenticatedUser>,
    mut static_services: Extension<StaticServices>,
    req: Request<ReqBody>,
) -> Result<Response<ServeFileSystemResponseBody>, Redirect>
where
    ReqBody: 'static + Send,
{
    if let Some(user) = user {
        Err(Redirect::temporary("/dashboard"))
    } else {
        Ok(static_services.0.welcome.call(req).await.unwrap())
    }
}
