use axum::extract::State;
use axum::{routing::get, Router};

use crate::github::webhook_models::Event;
use crate::router::state::AppState;
use crate::string_header;

string_header!(XGithubEvent, X_GITHUB_EVENT_HEADER, "x-github-event");
string_header!(XHubSignature, X_HUB_SIGNATURE, "x-hub-signature-256");

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/github", get(on_github_event))
        .fallback(crate::router::fallback)
}

#[allow(clippy::unused_async)]
async fn on_github_event(
    XGithubEvent(event_type): XGithubEvent,
    XHubSignature(signature): XHubSignature,
    State(state): State<AppState>,
    payload: String,
) {
    let event_result = to_event(
        &event_type,
        &payload,
        &state.config.github_app_webhook_secret,
        &signature,
    );

    match event_result {
        Ok(result) => {
            // TODO save user info
            tracing::info!("Received webhook event={result:?}");
        }
        Err(err) => {
            tracing::warn!("Webhook failure: {err:?}");
        }
    }
}

type HmacSha256 = hmac::Hmac<sha2::Sha256>;

pub fn to_event(
    event_type: &str,
    payload: &str,
    secret: &str,
    signature: &str,
) -> Result<Event, EventError> {
    use hmac::Mac;

    if let Some((_raw_alg, sig)) = signature.split_once('=') {
        #[allow(clippy::expect_used)]
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .expect("could not fail, waiting for into_ok() stabilization");

        mac.update(payload.as_bytes());

        let code_bytes = decode_hex(sig).map_err(|_| EventError::InvalidSignature)?;

        mac.verify_slice(&code_bytes[..])
            .map_err(|_| EventError::InvalidSignature)?;

        let wrapped_payload = format!(r#"{{"event":"{event_type}", "payload":{payload}}}"#);
        serde_json::from_str::<Event>(&wrapped_payload).map_err(EventError::ParsingError)
    } else {
        Err(EventError::InvalidSignature)
    }
}

fn decode_hex(s: &str) -> Result<Vec<u8>, std::num::ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

#[derive(Debug)]
pub enum EventError {
    InvalidSignature,
    ParsingError(serde_json::Error),
}
