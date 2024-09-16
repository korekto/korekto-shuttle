use crate::entities::NewGradingTask;
use axum::extract::{Path, Query};
use axum::response::Redirect;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use axum_extra::either::Either;
use http::StatusCode;
use time::OffsetDateTime;
use tracing::{error, info, warn};

use crate::router::auth::AuthenticatedUser;
use crate::router::state::AppState;
use crate::service::dtos::{
    UserAssignmentResponse, UserModuleDescResponse, UserModuleResponse, VecInto,
};
use crate::service::{ObfuscatedStr, SyncError};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_modules))
        .route("/redeem", get(redeem_module))
        .route("/:module_id", get(get_module))
        .route("/:module_id/assignment/:assignment_id", get(get_assignment))
        .route(
            "/:module_id/assignment/:assignment_id/trigger-grading",
            post(trigger_grading),
        )
        .route(
            "/:module_id/assignment/:assignment_id/sync-repo",
            post(sync_repo),
        )
}

async fn get_assignment(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Path((module_id, assignment_id)): Path<(String, String)>,
) -> Result<Json<UserAssignmentResponse>, StatusCode> {
    let assignment = state
        .service
        .get_assignment(
            &user,
            &module_id,
            &assignment_id,
            state.config.min_grading_interval_in_secs,
        )
        .await
        .map_err(|err| {
            error!(error = ?err, %user, module_id, ?assignment_id, "[http] get_assignment");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(assignment))
}

async fn get_module(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Path(module_id): Path<String>,
) -> Result<Json<UserModuleResponse>, StatusCode> {
    let module = state
        .service
        .repo
        .get_module(&user, &module_id)
        .await
        .map_err(|err| {
            error!(error = ?err, %user, module_id, "[http] get_module");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(module.into()))
}

async fn list_modules(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<UserModuleDescResponse>>, StatusCode> {
    let modules = state
        .service
        .repo
        .list_modules(&user)
        .await
        .map_err(|err| {
            error!(error = ?err, %user, "[http] list_modules");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let resp = VecInto::<UserModuleDescResponse>::vec_into(modules);
    Ok(Json(resp))
}

async fn redeem_module(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Query(query): Query<RedeemQuery>,
) -> Result<Either<Redirect, Json<String>>, StatusCode> {
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    info!("Trying to redeem module with key {}", &query.key);
    let module_id = state
        .service
        .redeem_module(&query.key, &user)
        .await
        .map_err(|err| {
            warn!(error = ?err, %user, ?query, "[http] redeem_module: Unable to redeem module");
            StatusCode::FORBIDDEN
        })?;

    info!("Found module with id {}", &module_id.uuid);

    if query.redirect == Some(false) {
        Ok(Either::E2(Json(format!("/module/{}", module_id.uuid))))
    } else {
        Ok(Either::E1(Redirect::to(&format!(
            "/module/{}",
            module_id.uuid
        ))))
    }
}

#[derive(serde::Deserialize, Debug)]
struct RedeemQuery {
    key: ObfuscatedStr,
    redirect: Option<bool>,
}

async fn trigger_grading(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Path((module_id, assignment_id)): Path<(String, String)>,
) -> Result<Json<Option<OffsetDateTime>>, StatusCode> {
    state
        .service
        .repo
        .upsert_grading_task(&NewGradingTask::External {
            assignment_uuid: assignment_id.clone(),
            user_uuid: user.uuid.clone(),
        })
        .await
        .map(Json)
        .map_err(|err| {
            error!(error = ?err, %user, ?module_id, ?assignment_id, "[http] trigger_grading: Unable to trigger grading");
            StatusCode::FORBIDDEN
        })
}

async fn sync_repo(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Path((module_id, assignment_id)): Path<(String, String)>,
) -> Result<(), StatusCode> {
    state
        .service
        .sync_repo(&user, &module_id, &assignment_id, state.config.min_grading_interval_in_secs, &state.github_clients)
        .await
        .map_err(|err| {
            match err {
                SyncError::AssignmentNotFound => StatusCode::NOT_FOUND,
                SyncError::UserInstallationUnknown => StatusCode::NOT_IMPLEMENTED,
                SyncError::BadInstallationId | SyncError::Unknown(_) => {
                    error!(error = ?err, %user, ?module_id, ?assignment_id, "[http] sync_repo: Unable to sync repo");
                    StatusCode::INTERNAL_SERVER_ERROR},
            }
        })
}
