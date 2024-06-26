use std::path::{Path, PathBuf};

use axum::{
    http::{Request, Response, StatusCode},
    response::Redirect,
    Extension,
};
use axum_extra::TypedHeader;
use headers::IfModifiedSince;
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

pub fn static_services() -> StaticServices {
    let static_folder: PathBuf = PathBuf::from("static");
    StaticServices {
        welcome: welcome_service(&static_folder),
        spa: spa_service(&static_folder),
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
    if_modified_since: Option<TypedHeader<IfModifiedSince>>,
    mut static_services: Extension<StaticServices>,
    req: Request<ReqBody>,
) -> Result<Result<Response<ServeFileSystemResponseBody>, StatusCode>, Redirect>
where
    ReqBody: 'static + Send,
{
    if let Some(_user) = user {
        // This is a hack as for some reason, when the header If-Modified-Since is set (by the browser)
        // the ServeDir service returns Ok (200) with an empty body, instead of Not Modified (304)
        // leading to a blank page
        if if_modified_since.is_some() {
            Ok(Err(StatusCode::NOT_MODIFIED))
        } else {
            Ok(Ok(static_services.0.spa.call(req).await.unwrap()))
        }
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
        Ok(static_services.0.welcome.call(req).await.unwrap())
    }
}
