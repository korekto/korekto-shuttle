use crate::config::Config;
use octocrab::Octocrab;

mod client;
pub mod client_cache;

pub fn create_gh_app_client(config: &Config) -> anyhow::Result<Octocrab> {
    Ok(Octocrab::builder()
        .app(
            config.github_app_id.into(),
            jsonwebtoken::EncodingKey::from_rsa_pem(config.github_app_private_key.as_bytes())?,
        )
        .build()?)
}

pub struct GitHubUserLogged {
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: String,
    pub email: Option<String>,
}
