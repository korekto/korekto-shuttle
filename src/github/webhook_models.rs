use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(tag = "event", content = "payload", rename_all = "snake_case")]
pub enum Event {
    InstallationRepositories(InstallationRepositories),
    Installation(InstallationModification),
    Push(Push),
    Repository(RepositoryModification),
}

#[derive(Deserialize, Debug)]
pub struct Push {
    #[serde(rename = "ref")]
    pub git_ref: String,
    pub repository: String,
}

#[derive(Deserialize, Debug)]
pub struct PushRepository {
    pub name: String,
    pub full_name: String,
    pub owner: Account,
}

#[derive(Deserialize, Debug)]
pub struct InstallationRepositories {
    pub action: Action,
    pub installation: Installation,
    pub repository_selection: RepositorySelection,
    pub repositories_added: Vec<Repository>,
    pub repositories_removed: Vec<Repository>,
}

#[derive(Deserialize, Debug)]
pub struct InstallationModification {
    pub action: Action,
    pub installation: Installation,
}

#[derive(Deserialize, Debug)]
pub struct RepositoryModification {
    pub action: Action,
    pub repository: Repository,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Added,
    Created,
    Removed,
}

#[derive(Deserialize, Debug)]
pub struct Installation {
    pub id: u64,
    pub account: Account,
    pub repository_selection: RepositorySelection,
    pub target_type: TargetType,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum RepositorySelection {
    Selected,
    All,
}

#[derive(Deserialize, Debug)]
pub enum TargetType {
    User,
    Organization,
}

#[derive(Deserialize, Debug)]
pub struct Repository {
    pub name: String,
    pub full_name: String,
    pub private: bool,
    pub owner: Account,
}

#[derive(Deserialize, Debug)]
pub struct Account {
    pub login: String,
}
