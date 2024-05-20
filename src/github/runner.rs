use crate::config::Config;
use crate::entities::GitHubGradingTask;
use crate::github::client_cache::ClientCache;
use crate::github::url_to_slug;
use anyhow::anyhow;

#[derive(Clone)]
pub struct Runner {
    org_name: String,
    repo_name: String,
    installation_id: u64,
    client: ClientCache,
    config: Config,
}

impl Runner {
    pub const fn new(
        org_name: String,
        repo_name: String,
        installation_id: u64,
        client: ClientCache,
        config: Config,
    ) -> Self {
        Self {
            org_name,
            repo_name,
            installation_id,
            client,
            config,
        }
    }

    pub async fn send_grading_command(&self, task: &GitHubGradingTask) -> anyhow::Result<()> {
        let client = self.client.get_for_installation(self.installation_id)?;
        let slug = url_to_slug(&task.grader_url)
            .ok_or_else(|| anyhow!("Invalid grader URL: {}", &task.grader_url))?;
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
                "start-callback-url": format!("{}/github/todo", self.config.server_base_url()),
                "callback-url": format!("{}/github/todo", self.config.server_base_url())
            }))
            .send()
            .await?;

        Ok(())
    }
}
