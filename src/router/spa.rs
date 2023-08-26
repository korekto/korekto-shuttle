use std::path::Path;

use axum::response::Redirect;
use axum::{
    http::{Request, Response, StatusCode},
    Extension,
};
use tower_http::{
    services::{fs::ServeFileSystemResponseBody, ServeDir, ServeFile},
    set_status::SetStatus,
};
use tower_service::Service;

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
    let index_path = static_folder.join("index.html");

    ServeDir::new(static_folder)
        .fallback(SetStatus::new(ServeFile::new(index_path), StatusCode::OK))
}

fn welcome_service(static_folder: &Path) -> ServeFile {
    ServeFile::new(static_folder.join("welcome.html"))
}

#[allow(clippy::module_name_repetitions)]
pub async fn spa_handler<ReqBody>(
    user: Option<AuthenticatedUser>,
    mut static_services: Extension<StaticServices>,
    req: Request<ReqBody>,
) -> Result<Response<ServeFileSystemResponseBody>, Redirect>
where
    ReqBody: 'static + Send,
{
    if let Some(_user) = user {
        Ok(static_services
            .0
            .spa
            .call(req)
            .await
            .map_err(|_| Redirect::temporary("/error"))?)
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
    if let Some(_user) = user {
        Err(Redirect::temporary("/dashboard"))
    } else {
        static_services
            .0
            .welcome
            .call(req)
            .await
            .map_err(|_| Redirect::temporary("/error"))
    }
}
