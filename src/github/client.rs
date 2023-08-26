use octocrab::Octocrab;

#[derive(Clone)]
pub struct InstallationClient {
    pub octocrab: Octocrab,
}

impl InstallationClient {
    pub fn new(installation_client: Octocrab) -> Self {
        Self {
            octocrab: installation_client,
        }
    }
}
