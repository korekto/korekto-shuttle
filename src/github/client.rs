use octocrab::Octocrab;

#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct GitHubClient(pub Octocrab);

// This is really no the good approach, waiting for async 2023
// stabilization to have a proper trait for new methods on Octocrab
impl GitHubClient {
    pub const fn new(octocrab_client: Octocrab) -> Self {
        Self(octocrab_client)
    }

    pub async fn current_user(&self) -> anyhow::Result<CustomAuthor> {
        Ok(self.0.get("/user", None::<&()>).await?)
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct CustomAuthor {
    pub login: String,
    pub avatar_url: String,
    pub email: Option<String>,
    pub name: Option<String>,
}
