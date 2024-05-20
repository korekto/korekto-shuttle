use crate::github::webhook_models::parse_event;
use crate::router::state::AppState;
use crate::string_header;
use anyhow::anyhow;
use axum::extract::State;
use axum::{routing::post, Router};
use axum_extra::TypedHeader;

string_header!(XGithubEvent, X_GITHUB_EVENT_HEADER, "x-github-event");
string_header!(XHubSignature, X_HUB_SIGNATURE, "x-hub-signature-256");

pub fn router() -> Router<AppState> {
    Router::new().route("/github", post(on_github_event))
}

#[allow(clippy::unused_async)]
async fn on_github_event(
    TypedHeader(XGithubEvent(event_type)): TypedHeader<XGithubEvent>,
    TypedHeader(XHubSignature(signature)): TypedHeader<XHubSignature>,
    State(state): State<AppState>,
    payload: String,
) {
    if let Err(err) = is_signature_valid(
        &payload,
        &state.config.github_app_webhook_secret,
        &signature,
    ) {
        tracing::debug!("Received webhook \"{event_type}\" with invalid signature: {err:?}");
    } else {
        match parse_event(&event_type, &payload) {
            Ok(result) => {
                // TODO save user info
                tracing::info!("Received webhook event={result:?}");
            }
            Err(err) => {
                if let Err(err) = state
                    .service
                    .repo
                    .insert_unparseable_webhook("github", &event_type, &payload, &err.to_string())
                    .await
                {
                    tracing::warn!("Fail to save unparseable webhook: {err:?}");
                }
            }
        }
    }
}

pub fn is_signature_valid(payload: &str, secret: &str, signature: &str) -> anyhow::Result<()> {
    use hmac::Mac;

    type HmacSha256 = hmac::Hmac<sha2::Sha256>;

    if let Some((_raw_alg, sig)) = signature.split_once('=') {
        #[allow(clippy::expect_used)]
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .expect("could not fail, waiting for into_ok() stabilization");

        mac.update(payload.as_bytes());

        let code_bytes = decode_hex(sig)?;

        mac.verify_slice(&code_bytes[..])?;

        Ok(())
    } else {
        Err(anyhow!("Missing algorithm:"))
    }
}

fn decode_hex(s: &str) -> Result<Vec<u8>, std::num::ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}
