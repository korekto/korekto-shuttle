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
use tracing::error;

mod auth;
mod error;
mod fapi;
mod spa;
pub mod state;
mod webhook;

pub fn router(state: AppState) -> Router {
    let mut router = Router::new()
        .route("/", get(spa::welcome_handler))
        .route("/error", get(error))
        .route("/panic", get(panic))
        .nest("/auth", auth::router())
        .nest("/fapi", fapi::router())
        .nest("/webhook", webhook::router())
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
    router = crate::sentry::configure_router(router);
    router
}

async fn error() {
    error!("test error!");
}

async fn panic() {
    panic!("Everything is on fire!");
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

        impl headers::Header for $type {
            fn name() -> &'static headers::HeaderName {
                &$upcase
            }

            fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
            where
                I: Iterator<Item = &'i headers::HeaderValue>,
            {
                let value = values.next().ok_or_else(headers::Error::invalid)?;

                if let Ok(str_value) = value.to_str() {
                    Ok(Self(str_value.to_string()))
                } else {
                    Err(headers::Error::invalid())
                }
            }

            fn encode<E>(&self, values: &mut E)
            where
                E: Extend<headers::HeaderValue>,
            {
                let value = headers::HeaderValue::from_str(&self.0).unwrap();
                values.extend(std::iter::once(value));
            }
        }
    };
}
