use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    response::{IntoResponse, Redirect, Response},
    routing::get,
    RequestPartsExt, Router,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::PrivateCookieJar;
use time::Duration;

use crate::router::state::AppState;

mod github;

const SESSION_ID_COOKIE: &str = "session_id";
const SESSION_ID_COOKIE_DURATION: Duration = Duration::days(1);

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/gh/start", get(github::gh_login_start))
        .route("/gh/authorized", get(github::gh_login_authorized))
}

#[derive(Debug)]
pub struct AuthenticatedUser(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthenticationRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let user = extract_user_from_cookie(parts, &app_state).await?;
        drop(app_state);

        Ok(Self(user))
    }
}

async fn extract_user_from_cookie(
    parts: &mut Parts,
    app_state: &AppState,
) -> Result<String, AuthenticationRejection> {
    #[allow(clippy::expect_used)]
    let cookies = parts
        .extract_with_state::<PrivateCookieJar, AppState>(app_state)
        .await
        .expect("could not fail, waiting for into_ok() stabilization");

    let cookie = cookies
        .get(SESSION_ID_COOKIE)
        .ok_or(AuthenticationRejection::AuthRedirect)?;

    let username = cookie
        .value()
        .parse::<String>()
        .map_err(|_| AuthenticationRejection::AuthRedirect)?;

    Ok(username)
}

pub enum AuthenticationRejection {
    AuthRedirect,
}

impl IntoResponse for AuthenticationRejection {
    fn into_response(self) -> Response {
        match self {
            Self::AuthRedirect => Redirect::temporary("/").into_response(),
        }
    }
}

fn set_session_id_cookie(jar: PrivateCookieJar, user_id: &str) -> PrivateCookieJar {
    jar.add(session_cookie(user_id))
}

fn session_cookie<'a>(user_id: &str) -> Cookie<'a> {
    Cookie::build(SESSION_ID_COOKIE, String::from(user_id))
        .max_age(SESSION_ID_COOKIE_DURATION)
        .same_site(SameSite::Lax)
        .path("/")
        .finish()
}
