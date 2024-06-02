use crate::entities::{InstantGrade, User};
use crate::github::client_cache::ClientCache;
use crate::repository::Repository;
use crate::service::dtos::{NewGradeRequest, UserAssignmentResponse, VecInto};
use crate::service::{Service, SyncError};
use http::StatusCode;
use octocrab::Error;
use sqlx::{Executor, Postgres};
use time::OffsetDateTime;
use tracing::info;

impl Service {
    pub async fn update_assignment_grade(
        &self,
        user_assignment_id: i32,
        new_grade: NewGradeRequest,
    ) -> anyhow::Result<()> {
        Self::update_assignment_grade_transact(user_assignment_id, new_grade, &self.repo.pool).await
    }

    pub async fn update_assignment_grade_transact<'e, 'c: 'e, E>(
        user_assignment_id: i32,
        new_grade: NewGradeRequest,
        transaction: E,
    ) -> anyhow::Result<()>
    where
        E: 'e + Executor<'c, Database = Postgres>,
    {
        let grade: f32 = new_grade.details.iter().map(|d| d.grade).sum();
        let max_grade: f32 = new_grade
            .details
            .iter()
            .map(|d| d.max_grade.unwrap_or_default())
            .sum();

        let grade_entity = InstantGrade {
            grade,
            max_grade,
            time: new_grade.time.unwrap_or_else(OffsetDateTime::now_utc),
            short_commit_id: new_grade.short_commit_id,
            commit_url: new_grade.commit_url,
            grading_log_url: new_grade.grading_log_url,
            details: new_grade.details.vec_into(),
        };
        Repository::update_assignment_grade_transact(user_assignment_id, &grade_entity, transaction)
            .await
    }

    pub async fn sync_repo(
        &self,
        user: &User,
        module_uuid: &str,
        assignment_uuid: &str,
        min_execution_interval_in_secs: i32,
        app_client: &ClientCache,
    ) -> Result<(), SyncError> {
        let assignment = &self
            .repo
            .get_assignment(
                user,
                module_uuid,
                assignment_uuid,
                min_execution_interval_in_secs,
            )
            .await
            .map_err(SyncError::Unknown)?
            .ok_or(SyncError::AssignmentNotFound)?;
        if !assignment.repo_linked {
            let installation_id = user
                .clone()
                .installation_id
                .ok_or(SyncError::UserInstallationUnknown)?
                .parse::<u64>()
                .map_err(|_| SyncError::BadInstallationId)?;
            let gh_client = app_client
                .get_for_installation(installation_id)
                .map_err(SyncError::Unknown)?;
            let repo_result = gh_client
                .0
                .repos(&user.provider_login, &assignment.repository_name)
                .get()
                .await;

            let error: Option<String> = match repo_result {
                Ok(_) => {
                    self.link_repos(&user.provider_login, vec![&assignment.repository_name])
                        .await
                        .map_err(SyncError::Unknown)?;
                    None
                }
                Err(Error::GitHub { source, .. }, ..)
                    if source.status_code == StatusCode::NOT_FOUND =>
                {
                    Some(source.message)
                }
                Err(err) => Some(format!("{err:?}")),
            };
            if let Some(message) = error {
                info!(
                    "Syncing repo {}/{} failed: {message}",
                    &user.provider_login, &assignment.repository_name
                );
            }
        }

        Ok(())
    }

    pub async fn get_assignment(
        &self,
        user: &User,
        module_uuid: &str,
        assignment_uuid: &str,
        min_execution_interval_in_secs: i32,
    ) -> anyhow::Result<Option<UserAssignmentResponse>> {
        self.repo
            .get_assignment(
                user,
                module_uuid,
                assignment_uuid,
                min_execution_interval_in_secs,
            )
            .await
            .map(|opt| opt.and_then(|ua| ua.try_into().ok()))
    }
}
