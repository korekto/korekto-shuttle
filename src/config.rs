use shuttle_runtime::SecretStore;
use validator::Validate;

#[derive(serde::Deserialize, Clone, validator::Validate)]
pub struct Config {
    pub base_url: String,
    pub cookie_secret_key: Option<String>,
    pub first_admin: Option<String>,
    pub github_app_id: u64,
    pub github_app_name: String,
    pub github_app_client_id: String,
    pub github_app_client_secret: String,
    pub github_app_private_key: String,
    pub github_app_webhook_secret: String,
    #[serde(default = "default_github_client_cache_size")]
    #[validate(range(min = 1))]
    pub github_client_cache_size: usize,
    pub github_runner_repo_slug: String,
    #[serde(default = "default_github_runner_workflow_id")]
    pub github_runner_workflow_id: String,
    pub github_runner_installation_id: u64,
    #[serde(default)]
    pub github_runner_callback_url_override: Option<String>,
    #[serde(default = "default_scheduler_interval_in_secs")]
    #[validate(range(min = 1))]
    pub scheduler_interval_in_secs: u64,
    #[serde(default = "default_min_grading_interval_in_secs")]
    #[validate(range(min = 1))]
    pub min_grading_interval_in_secs: i32,
    #[serde(default = "default_grading_ordered_timeout_in_secs")]
    #[validate(range(min = 1))]
    pub grading_ordered_timeout_in_secs: i32,
    #[serde(default = "default_grading_started_timeout_in_secs")]
    #[validate(range(min = 1))]
    pub grading_started_timeout_in_secs: i32,
    #[serde(default = "default_max_parallel_gradings")]
    #[validate(range(min = 1))]
    pub max_parallel_gradings: i32,
}

const fn default_github_client_cache_size() -> usize {
    50
}

fn default_github_runner_workflow_id() -> String {
    String::from("grade.yml")
}

const fn default_scheduler_interval_in_secs() -> u64 {
    15
}

const fn default_min_grading_interval_in_secs() -> i32 {
    20 * 60
}

const fn default_grading_ordered_timeout_in_secs() -> i32 {
    5 * 60
}

const fn default_grading_started_timeout_in_secs() -> i32 {
    15 * 60
}

const fn default_max_parallel_gradings() -> i32 {
    3
}

impl Config {
    #[must_use]
    pub fn runner_callback_base_url(&self) -> &str {
        self.github_runner_callback_url_override
            .as_deref()
            .unwrap_or(&self.base_url)
    }
}

impl TryFrom<SecretStore> for Config {
    type Error = anyhow::Error;

    fn try_from(value: SecretStore) -> Result<Self, Self::Error> {
        let config = envy::from_iter::<_, Self>(value)?;
        config.validate()?;
        Ok(config)
    }
}
