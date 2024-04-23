use crate::entities::NewGradingTask;
use crate::github::webhook_models::GhWebhookEvent;
use crate::service::Service;
use tracing::warn;

impl Service {
    pub async fn on_webhook(&self, event: GhWebhookEvent) -> anyhow::Result<()> {
        match event {
            GhWebhookEvent::InstallationRepositories(ir) => {
                if ir.action == crate::github::webhook_models::Action::Added {
                    let repo_names = ir
                        .repositories_added
                        .iter()
                        .map(Into::into)
                        .collect::<Vec<&str>>();
                    self.link_repos(&ir.installation.account.login, repo_names)
                        .await?;
                } else {
                    warn!(
                        "Unhandled webhook InstallationRepositories action: {:?}",
                        ir.action
                    );
                }
            }
            GhWebhookEvent::Installation(i) => {
                if i.action == crate::github::webhook_models::Action::Created {
                    let repo_names = i
                        .repositories
                        .iter()
                        .map(|repo| repo.name.as_str())
                        .collect::<Vec<&str>>();
                    self.link_repos(&i.installation.account.login, repo_names)
                        .await?;
                } else {
                    warn!("Unhandled webhook Installation action: {:?}", i.action);
                }
            }
            GhWebhookEvent::Push(p) => {
                self.link_repos(&p.repository.owner.login, vec![&p.repository.name])
                    .await?;
            }
            GhWebhookEvent::Repository(r) => {
                if r.action == crate::github::webhook_models::Action::Created {
                    self.link_repos(&r.repository.owner.login, vec![&r.repository.name])
                        .await?;
                } else {
                    warn!("Unhandled webhook Repository action: {:?}", r.action);
                }
            }
        };
        Ok(())
    }

    async fn link_repos(
        &self,
        user_provider_name: &str,
        repo_names: Vec<&str>,
    ) -> anyhow::Result<()> {
        let retained_repos = self
            .repo
            .upsert_user_assignments(user_provider_name, &repo_names, true)
            .await?;
        for retained_assignment in retained_repos {
            self.repo
                .upsert_grading_task(&NewGradingTask {
                    user_assignment_id: retained_assignment.id,
                    user_provider_name: user_provider_name.to_string(),
                    repository: retained_assignment.repository_name,
                    grader_repository: retained_assignment.grader_url,
                })
                .await?;
        }
        Ok(())
    }
}
