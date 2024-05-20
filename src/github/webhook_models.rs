use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "event", content = "payload", rename_all = "snake_case")]
pub enum GhWebhookEvent {
    InstallationRepositories(InstallationRepositories),
    Installation(InstallationModification),
    Push(Push),
    Repository(RepositoryModification),
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
    pub action: Action,
    pub installation: Installation,
    pub repository_selection: RepositorySelection,
    pub repositories_added: Vec<Repository>,
    pub repositories_removed: Vec<Repository>,
    pub sender: Account,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct InstallationModification {
    pub action: Action,
    pub installation: Installation,
    pub repositories: Vec<Repository>,
    pub sender: Account,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct RepositoryModification {
    pub action: Action,
    pub repository: RepositoryWithOwner,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Action {
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
        parse_event, Account, Action, GhWebhookEvent, Installation, InstallationModification,
        InstallationRepositories, Push, Repository, RepositoryModification, RepositorySelection,
        RepositoryWithOwner, TargetType,
    };
    use pretty_assertions::assert_eq;
    use std::fs;

    #[test]
    #[cfg_attr(not(feature = "tests-with-resources"), ignore)]
    fn parse_installation_event() {
        let payload = fs::read_to_string("test_files/webhook_installation_payload.json").unwrap();
        let result = parse_event("installation", &payload).unwrap();

        assert_eq!(
            result,
            GhWebhookEvent::Installation(InstallationModification {
                action: Action::Created,
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
                action: Action::Added,
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
                action: Action::Created,
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
}
