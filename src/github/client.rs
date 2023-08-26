use octocrab::Octocrab;

#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct InstallationClient {
    pub octocrab: Octocrab,
}

impl InstallationClient {
    pub const fn new(installation_client: Octocrab) -> Self {
        Self {
            octocrab: installation_client,
        }
    }
}
