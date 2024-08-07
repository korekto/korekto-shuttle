use crate::config::Config;
use crate::entities::GitHubGradingTask;
use crate::github::url_to_slug;
use anyhow::anyhow;
use jsonwebtoken::{
    decode, decode_header,
    jwk::{AlgorithmParameters, JwkSet},
    Algorithm, DecodingKey, Validation,
};
use octocrab::{models::Repository, Octocrab, Page};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use time::OffsetDateTime;
use tracing::info;

#[derive(Clone)]
pub struct Runner {
    org_name: String,
    repo_name: String,
    app_client: Octocrab,
    installation_client: Octocrab,
    config: Config,
    jwk_set: JwkSet,
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct Metadata {
    app_id: u64,
    app_name: String,
    accessible_repositories: Vec<String>,
}

impl Runner {
    pub async fn new(
        org_name: String,
        repo_name: String,
        app_client: Octocrab,
        installation_client: Octocrab,
        config: Config,
    ) -> anyhow::Result<Self> {
        let raw_jwk_set =
            reqwest::get("https://token.actions.githubusercontent.com/.well-known/jwks")
                .await?
                .text()
                .await?;
        let jwk_set = serde_json::from_str(&raw_jwk_set)?;
        Ok(Self {
            org_name,
            repo_name,
            app_client,
            installation_client,
            config,
            jwk_set,
        })
    }

    pub async fn metadata(&self) -> anyhow::Result<Metadata> {
        let app = self.app_client.current().app().await?;
        let repos: Page<Repository> = self
            .installation_client
            .get("/installation/repositories", None::<&()>)
            .await?;
        let accessible_repositories = repos
            .into_iter()
            .map(|r| {
                format!(
                    "{}/{}",
                    r.owner.map_or("<unknown>".to_string(), |o| o.login),
                    r.name
                )
            })
            .collect();
        Ok(Metadata {
            app_id: app.id.into_inner(),
            app_name: app.name,
            accessible_repositories,
        })
    }

    pub async fn send_grading_command(&self, task: &GitHubGradingTask) -> anyhow::Result<()> {
        let slug = url_to_slug(&task.grader_url)
            .ok_or_else(|| anyhow!("Invalid grader URL: {}", &task.grader_url))?;
        let original_callback_url = format!(
            "{}/webhook/github/runner",
            self.config.runner_callback_base_url()
        );
        let callback_url = self
            .config
            .github_runner_callback_url_override
            .as_deref()
            .unwrap_or(&original_callback_url);
        info!(
            "Triggering remote job: {}/{} - {}",
            &self.org_name, &self.repo_name, &self.config.github_runner_workflow_id
        );
        self.installation_client
            .actions()
            .create_workflow_dispatch(
                &self.org_name,
                &self.repo_name,
                &self.config.github_runner_workflow_id,
                "main",
            )
            .inputs(serde_json::json!({
                "grader-repo": slug.to_string(),
                "student-login": task.provider_login,
                "student-repo": task.repository_name,
                "callback-url": callback_url,
                "task-id": task.uuid,
            }))
            .send()
            .await?;

        Ok(())
    }

    pub fn verify_jwt(&self, jwt: &str) -> anyhow::Result<()> {
        let header = decode_header(jwt)?;
        let key_store = self
            .jwk_set
            .find(
                header
                    .kid
                    .as_ref()
                    .ok_or_else(|| anyhow!("Mising KID from JWT"))?,
            )
            .ok_or_else(|| anyhow!("No GH JWK matching kid={:?}", &header.kid))?;
        let alg = Algorithm::from_str(
            key_store
                .common
                .key_algorithm
                .ok_or_else(|| {
                    anyhow!(
                        "Key {:?} is missing algorithm in JWK set",
                        key_store.common.key_id
                    )
                })?
                .to_string()
                .as_str(),
        )?;
        let mut validation = Validation::new(alg);
        validation.set_audience(&[format!("https://github.com/{}", self.org_name)]);

        let token_message =
            decode::<GitHubClaims>(jwt, &key_store.algorithm.decoding_key()?, &validation)?;
        if token_message.claims.repository == format!("{}/{}", self.org_name, self.repo_name) {
            Ok(())
        } else {
            Err(anyhow!(
                "Grading from a rogue repo: {}",
                token_message.claims.repository
            ))
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

            let code_bytes = Self::decode_hex(sig)?;

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
}

trait AlgParams {
    fn decoding_key(&self) -> anyhow::Result<DecodingKey>;
}

impl AlgParams for AlgorithmParameters {
    fn decoding_key(&self) -> anyhow::Result<DecodingKey> {
        match self {
            Self::RSA(params) => Ok(DecodingKey::from_rsa_components(&params.n, &params.e)?),
            _ => Err(anyhow!("Unsupported algorithm: {self:?}")),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct GitHubClaims {
    repository: String,
    iss: String,
    #[serde(with = "time::serde::timestamp")]
    exp: OffsetDateTime,
}
