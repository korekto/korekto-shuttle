use axum::extract::OriginalUri;
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    RequestPartsExt, Router,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar, PrivateCookieJar,
};
use http::uri::PathAndQuery;
use http::StatusCode;
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
        .route("/gh/post_install", get(github::gh_post_install))
        .route("/logout", post(logout))
}

#[allow(clippy::unused_async)]
pub async fn logout(jar: CookieJar) -> (CookieJar, Redirect) {
    // Because there is no shorthand Redirect::found for now
    (remove_session_id_cookie(jar), Redirect::to("/"))
}

pub struct AuthenticatedUser(pub User);

pub struct AdminUser(pub User);

pub struct TeacherUser(pub User);

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

#[async_trait]
impl<S> FromRequestParts<S> for AdminUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthenticationRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let AuthenticatedUser(user) = AuthenticatedUser::from_request_parts(parts, state).await?;

        if user.admin {
            Ok(Self(user))
        } else {
            Err(AuthenticationRejection::NeedsAppropriateRight)
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for TeacherUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthenticationRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let AuthenticatedUser(user) = AuthenticatedUser::from_request_parts(parts, state).await?;

        if user.teacher {
            Ok(Self(user))
        } else {
            Err(AuthenticationRejection::NeedsAppropriateRight)
        }
    }
}

async fn extract_user_from_cookie(
    parts: &mut Parts,
    app_state: &AppState,
) -> Result<User, AuthenticationRejection> {
    let query = parts
        .extensions
        .get::<OriginalUri>()
        .and_then(|ou| ou.0.path_and_query())
        .cloned();

    #[allow(clippy::expect_used)]
    let cookies = parts
        .extract_with_state::<PrivateCookieJar, AppState>(app_state)
        .await
        .expect("could not fail, waiting for into_ok() stabilization");

    let cookie = cookies
        .get(SESSION_ID_COOKIE)
        .ok_or(AuthenticationRejection::AuthRedirect(query.clone()))?;

    let user_id = cookie
        .value()
        .parse::<i32>()
        .map_err(|_| AuthenticationRejection::AuthRedirect(query.clone()))?;

    let user = app_state
        .service
        .find_user_by_id(&user_id)
        .await
        .ok_or_else(|| {
            warn!("User with valid cookie, but not found in Database");
            AuthenticationRejection::AuthRedirect(query.clone())
        })?;

    Ok(user)
}

pub enum AuthenticationRejection {
    AuthRedirect(Option<PathAndQuery>),
    NeedsAppropriateRight,
}

impl IntoResponse for AuthenticationRejection {
    fn into_response(self) -> Response {
        match self {
            Self::AuthRedirect(Some(path_and_query)) => (
                CookieJar::new().add(
                    Cookie::build(("auth_origin", path_and_query.to_string()))
                        .path("/")
                        .same_site(SameSite::Lax)
                        .max_age(Duration::minutes(5)),
                ),
                Redirect::to("/"),
            )
                .into_response(),
            Self::AuthRedirect(None) => Redirect::temporary("/").into_response(),
            Self::NeedsAppropriateRight => StatusCode::FORBIDDEN.into_response(),
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
    Cookie::build((SESSION_ID_COOKIE, value))
        .max_age(SESSION_ID_COOKIE_DURATION)
        .same_site(SameSite::Lax)
        .path("/")
        .build()
}
