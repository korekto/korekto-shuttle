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
mod webhook;

pub fn router(static_folder: &Path, config: &Config) -> anyhow::Result<shuttle_axum::AxumService> {
    let router = Router::new()
        .route("/", get(spa::welcome_handler))
        .nest("/auth", auth::router())
        .nest("/fapi", fapi::router())
        .nest("/webhook", webhook::router())
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

#[macro_export]
macro_rules! string_header {
    ($type:ident, $upcase:ident, $name:literal) => {
        static $upcase: headers::HeaderName = headers::HeaderName::from_static($name);

        pub struct $type(String);

        #[axum::async_trait]
        impl<S> axum::extract::FromRequestParts<S> for $type
        where
            S: Send + Sync,
        {
            type Rejection = (axum::http::StatusCode, String);

            async fn from_request_parts(
                parts: &mut http::request::Parts,
                _state: &S,
            ) -> Result<Self, Self::Rejection> {
                if let Some(header_value) = parts.headers.get(&$upcase) {
                    Ok($type(
                        header_value
                            .to_str()
                            .ok()
                            .ok_or((
                                axum::http::StatusCode::BAD_REQUEST,
                                format!("`{}` header is missing", $name),
                            ))?
                            .parse()
                            .unwrap(),
                    ))
                } else {
                    Err((
                        axum::http::StatusCode::BAD_REQUEST,
                        format!("`{}` header is missing", $name),
                    ))
                }
            }
        }
    };
}
