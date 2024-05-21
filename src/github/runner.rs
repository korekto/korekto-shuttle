use crate::config::Config;
use crate::entities::GitHubGradingTask;
use crate::github::client_cache::ClientCache;
use crate::github::url_to_slug;
use anyhow::anyhow;
use jsonwebtoken::jwk::{AlgorithmParameters, JwkSet};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use time::OffsetDateTime;

#[derive(Clone)]
pub struct Runner {
    org_name: String,
    repo_name: String,
    installation_id: u64,
    client: ClientCache,
    config: Config,
    jwk_set: JwkSet,
}

impl Runner {
    pub async fn new(
        org_name: String,
        repo_name: String,
        installation_id: u64,
        client: ClientCache,
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
            installation_id,
            client,
            config,
            jwk_set,
        })
    }

    pub async fn send_grading_command(&self, task: &GitHubGradingTask) -> anyhow::Result<()> {
        let client = self.client.get_for_installation(self.installation_id)?;
        let slug = url_to_slug(&task.grader_url)
            .ok_or_else(|| anyhow!("Invalid grader URL: {}", &task.grader_url))?;
        let original_callback_url = format!("{}/github/todo", self.config.server_base_url());
        let callback_url = self
            .config
            .github_runner_callback_url_override
            .as_deref()
            .unwrap_or(&original_callback_url);
        client
            .0
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
