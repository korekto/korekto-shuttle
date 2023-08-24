use crate::router::state::AppState;
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    response::{IntoResponse, Redirect, Response},
    RequestPartsExt,
};
use axum_extra::extract::PrivateCookieJar;

const SESSION_ID_COOKIE: &str = "session_id";

#[derive(Debug)]
pub struct AuthenticatedUser(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let user = extract_user_from_cookie(parts, &app_state).await?;

        Ok(Self(user))
    }
}

async fn extract_user_from_cookie(
    parts: &mut Parts,
    app_state: &AppState,
) -> Result<String, AuthRejection> {
    let cookies = parts
        .extract_with_state::<PrivateCookieJar, AppState>(app_state)
        .await
        .expect("could not fail, waiting for into_ok() stabilization");

    let cookie = cookies
        .get(SESSION_ID_COOKIE)
        .ok_or(AuthRejection::AuthRedirect)?;

    let username = cookie
        .value()
        .parse::<String>()
        .map_err(|_| AuthRejection::AuthRedirect)?;

    Ok(username)
}

pub enum AuthRejection {
    AuthRedirect,
}

impl IntoResponse for AuthRejection {
    fn into_response(self) -> Response {
        match self {
            Self::AuthRedirect => Redirect::temporary("/").into_response(),
        }
    }
}
