use crate::github::runner::Runner;
use crate::github::webhook_models::parse_event;
use crate::router::state::AppState;
use crate::service::webhook_models::RunnerPayload;
use crate::string_header;
use axum::extract::rejection::JsonRejection;
use axum::extract::State;
use axum::{routing::post, Json, Router};
use axum_extra::TypedHeader;
use headers::authorization::Bearer;
use headers::Authorization;
use http::StatusCode;
use tracing::{debug, error};

string_header!(XGithubEvent, X_GITHUB_EVENT_HEADER, "x-github-event");
string_header!(XHubSignature, X_HUB_SIGNATURE, "x-hub-signature-256");

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/github", post(on_github_event))
        .route("/github/runner", post(on_github_runner_event))
}

#[allow(clippy::unused_async)]
async fn on_github_event(
    TypedHeader(XGithubEvent(event_type)): TypedHeader<XGithubEvent>,
    TypedHeader(XHubSignature(signature)): TypedHeader<XHubSignature>,
    State(state): State<AppState>,
    payload: String,
) {
    if let Err(err) = Runner::is_signature_valid(
        &payload,
        &state.config.github_app_webhook_secret,
        &signature,
    ) {
        debug!(error = ?err, ?event_type, "Received webhook with invalid signature");
    } else {
        match parse_event(&event_type, &payload) {
            Ok(result) => {
                let _res = state.service.on_webhook(result).await;
            }
            Err(err) => {
                if let Err(err) = state
                    .service
                    .repo
                    .insert_unparseable_webhook("github", &event_type, &payload, &err.to_string())
                    .await
                {
                    error!(error = ?err, ?event_type, ?payload, "[http] on_github_event");
                }
            }
        }
    }
}

#[allow(clippy::unused_async)]
async fn on_github_runner_event(
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    State(state): State<AppState>,
    payload_result: Result<Json<RunnerPayload>, JsonRejection>,
) -> Result<(), (StatusCode, String)> {
    match payload_result {
        Ok(Json(payload)) => {
            state.gh_runner.verify_jwt(bearer.token()).map_err(|err| {
                debug!(error = ?err, token = bearer.token(), ?payload, "[http] on_github_runner_event: Invalid JWT");
                (StatusCode::UNAUTHORIZED, format!("{err:?}"))
            })?;

            state
                .service
                .on_runner_webhook(&payload)
                .await
                .map_err(|err| {
                    error!(error = ?err, ?payload, "[http] on_github_runner_event: Unknown error");
                    (StatusCode::INTERNAL_SERVER_ERROR, format!("{err:?}"))
                })?;
        }
        Err(err) => {
            error!(error = ?err, payload = err.body_text(), "[http] on_github_runner_event: Invalid JSON");
        }
    }

    Ok(())
}
