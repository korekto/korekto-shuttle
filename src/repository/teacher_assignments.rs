use crate::entities::{Assignment, NewAssignment};

use super::Repository;

impl Repository {
    pub async fn create_assignment(
        &self,
        module_uuid: &str,
        assignment: &NewAssignment,
    ) -> anyhow::Result<Assignment> {
        const QUERY: &str = "INSERT INTO \"assignment\" \
            (module_id, name, start, stop, description, type, subject_url, grader_url, repository_name, factor_percentage, grader_run_url) \
            SELECT m.id, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11 \
            FROM \"module\" m \
            WHERE m.uuid::varchar = $1 \
            RETURNING *, type as \"a_type\"";

        Ok(sqlx::query_as::<_, Assignment>(QUERY)
            .bind(module_uuid)
            .bind(&assignment.name)
            .bind(assignment.start)
            .bind(assignment.stop)
            .bind(&assignment.description)
            .bind(&assignment.a_type)
            .bind(&assignment.subject_url)
            .bind(&assignment.grader_url)
            .bind(&assignment.repository_name)
            .bind(assignment.factor_percentage)
            .bind(&assignment.grader_run_url)
            .fetch_one(&self.pool)
            .await?)
    }
}
