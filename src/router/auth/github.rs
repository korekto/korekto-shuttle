use anyhow::anyhow;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::{extract::State, response::Redirect};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    PrivateCookieJar,
};
use oauth2::{basic::BasicTokenResponse, TokenResponse};
use oauth2::{reqwest::async_http_client, AuthorizationCode, CsrfToken, Scope};
use sqlx::types::Json;
use time::{Duration, OffsetDateTime, PrimitiveDateTime};
use tracing::error;

use crate::entities::{GitHubUserTokens, NewUser, Token};
use crate::github::GitHubUserLogged;
use crate::router::auth::{set_session_id_cookie, AuthenticatedUser};
use crate::router::state::AppState;

const GH_STATE_COOKIE: &str = "gh_state";
const GH_STATE_COOKIE_DURATION: Duration = Duration::minutes(10);

#[derive(Debug, serde::Deserialize)]
pub struct AuthRequest {
    code: String,
    state: String,
}

#[allow(clippy::unused_async)]
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
            Cookie::build((GH_STATE_COOKIE, csrf_state.secret().clone()))
                .path("/")
                .max_age(GH_STATE_COOKIE_DURATION)
                .same_site(SameSite::Lax),
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
    if state_check.1.is_err() {
        return (jar, Ok(Redirect::to("/")));
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
                Ok(user_logged) => {
                    let user_flow = decide_user_flow(&token, &user_logged, &state).await;
                    match user_flow {
                        Err(err) => {
                            error!("Unexpected error: {err})");
                            // TODO maybe some 500 page with info
                            (jar, Ok(Redirect::to("/")))
                        }
                        Ok((user_id, redirect)) => {
                            jar = set_session_id_cookie(jar, user_id);
                            (jar, Ok(redirect))
                        }
                    }
                }
                Err(error) => {
                    tracing::warn!("Error while retrieving GH user infos: {:?}", error);
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

pub async fn gh_post_install(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Redirect {
    let installation_id = state.github_clients.get_user_installation_id(&user).await;

    if let Some(installation_id) = installation_id {
        let result = state
            .service
            .repo
            .update_installation_id(&user.id, &installation_id)
            .await;
        if let Err(err) = result {
            error!(
                "Failed to store installation_id {} for user {}: {err}",
                &installation_id, &user.provider_login
            );
        }
    }

    Redirect::to("/dashboard")
}

async fn decide_user_flow(
    token: &BasicTokenResponse,
    user_logged: &GitHubUserLogged,
    state: &AppState,
) -> anyhow::Result<(i32, Redirect)> {
    let user = state
        .service
        .repo
        .upsert_user(&(token, user_logged).try_into()?)
        .await?;
    if let Some(first_admin) = &state.config.first_admin {
        if first_admin == &user.provider_login {
            state.service.repo.set_user_admin(user.id).await?;
        }
    }
    if user.installation_id.is_none() {
        let installation_url = format!(
            "https://github.com/apps/{}/installations/new",
            &state.config.github_app_name
        );
        Ok((user.id, Redirect::to(&installation_url)))
    } else {
        Ok((user.id, Redirect::to("/dashboard")))
    }
}

fn check_state(query: &AuthRequest, jar: PrivateCookieJar) -> (PrivateCookieJar, Result<(), ()>) {
    let state_token = CsrfToken::new(query.state.clone());
    let stored_secret: Option<String> = jar
        .get(GH_STATE_COOKIE)
        .map(|cookie| cookie.value().to_owned());

    let jar = jar.remove(Cookie::from(GH_STATE_COOKIE));

    if stored_secret
        .as_ref()
        .is_some_and(|ss| ss.ne(state_token.secret()))
    {
        tracing::warn!(
            "Invalid state, expected:{:?}, got:{}",
            stored_secret,
            state_token.secret()
        );
        (jar, Err(()))
    } else {
        if stored_secret.is_none() {
            tracing::warn!(
                "Missing state from cookies, not able to confirm the one sent by GitHub"
            );
        }
        (jar, Ok(()))
    }
}

impl TryFrom<(&BasicTokenResponse, &GitHubUserLogged)> for NewUser {
    type Error = anyhow::Error;
    fn try_from(
        (token, user): (&BasicTokenResponse, &GitHubUserLogged),
    ) -> Result<Self, Self::Error> {
        const fn to_primitive(date_time: OffsetDateTime) -> PrimitiveDateTime {
            PrimitiveDateTime::new(date_time.date(), date_time.time())
        }

        let access_token_expiration = OffsetDateTime::now_utc()
            + Duration::try_from(
                token
                    .expires_in()
                    .ok_or_else(|| anyhow!("Missing expiration from access token"))?,
            )?;

        let refresh_token_expiration = OffsetDateTime::now_utc() + Duration::days(30 * 4);

        Ok(Self {
            provider_name: user.name.clone().unwrap_or_else(|| user.login.clone()),
            provider_login: user.login.clone(),
            provider_email: user.email.clone().unwrap_or_default(),
            avatar_url: user.avatar_url.clone(),
            github_user_tokens: Some(Json(GitHubUserTokens {
                access_token: Token {
                    value: token.access_token().secret().to_string(),
                    expiration_date: to_primitive(access_token_expiration),
                },
                refresh_token: Token {
                    value: token
                        .refresh_token()
                        .ok_or_else(|| anyhow!("Missing refresh token for user {}", &user.login))?
                        .secret()
                        .to_string(),
                    expiration_date: to_primitive(refresh_token_expiration),
                },
            })),
        })
    }
}
