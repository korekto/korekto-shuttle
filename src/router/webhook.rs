use crate::github::webhook_models::GhWebhookEvent;
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

pub fn parse_event(event_type: &str, payload: &str) -> serde_json::Result<GhWebhookEvent> {
    let wrapped_payload = format!(r#"{{"event":"{event_type}", "payload":{payload}}}"#);
    serde_json::from_str::<GhWebhookEvent>(&wrapped_payload)
}

fn decode_hex(s: &str) -> Result<Vec<u8>, std::num::ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::github::webhook_models::{
        Account, Action, GhWebhookEvent, Installation, InstallationModification,
        InstallationRepositories, Push, Repository, RepositoryModification, RepositorySelection,
        TargetType,
    };
    use crate::router::webhook::parse_event;
    use pretty_assertions::assert_eq;
    use std::fs;

    #[test]
    #[cfg_attr(not(feature = "tests-with-resources"), ignore)]
    fn parse_installation_event() {
        let payload = fs::read_to_string("test_files/webhook_installation_payload.json").unwrap();
        let result = parse_event("installation", &payload).unwrap();

        assert_eq!(
            result,
            GhWebhookEvent::Installation(InstallationModification {
                action: Action::Created,
                installation: Installation {
                    id: 41266767,
                    account: Account {
                        login: "ledoyen".to_string()
                    },
                    repository_selection: RepositorySelection::All,
                    target_type: TargetType::User,
                }
            })
        );
    }

    #[test]
    #[cfg_attr(not(feature = "tests-with-resources"), ignore)]
    fn parse_installation_repositories_event() {
        let payload =
            fs::read_to_string("test_files/webhook_installation_repositories_payload.json")
                .unwrap();
        let result = parse_event("installation_repositories", &payload).unwrap();

        assert_eq!(
            result,
            GhWebhookEvent::InstallationRepositories(InstallationRepositories {
                action: Action::Added,
                installation: Installation {
                    id: 41266767,
                    account: Account {
                        login: "ledoyen".to_string()
                    },
                    repository_selection: RepositorySelection::All,
                    target_type: TargetType::User,
                },
                repository_selection: RepositorySelection::All,
                repositories_added: vec![Repository {
                    name: "tutu".to_string(),
                    full_name: "ledoyen/tutu".to_string(),
                    private: true,
                },],
                repositories_removed: vec![],
            })
        );
    }

    #[test]
    #[cfg_attr(not(feature = "tests-with-resources"), ignore)]
    fn parse_push_event() {
        let payload = fs::read_to_string("test_files/webhook_push_payload.json").unwrap();
        let result = parse_event("push", &payload).unwrap();

        assert_eq!(
            result,
            GhWebhookEvent::Push(Push {
                git_ref: "refs/heads/main".to_string(),
                repository: Repository {
                    name: "tutu".to_string(),
                    full_name: "ledoyen/tutu".to_string(),
                    private: true,
                },
            })
        );
    }

    #[test]
    #[cfg_attr(not(feature = "tests-with-resources"), ignore)]
    fn parse_repository_event() {
        let payload = fs::read_to_string("test_files/webhook_repository_payload.json").unwrap();
        let result = parse_event("repository", &payload).unwrap();

        assert_eq!(
            result,
            GhWebhookEvent::Repository(RepositoryModification {
                action: Action::Created,
                repository: Repository {
                    name: "tutu".to_string(),
                    full_name: "ledoyen/tutu".to_string(),
                    private: true,
                },
            })
        );
    }
}
