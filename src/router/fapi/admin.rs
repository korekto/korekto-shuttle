use axum::extract::Query;
use axum::{
    extract::State,
    routing::{delete, get, patch},
    Json, Router,
};
use http::StatusCode;
use tracing::{error, warn};
use validator::Validate;

use crate::github::runner;
use crate::service::dtos::{
    GradingTaskResponse, Page, PaginationQuery, UnparseableWebhookResponse, UserForAdminResponse,
};
use crate::{
    entities::Table,
    router::{auth::AdminUser, state::AppState},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_metadata))
        .route("/db/table", get(get_tables).delete(drop_table))
        .route("/db/migrations", delete(rerun_only_migrations))
        .route("/db", delete(recreate_db))
        .route("/user", get(get_users).delete(delete_users))
        .route("/teacher", patch(set_users_teacher))
        .route(
            "/unparseable_webhooks",
            get(get_unparseable_webhooks).delete(delete_unparseable_webhooks),
        )
        .route("/grading_tasks", get(get_grading_tasks))
}

async fn get_metadata(
    _user: AdminUser,
    State(state): State<AppState>,
) -> Result<Json<AdminMetadata>, (StatusCode, String)> {
    let runner = state.gh_runner.metadata().await.map_err(|err| {
        warn!("{err:?}");
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{err}"))
    })?;

    Ok(Json(AdminMetadata { runner }))
}

#[derive(serde::Serialize, Debug, Clone)]
struct AdminMetadata {
    runner: runner::Metadata,
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

async fn rerun_only_migrations(
    _user: AdminUser,
    State(state): State<AppState>,
) -> Result<(), (StatusCode, Json<String>)> {
    rerun_migrations(false, &state)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, Json(format!("{err}"))))
}

async fn drop_table(
    _user: AdminUser,
    State(state): State<AppState>,
    table_name: String,
) -> Result<(), (StatusCode, Json<String>)> {
    state
        .service
        .repo
        .drop_table(&table_name)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, Json(format!("{err}"))))
}

async fn recreate_db(
    _user: AdminUser,
    State(state): State<AppState>,
) -> Result<(), (StatusCode, Json<String>)> {
    rerun_migrations(true, &state)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, Json(format!("{err}"))))
}

async fn rerun_migrations(wipe_db: bool, state: &AppState) -> anyhow::Result<()> {
    if wipe_db {
        state.service.repo.wipe_database().await?;
    }
    state.service.repo.reset_migrations().await?;
    state.service.repo.run_migrations().await?;
    Ok(())
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
