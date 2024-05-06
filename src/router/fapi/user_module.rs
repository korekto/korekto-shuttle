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
use tracing::{error, info, warn};

use crate::router::auth::AuthenticatedUser;
use crate::router::state::AppState;
use crate::service::dtos::{
    UserAssignmentResponse, UserModuleDescResponse, UserModuleResponse, VecInto,
};
use crate::service::ObfuscatedStr;

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
}

async fn get_assignment(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Path((module_id, assignment_id)): Path<(String, String)>,
) -> Result<Json<UserAssignmentResponse>, StatusCode> {
    let assignment = state
        .service
        .repo
        .get_assignment(&user, &module_id, &assignment_id)
        .await
        .map_err(|err| {
            error!("get_assignment {err:#?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(assignment.into()))
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
            error!("get_module {err:#?}");
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
            error!("list_modules {err:#?}");
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
    info!("Trying to redeem module with key {}", &query.key);
    let module_id = state
        .service
        .redeem_module(&query.key, &user)
        .await
        .map_err(|err| {
            warn!(
                "Unable to redeem module with key: {}, for user {user}: {err}",
                &query.key
            );
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

#[derive(serde::Deserialize)]
struct RedeemQuery {
    key: ObfuscatedStr,
    redirect: Option<bool>,
}

async fn trigger_grading(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Path((module_id, assignment_id)): Path<(String, String)>,
) -> Result<(), StatusCode> {
    state
        .service
        .repo
        .upsert_grading_task(&NewGradingTask::External {
            user_assignment_uuid: assignment_id.clone(),
            user_uuid: user.uuid.clone(),
        })
        .await
        .map_err(|err| {
            warn!(
                "Unable to trigger grading of module: {module_id}, assignment: {assignment_id}, for user {user}: {err}"
            );
            StatusCode::FORBIDDEN
        })?;
    Ok(())
}
