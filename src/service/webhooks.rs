use crate::entities::{GradingMetadata, NewGradingTask};
use crate::github::webhook_models::GhWebhookEvent;
use crate::repository::grading_task::GradingStatus;
use crate::repository::Repository;
use crate::service::dtos::{NewGradeRequest, VecInto};
use crate::service::webhook_models::{RunnerPayload, RunnerStatus};
use crate::service::Service;
use time::OffsetDateTime;
use tracing::{debug, warn};

impl Service {
    pub async fn on_webhook(&self, event: GhWebhookEvent) -> anyhow::Result<()> {
        match event {
            GhWebhookEvent::InstallationRepositories(ir) => {
                if ir.action == crate::github::webhook_models::RepositoryAction::Added {
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
                if i.action == crate::github::webhook_models::RepositoryAction::Created {
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
                if r.action == crate::github::webhook_models::RepositoryAction::Created {
                    self.link_repos(&r.repository.owner.login, vec![&r.repository.name])
                        .await?;
                } else {
                    warn!("Unhandled webhook Repository action: {:?}", r.action);
                }
            }
            // Ignore these events as they carry no correlation information about the student or the assignment
            GhWebhookEvent::WorkflowJob(_) => {}
        };
        Ok(())
    }

    pub async fn link_repos(
        &self,
        user_provider_login: &str,
        repo_names: Vec<&str>,
    ) -> anyhow::Result<()> {
        let retained_repos = self
            .repo
            .upsert_user_assignments(user_provider_login, &repo_names, true)
            .await?;
        for retained_assignment in retained_repos {
            self.repo
                .upsert_grading_task(&NewGradingTask::Internal {
                    user_assignment_id: retained_assignment.id,
                    user_provider_name: user_provider_login.to_string(),
                    repository: retained_assignment.repository_name,
                    grader_repository: retained_assignment.grader_url,
                })
                .await?;
        }
        Ok(())
    }

    pub async fn on_runner_webhook(&self, event: &RunnerPayload) -> anyhow::Result<()> {
        debug!("Received runner event: {event:?}");
        match event.status {
            RunnerStatus::Started => {
                self.on_runner_event_started(event).await?;
            }
            RunnerStatus::Completed => {
                self.on_runner_event_completed(event).await?;
            }
            RunnerStatus::Failure => {
                self.repo
                    .delete_grading_task(
                        &event.task_id,
                        Some("GitHub runner job failed".to_string()),
                    )
                    .await?;
            }
        };
        Ok(())
    }

    async fn on_runner_event_started(&self, event: &RunnerPayload) -> anyhow::Result<()> {
        let mut transaction = self.repo.start_transaction().await?;

        let raw_grading_task = Repository::update_grading_task_non_terminal_status_transact(
            &event.task_id,
            &GradingStatus::STARTED,
            &mut *transaction,
        )
        .await?;
        let metadata = GradingMetadata {
            short_commit_id: event.metadata.short_commit_id.clone(),
            commit_url: event.metadata.commit_url.clone(),
            full_log_url: event.full_log_url.clone(),
        };
        Repository::update_assignment_current_grading_metadata(
            raw_grading_task.user_assignment_id,
            &metadata,
            &mut *transaction,
        )
        .await?;

        transaction.commit().await?;
        Ok(())
    }

    async fn on_runner_event_completed(&self, event: &RunnerPayload) -> anyhow::Result<()> {
        let mut transaction = self.repo.start_transaction().await?;
        let error_message = if event.details.is_none() {
            Some("GitHub runner job completed without grading details".to_string())
        } else {
            None
        };
        let task = Repository::delete_grading_task_transact(
            &event.task_id,
            error_message,
            &mut *transaction,
        )
        .await?;

        if let Some(details) = &event.details {
            let grade = NewGradeRequest {
                time: Some(OffsetDateTime::now_utc()),
                short_commit_id: event.metadata.short_commit_id.clone(),
                commit_url: event.metadata.commit_url.clone(),
                grading_log_url: event.full_log_url.clone(),
                details: details.parts.clone().vec_into(),
            };
            Self::update_assignment_grade_transact(
                task.user_assignment_id,
                grade,
                &mut *transaction,
            )
            .await?;
        }

        transaction.commit().await?;
        Ok(())
    }
}
