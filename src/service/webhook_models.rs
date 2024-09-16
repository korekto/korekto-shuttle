use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
pub struct RunnerPayload {
    pub status: RunnerStatus,
    pub student_login: String,
    pub grader_repo: String,
    pub task_id: String,
    pub full_log_url: String,
    pub details: Option<RunnerGradeDetails>,
    pub metadata: RunnerMetadata,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunnerStatus {
    Started,
    Completed,
    Failure,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct RunnerGradeDetails {
    pub grade: f32,
    #[serde(rename = "maxGrade")]
    pub max_grade: f32,
    pub parts: Vec<RunnerGradePart>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct RunnerGradePart {
    pub id: String,
    pub grade: f32,
    #[serde(rename = "maxGrade")]
    pub max_grade: Option<f32>,
    pub comments: Vec<String>,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct RunnerMetadata {
    pub commit_id: Option<String>,
    pub short_commit_id: Option<String>,
    pub commit_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use crate::service::webhook_models::{RunnerMetadata, RunnerPayload, RunnerStatus};
    use pretty_assertions::assert_eq;
    use std::fs;

    #[test]
    #[cfg_attr(not(feature = "tests-with-resources"), ignore)]
    fn parse_failure_payload() {
        let payload = fs::read_to_string("test_files/runner_webhook_failure.json").unwrap();
        let result: RunnerPayload = serde_json::from_str(&payload).unwrap();

        assert_eq!(result, RunnerPayload {
            status: RunnerStatus::Failure,
            student_login: "ledoyen".to_string(),
            grader_repo: "lernejo/korekto-java-basics-grader".to_string(),
            task_id: "518e2eba-8d12-4833-ab5c-460b0e4a9fa6".to_string(),
            full_log_url: "https://github.com/lernejo/korekto-runner/actions/runs/9255315322".to_string(),
            details: None,
            metadata: RunnerMetadata {
                commit_id: Some("cb2fa5425250371d70a9d06f4fb7202cd62b7738".to_string()),
                short_commit_id: Some("cb2fa54".to_string()),
                commit_url: Some("https://github.com/ledoyen/java_exercise_1/commit/cb2fa5425250371d70a9d06f4fb7202cd62b7738".to_string()),
            },
        })
    }
}
