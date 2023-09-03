use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    RequestPartsExt,
    response::{IntoResponse, Redirect, Response},
    Router, routing::{get, post},
};
use axum_extra::extract::{CookieJar, PrivateCookieJar, cookie::{Cookie, SameSite}};
use http::{header::LOCATION, HeaderValue, StatusCode};
use time::Duration;
use tracing::warn;

use crate::{entities::User, router::state::AppState};

mod github;

const SESSION_ID_COOKIE: &str = "session_id";
const SESSION_ID_COOKIE_DURATION: Duration = Duration::days(1);

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/gh/start", get(github::gh_login_start))
        .route("/gh/authorized", get(github::gh_login_authorized))
        .route("/logout", post(logout))
}

pub async fn logout(jar: CookieJar) -> (CookieJar, Response) {
    // Because there is no shorthand Redirect::found for now
    (remove_session_id_cookie(jar), (StatusCode::FOUND, [(LOCATION, HeaderValue::from_static("/"))]).into_response())
}

#[derive(Debug)]
pub struct AuthenticatedUser(pub User);

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

        // TODO remove this lock after beta
        if user.provider_login.ne("ledoyen") {
            Err(AuthenticationRejection::AuthRedirect)
        } else {
            Ok(Self(user))
        }
    }
}

async fn extract_user_from_cookie(
    parts: &mut Parts,
    app_state: &AppState,
) -> Result<User, AuthenticationRejection> {
    #[allow(clippy::expect_used)]
        let cookies = parts
        .extract_with_state::<PrivateCookieJar, AppState>(app_state)
        .await
        .expect("could not fail, waiting for into_ok() stabilization");

    let cookie = cookies
        .get(SESSION_ID_COOKIE)
        .ok_or(AuthenticationRejection::AuthRedirect)?;

    let user_id = cookie
        .value()
        .parse::<i32>()
        .map_err(|_| AuthenticationRejection::AuthRedirect)?;

    let user = app_state
        .service
        .find_user_by_id(&user_id)
        .await
        .ok_or_else(|| {
            warn!("User with valid cookie, but not found in Database");
            AuthenticationRejection::AuthRedirect
        })?;

    Ok(user)
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

fn set_session_id_cookie(jar: PrivateCookieJar, user_id: i32) -> PrivateCookieJar {
    jar.add(session_cookie(user_id.to_string()))
}

pub fn remove_session_id_cookie(jar: CookieJar) -> CookieJar {
    // Otherwise the browser might keep the previous value if the cookie is conventionally deleted
    jar.add(session_cookie(String::from("deleted")))
}

fn session_cookie<'a>(value: String) -> Cookie<'a> {
    Cookie::build(SESSION_ID_COOKIE, value)
        .max_age(SESSION_ID_COOKIE_DURATION)
        .same_site(SameSite::Lax)
        .path("/")
        .finish()
}
