use crate::entities::{InstantGrade, User};
use crate::github::client_cache::ClientCache;
use crate::service::dtos::{NewGradeRequest, VecInto};
use crate::service::{Service, SyncError};
use time::OffsetDateTime;
use tracing::info;

impl Service {
    pub async fn update_assignment_grade(
        &self,
        user_uuid: &str,
        assignment_uuid: &str,
        new_grade: NewGradeRequest,
    ) -> anyhow::Result<()> {
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
        self.repo
            .update_assignment_grade(user_uuid, assignment_uuid, &grade_entity)
            .await
    }

    pub async fn sync_repo(
        &self,
        user: &User,
        module_uuid: &str,
        assignment_uuid: &str,
        app_client: &ClientCache,
    ) -> Result<(), SyncError> {
        let assignment = &self
            .repo
            .get_assignment(user, module_uuid, assignment_uuid)
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
            match repo_result {
                Ok(_) => {
                    self.link_repos(&user.provider_login, vec![&assignment.repository_name])
                        .await
                        .map_err(SyncError::Unknown)?;
                }
                Err(err) => {
                    info!(
                        "Syncing repo {}/{} failed: {err:?}",
                        &user.provider_login, &assignment.repository_name
                    );
                }
            }
        }

        Ok(())
    }
}
