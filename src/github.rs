use crate::config::Config;
use octocrab::Octocrab;

mod client;
pub mod client_cache;

pub fn create_gh_app_client(config: &Config) -> Octocrab {
    Octocrab::builder()
        .app(
            config.github_app_id.into(),
            jsonwebtoken::EncodingKey::from_rsa_pem(config.github_app_private_key.as_bytes())
                .unwrap(),
        )
        .build()
        .expect("Unable to create GH app client")
}

pub struct GitHubUser {
    pub login: String,
    pub installation_id: String,
    pub avatar_url: String,
}