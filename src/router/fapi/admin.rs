use axum::extract::Query;
use axum::{
    extract::State,
    routing::{get, patch},
    Json, Router,
};
use http::StatusCode;
use tracing::error;
use validator::Validate;

use crate::service::dtos::{
    GradingTaskResponse, Page, PaginationQuery, UnparseableWebhookResponse, UserForAdminResponse,
};
use crate::{
    entities::Table,
    router::{auth::AdminUser, state::AppState},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/table", get(get_tables))
        .route("/user", get(get_users).delete(delete_users))
        .route("/teacher", patch(set_users_teacher))
        .route(
            "/unparseable_webhooks",
            get(get_unparseable_webhooks).delete(delete_unparseable_webhooks),
        )
        .route("/grading_tasks", get(get_grading_tasks))
}

async fn get_tables(
    _user: AdminUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<Table>>, StatusCode> {
    let tables = state
        .service
        .repo
        .find_tables()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(tables))
}

async fn get_users(
    _user: AdminUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<UserForAdminResponse>>, StatusCode> {
    let users = state
        .service
        .repo
        .find_users()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(UserForAdminResponse::try_from)
        .filter_map(|user_res| match user_res {
            Ok(user) => Some(user),
            Err(err) => {
                error!("Unable to map User: {err}");
                None
            }
        })
        .collect();
    Ok(Json(users))
}

async fn delete_users(
    _user: AdminUser,
    State(state): State<AppState>,
    Json(user_ids): Json<Vec<i32>>,
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
    Json(user_ids): Json<Vec<i32>>,
) -> Result<(), StatusCode> {
    state
        .service
        .repo
        .set_users_teacher(&user_ids)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(())
}

async fn get_unparseable_webhooks(
    _user: AdminUser,
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<Page<UnparseableWebhookResponse>>, (StatusCode, Json<String>)> {
    pagination
        .validate()
        .map_err(|err| (StatusCode::BAD_REQUEST, Json(format!("{err}"))))?;
    Ok(Json(
        state
            .service
            .get_unparseable_webhooks(&pagination)
            .await
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, Json(format!("{err}"))))?,
    ))
}

async fn delete_unparseable_webhooks(
    _user: AdminUser,
    State(state): State<AppState>,
) -> Result<(), (StatusCode, Json<String>)> {
    state
        .service
        .repo
        .delete_unparseable_webhooks()
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, Json(format!("{err}"))))?;
    Ok(())
}

async fn get_grading_tasks(
    _user: AdminUser,
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<Page<GradingTaskResponse>>, (StatusCode, Json<String>)> {
    pagination
        .validate()
        .map_err(|err| (StatusCode::BAD_REQUEST, Json(format!("{err}"))))?;
    Ok(Json(
        state
            .service
            .get_grading_tasks(&pagination)
            .await
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, Json(format!("{err}"))))?,
    ))
}
