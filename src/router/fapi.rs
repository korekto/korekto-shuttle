use crate::entities::UserProfileUpdate;
use axum::extract::State;
use axum::{
    routing::{get, patch},
    Json, Router,
};
use http::StatusCode;
use tracing::{error, warn};

use crate::router::auth::AuthenticatedUser;
use crate::router::state::AppState;

mod admin;
mod teacher;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/user/self", get(get_self).put(update_self))
        .route("/settings/redeem_code", patch(redeem_code))
        .nest("/admin", admin::router())
        .nest("/teacher", teacher::router())
        .fallback(crate::router::fallback)
}

#[allow(clippy::unused_async)]
async fn get_self(AuthenticatedUser(user): AuthenticatedUser) -> Json<User> {
    Json(user.into())
}

#[allow(clippy::unused_async)]
async fn update_self(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Json(update): Json<UserProfileUpdate>,
) -> Result<Json<User>, StatusCode> {
    let updated_user = state
        .service
        .repo
        .update_user_profile(&user.id, &update)
        .await
        .map_err(|err| {
            error!("{err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(updated_user.into()))
}

async fn redeem_code(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    payload: String,
) -> Result<(), StatusCode> {
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    if state.instance_secret.eq(&payload) {
        state
            .service
            .repo
            .set_user_admin(&user.0.id)
            .await
            .map_err(|err| {
                error!("{err}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        Ok(())
    } else {
        warn!(
            "Provided code was: {}, but expected: {}",
            &payload, state.instance_secret
        );
        Err(StatusCode::FORBIDDEN)
    }
}

#[derive(Debug, serde::Serialize)]
struct User {
    firstname: String,
    lastname: String,
    school_group: String,
    school_email: String,
    role: String,
    avatar_url: String,
    installation_id: Option<String>,
    admin: bool,
    teacher: bool,
}

impl From<crate::entities::User> for User {
    fn from(user: crate::entities::User) -> Self {
        let mut role = if user.admin { "Admin" } else { "Student" }.to_string();
        if user.teacher {
            role.push_str(" & Teacher");
        }
        Self {
            firstname: user.first_name,
            lastname: user.last_name,
            school_group: user.school_group,
            school_email: user.school_email,
            role,
            avatar_url: user.avatar_url,
            installation_id: user.installation_id,
            admin: user.admin,
            teacher: user.teacher,
        }
    }
}
