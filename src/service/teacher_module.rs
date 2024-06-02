use crate::entities::User;
use crate::service::dtos::{GradeAssignmentResponse, ModuleGradesResponse, StudentGradesResponse};
use crate::service::Service;

impl Service {
    pub async fn get_grades(
        &self,
        uuid: &str,
        teacher: &User,
    ) -> anyhow::Result<ModuleGradesResponse> {
        let entities = self.repo.get_grades(uuid, teacher).await?;
        let assignments: Vec<GradeAssignmentResponse> = entities
            .first()
            .map(|sg| sg.grades.0.iter().enumerate().map(Into::into).collect())
            .unwrap_or_default();
        let students: Vec<StudentGradesResponse> = entities.into_iter().map(Into::into).collect();
        Ok(ModuleGradesResponse {
            assignments,
            students,
        })
    }
}
