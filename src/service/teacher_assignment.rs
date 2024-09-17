use crate::entities::{NewGradingTask, User};
use crate::service::Service;
use anyhow::Context;
use tracing::info;

impl Service {
    pub async fn trigger_mass_grading_for_assignment(
        &self,
        module_uuid: &str,
        assignment_uuid: &str,
        user: &User,
    ) -> anyhow::Result<()> {
        let students = self.repo.get_module_grades(module_uuid, user).await?;
        let size = students.len();
        for student in students {
            self.repo
                .upsert_grading_task(
                    &NewGradingTask::External {
                        assignment_uuid: assignment_uuid.to_string(),
                        user_uuid: student.uuid.to_string(),
                    },
                    false,
                )
                .await
                .context(format!(
                    "[service] trigger_mass_grading_for_assignment(student={student})"
                ))?;
        }
        info!("[service] trigger_mass_grading_for_assignment(assignment_uuid={assignment_uuid}): {size} students");
        Ok(())
    }
}
