use axum::{
    extract,
    extract::State,
    routing::{get, patch},
    Router,
};
use http::StatusCode;
use time::format_description::well_known::Iso8601;
use tracing::error;

use crate::{
    entities,
    entities::Table,
    router::{auth::AdminUser, state::AppState},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/table", get(get_tables))
        .route("/user", get(get_users).delete(delete_users))
        .route("/teacher", patch(set_users_teacher))
}

async fn get_tables(
    _user: AdminUser,
    State(state): State<AppState>,
) -> Result<axum::Json<Vec<Table>>, StatusCode> {
    let tables = state
        .service
        .repo
        .find_tables()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(axum::Json(tables))
}

async fn get_users(
    _user: AdminUser,
    State(state): State<AppState>,
) -> Result<axum::Json<Vec<User>>, StatusCode> {
    let users = state
        .service
        .repo
        .find_users()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(User::try_from)
        .filter_map(|user_res| match user_res {
            Ok(user) => Some(user),
            Err(err) => {
                error!("Unable to map User: {err}");
                None
            }
        })
        .collect();
    Ok(axum::Json(users))
}

async fn delete_users(
    _user: AdminUser,
    State(state): State<AppState>,
    extract::Json(user_ids): extract::Json<Vec<i32>>,
) -> Result<(), StatusCode> {
    state
        .service
        .repo
        .delete_users_by_id(&user_ids)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(())
}

async fn set_users_teacher(
    _user: AdminUser,
    State(state): State<AppState>,
    extract::Json(user_ids): extract::Json<Vec<i32>>,
) -> Result<(), StatusCode> {
    state
        .service
        .repo
        .set_users_teacher(&user_ids)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(())
}

#[derive(serde::Serialize, Clone)]
pub struct User {
    pub id: i32,
    pub provider_login: String,
    pub name: String,
    pub email: String,
    pub accessible_repos: u8,
    pub teacher: bool,
    pub admin: bool,
    pub installation_id: String,
    pub created_at: String,
}

impl TryFrom<entities::User> for User {
    type Error = anyhow::Error;

    fn try_from(user: entities::User) -> Result<Self, Self::Error> {
        Ok(Self {
            id: user.id,
            provider_login: user.provider_login,
            name: user.name,
            email: user.email,
            accessible_repos: 0,
            teacher: user.teacher,
            admin: user.admin,
            installation_id: user.installation_id.unwrap_or_default(),
            created_at: user.created_at.format(&Iso8601::DEFAULT)?,
        })
    }
}
