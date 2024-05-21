use serde::Deserialize;
use time::serde::rfc3339 as gh_webhook_time_serde;
use time::OffsetDateTime;

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "event", content = "payload", rename_all = "snake_case")]
pub enum GhWebhookEvent {
    InstallationRepositories(InstallationRepositories),
    Installation(InstallationModification),
    Push(Push),
    Repository(RepositoryModification),
    WorkflowJob(WorkflowJobEvent),
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct WorkflowJobEvent {
    pub workflow_job: WorkflowJob,
    pub repository: RepositoryWithOwner,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct WorkflowJob {
    pub html_url: String,
    pub status: WorkflowJobStatus,
    pub conclusion: Option<WorkflowJobConclusion>,
    #[serde(with = "gh_webhook_time_serde")]
    pub created_at: OffsetDateTime,
    #[serde(with = "gh_webhook_time_serde::option")]
    pub completed_at: Option<OffsetDateTime>,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowJobStatus {
    Queued,
    InProgress,
    Completed,
    Waiting,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowJobConclusion {
    Success,
    Failure,
    Skipped,
    Cancelled,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct Push {
    #[serde(rename = "ref")]
    pub git_ref: String,
    pub repository: RepositoryWithOwner,
    pub sender: Account,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct PushRepository {
    pub name: String,
    pub full_name: String,
    pub owner: Account,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct InstallationRepositories {
    pub action: RepositoryAction,
    pub installation: Installation,
    pub repository_selection: RepositorySelection,
    pub repositories_added: Vec<Repository>,
    pub repositories_removed: Vec<Repository>,
    pub sender: Account,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct InstallationModification {
    pub action: RepositoryAction,
    pub installation: Installation,
    pub repositories: Vec<Repository>,
    pub sender: Account,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct RepositoryModification {
    pub action: RepositoryAction,
    pub repository: RepositoryWithOwner,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RepositoryAction {
    Added,
    Created,
    Removed,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct Installation {
    pub id: u64,
    pub account: Account,
    pub repository_selection: RepositorySelection,
    pub target_type: TargetType,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RepositorySelection {
    Selected,
    All,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum TargetType {
    User,
    Organization,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct Repository {
    pub name: String,
    pub full_name: String,
    pub private: bool,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct RepositoryWithOwner {
    pub name: String,
    pub full_name: String,
    pub private: bool,
    pub owner: Account,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct Account {
    pub login: String,
}

impl<'a> From<&'a Repository> for &'a str {
    fn from(repo: &'a Repository) -> Self {
        repo.name.as_str()
    }
}

pub fn parse_event(event_type: &str, payload: &str) -> serde_json::Result<GhWebhookEvent> {
    let wrapped_payload = format!(r#"{{"event":"{event_type}", "payload":{payload}}}"#);
    serde_json::from_str::<GhWebhookEvent>(&wrapped_payload)
}

#[cfg(test)]
mod tests {
    use crate::github::webhook_models::{
        parse_event, Account, GhWebhookEvent, Installation, InstallationModification,
        InstallationRepositories, Push, Repository, RepositoryAction, RepositoryModification,
        RepositorySelection, RepositoryWithOwner, TargetType, WorkflowJob, WorkflowJobEvent,
        WorkflowJobStatus,
    };
    use pretty_assertions::assert_eq;
    use std::fs;
    use time::macros::datetime;

    #[test]
    #[cfg_attr(not(feature = "tests-with-resources"), ignore)]
    fn parse_installation_event() {
        let payload = fs::read_to_string("test_files/webhook_installation_payload.json").unwrap();
        let result = parse_event("installation", &payload).unwrap();

        assert_eq!(
            result,
            GhWebhookEvent::Installation(InstallationModification {
                action: RepositoryAction::Created,
                installation: Installation {
                    id: 41266767,
                    account: Account {
                        login: "ledoyen".to_string()
                    },
                    repository_selection: RepositorySelection::All,
                    target_type: TargetType::User,
                },
                repositories: vec![
                    Repository {
                        name: "aash".to_string(),
                        full_name: "ledoyen/aash".to_string(),
                        private: false,
                    },
                    Repository {
                        name: "spring-automocker".to_string(),
                        full_name: "ledoyen/spring-automocker".to_string(),
                        private: true,
                    },
                ],
                sender: Account {
                    login: "ledoyen".to_string()
                },
            })
        );
    }

    #[test]
    #[cfg_attr(not(feature = "tests-with-resources"), ignore)]
    fn parse_installation_repositories_event() {
        let payload =
            fs::read_to_string("test_files/webhook_installation_repositories_payload.json")
                .unwrap();
        let result = parse_event("installation_repositories", &payload).unwrap();

        assert_eq!(
            result,
            GhWebhookEvent::InstallationRepositories(InstallationRepositories {
                action: RepositoryAction::Added,
                installation: Installation {
                    id: 41266767,
                    account: Account {
                        login: "ledoyen".to_string()
                    },
                    repository_selection: RepositorySelection::All,
                    target_type: TargetType::User,
                },
                repository_selection: RepositorySelection::All,
                repositories_added: vec![Repository {
                    name: "tutu".to_string(),
                    full_name: "ledoyen/tutu".to_string(),
                    private: true,
                },],
                repositories_removed: vec![],
                sender: Account {
                    login: "ledoyen".to_string()
                },
            })
        );
    }

    #[test]
    #[cfg_attr(not(feature = "tests-with-resources"), ignore)]
    fn parse_push_event() {
        let payload = fs::read_to_string("test_files/webhook_push_payload.json").unwrap();
        let result = parse_event("push", &payload).unwrap();

        assert_eq!(
            result,
            GhWebhookEvent::Push(Push {
                git_ref: "refs/heads/main".to_string(),
                repository: RepositoryWithOwner {
                    name: "tutu".to_string(),
                    full_name: "ledoyen/tutu".to_string(),
                    private: true,
                    owner: Account {
                        login: "ledoyen".to_string()
                    },
                },
                sender: Account {
                    login: "ledoyen".to_string()
                },
            })
        );
    }

    #[test]
    #[cfg_attr(not(feature = "tests-with-resources"), ignore)]
    fn parse_repository_event() {
        let payload = fs::read_to_string("test_files/webhook_repository_payload.json").unwrap();
        let result = parse_event("repository", &payload).unwrap();

        assert_eq!(
            result,
            GhWebhookEvent::Repository(RepositoryModification {
                action: RepositoryAction::Created,
                repository: RepositoryWithOwner {
                    name: "tutu".to_string(),
                    full_name: "ledoyen/tutu".to_string(),
                    private: true,
                    owner: Account {
                        login: "ledoyen".to_string()
                    },
                },
            })
        );
    }

    #[test]
    #[cfg_attr(not(feature = "tests-with-resources"), ignore)]
    fn parse_workflow_job_event() {
        let payload = fs::read_to_string("test_files/webhook_workflow_job_payload.json").unwrap();
        let result = parse_event("workflow_job", &payload).unwrap();

        assert_eq!(
            result,
            GhWebhookEvent::WorkflowJob(WorkflowJobEvent {
                workflow_job: WorkflowJob {
                    html_url: "https://github.com/lernejo/korekto-runner/actions/runs/9160150850/job/25182159656".to_string(),
                    status: WorkflowJobStatus::Queued,
                    conclusion: None,
                    created_at: datetime!(2024-05-20 14:19:46 UTC),
                    completed_at: None,
                },
                repository: RepositoryWithOwner {
                    name: "korekto-runner".to_string(),
                    full_name: "lernejo/korekto-runner".to_string(),
                    private: false,
                    owner: Account {
                        login: "lernejo".to_string()
                    },
                },
            })
        );
    }
}
