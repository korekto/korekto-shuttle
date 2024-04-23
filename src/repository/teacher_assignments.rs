use crate::entities::{Assignment, NewAssignment};
use tracing::{debug, error};

use super::Repository;

impl Repository {
    pub async fn create_assignment(
        &self,
        module_uuid: &str,
        assignment: &NewAssignment,
    ) -> anyhow::Result<Assignment> {
        const QUERY: &str = "INSERT INTO \"assignment\"
            (module_id, name, start, stop, description, type, subject_url, grader_url, repository_name, factor_percentage, grader_run_url)
            SELECT m.id, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11
            FROM \"module\" m
            WHERE m.uuid::varchar = $1
            RETURNING *, type as \"a_type\", uuid::varchar as \"uuid\"
            ";

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

    pub async fn find_assignment(
        &self,
        module_uuid: &str,
        uuid: &str,
    ) -> anyhow::Result<Assignment> {
        const QUERY: &str = "SELECT
            a.id,
            a.uuid::varchar as \"uuid\",
            a.name,
            a.start,
            a.stop,
            a.description,
            a.type as \"a_type\",
            a.subject_url,
            a.grader_url,
            a.repository_name,
            a.factor_percentage,
            a.grader_run_url
            FROM \"assignment\" a
            JOIN \"module\" m ON m.id = a.module_id
            WHERE a.uuid::varchar = $2
                  AND m.uuid::varchar = $1";

        debug!("Loading assignment: {uuid} (module {module_uuid})");

        Ok(sqlx::query_as::<_, Assignment>(QUERY)
            .bind(module_uuid)
            .bind(uuid)
            .fetch_one(&self.pool)
            .await?)
    }

    pub async fn update_assignment(
        &self,
        module_uuid: &str,
        uuid: &str,
        assignment: &NewAssignment,
    ) -> anyhow::Result<Assignment> {
        const QUERY: &str = "UPDATE \"assignment\" AS a SET
            name = $3,
            start = $4,
            stop = $5,
            description = $6,
            type = $7,
            subject_url = $8,
            grader_url = $9,
            repository_name = $10,
            factor_percentage = $11,
            grader_run_url = $12
            FROM \"module\" AS m
            WHERE m.id = a.module_id
                AND m.uuid::varchar = $1
                AND a.uuid::varchar = $2
            RETURNING a.*, a.type as \"a_type\", a.uuid::varchar as \"uuid\"
        ";

        debug!("Updating assignment: {uuid} (module {module_uuid})");

        Ok(sqlx::query_as::<_, Assignment>(QUERY)
            .bind(module_uuid)
            .bind(uuid)
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

    pub async fn delete_assignments(
        &self,
        module_uuid: &str,
        uuids: &Vec<String>,
    ) -> anyhow::Result<u64> {
        const QUERY: &str = "DELETE FROM \"assignment\" a
            USING \"module\" m
            WHERE m.id = a.module_id
            AND m.uuid::varchar = $1
            AND a.uuid::varchar = ANY($2)
        ";

        match sqlx::query(QUERY)
            .bind(module_uuid)
            .bind(uuids)
            .execute(&self.pool)
            .await
        {
            Err(err) => {
                error!("delete_assignments({:?}): {:?}", uuids, &err);
                Err(err.into())
            }
            Ok(query_result) => Ok(query_result.rows_affected()),
        }
    }

    pub async fn list_assignment_repositories(&self) -> anyhow::Result<Vec<String>> {
        const QUERY: &str = "SELECT repository_name from \"assignment\"";
        Ok(sqlx::query_scalar(QUERY).fetch_all(&self.pool).await?)
    }
}
