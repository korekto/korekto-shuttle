use crate::router::auth::set_session_id_cookie;
use crate::router::state::AppState;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::{extract::State, response::Redirect};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    PrivateCookieJar,
};
use oauth2::{reqwest::async_http_client, AuthorizationCode, CsrfToken, Scope};
use time::Duration;

const GH_STATE_COOKIE: &str = "gh_state";
const GH_STATE_COOKIE_DURATION: Duration = Duration::minutes(10);

#[derive(Debug, serde::Deserialize)]
pub struct AuthRequest {
    code: String,
    state: String,
}

pub async fn gh_login_start(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
) -> (PrivateCookieJar, Redirect) {
    let (authorize_url, csrf_state) = &state
        .oauth
        .gh_client
        .set_redirect_uri(state.oauth.redirect_url)
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("public_repo".to_string()))
        .add_scope(Scope::new("user:email".to_string()))
        .url();

    (
        jar.add(
            Cookie::build(GH_STATE_COOKIE, csrf_state.secret().clone())
                .max_age(GH_STATE_COOKIE_DURATION)
                .same_site(SameSite::Lax)
                .finish(),
        ),
        Redirect::to(authorize_url.as_ref()),
    )
}

pub async fn gh_login_authorized(
    Query(query): Query<AuthRequest>,
    State(state): State<AppState>,
    mut jar: PrivateCookieJar,
) -> (PrivateCookieJar, Result<Redirect, StatusCode>) {
    let state_check = check_state(&query, jar);
    jar = state_check.0;
    if let Err(status_code) = state_check.1 {
        return (jar, Err(status_code));
    }

    let token_res = state
        .oauth
        .gh_client
        .exchange_code(AuthorizationCode::new(query.code.clone()))
        .request_async(async_http_client)
        .await;

    match token_res {
        Ok(token) => {
            let new_user_result = state.github_clients.get_user_info(&token).await;

            match new_user_result {
                Ok(new_user) => {
                    jar = set_session_id_cookie(jar, &new_user.login);
                    (jar, Ok(Redirect::to("/dashboard")))
                }
                Err(error) => {
                    tracing::info!("Error while retrieving GH user infos: {:?}", error);
                    (jar, Err(StatusCode::FORBIDDEN))
                }
            }
        }
        Err(error) => {
            tracing::info!("Error while retrieving GH tokens: {:?}", error);
            (jar, Err(StatusCode::FORBIDDEN))
        }
    }
}

fn check_state(
    query: &AuthRequest,
    jar: PrivateCookieJar,
) -> (PrivateCookieJar, Result<(), StatusCode>) {
    let state_token = CsrfToken::new(query.state.clone());
    let stored_secret: Option<String> = jar
        .get(GH_STATE_COOKIE)
        .map(|cookie| cookie.value().to_owned());

    let jar = jar.remove(Cookie::named(GH_STATE_COOKIE));

    if stored_secret.is_none() || stored_secret.as_ref().unwrap().ne(state_token.secret()) {
        tracing::info!(
            "Invalid state, expected:{:?}, got:{}",
            stored_secret,
            state_token.secret()
        );
        (jar, Err(StatusCode::FORBIDDEN))
    } else {
        (jar, Ok(()))
    }
}
