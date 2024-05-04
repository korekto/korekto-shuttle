use crate::entities::InstantGrade;
use crate::service::dtos::{NewGradeRequest, VecInto};
use crate::service::Service;
use time::OffsetDateTime;

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
}
