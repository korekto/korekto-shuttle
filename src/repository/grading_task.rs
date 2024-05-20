//! Grading tasks status follow this graph
//! ```ascii
//! ┌──────┐
//! │queued│ Created or updated when a user triggers it or when an event is received from GitHub
//! └┬─┬───┘
//!  │┌▽───────┐
//!  ││reserved│ Locked for one running instance of Korekto
//!  │└┬─┬─────┘
//!  │ │┌▽──────┐
//!  │ ││ordered│ If the workflow dispatch event have been successfully sent
//!  │ │└┬──────┘
//!  │ │┌▽──────┐
//!  │ ││started│ When the started payload is received
//!  │ │└┬──┬───┘
//! ┌▽─▽─▽┐┌▽─────────┐
//! │error││successful│ When the grading payload is received
//! └─────┘└──────────┘
//! (source https://arthursonzogni.com/Diagon/#GraphDAG)
//! queued -> reserved -> ordered -> started -> successful
//! started -> error
//! queued -> error
//! reserved  -> error
//! ```
//! Reservation is guarantied by optimistic locking inside a transaction

use crate::entities::{GitHubGradingTask, GradingTask, NewGradingTask};
use crate::repository::Repository;
use anyhow::anyhow;
use sqlx::{Executor, Postgres};
use std::fmt;
use time::OffsetDateTime;

#[derive(Debug)]
pub enum Status {
    QUEUED,
    RESERVED,
    ORDERED,
    STARTED,
    ERROR,
    SUCCESSFUL,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Repository {
    pub async fn upsert_grading_task(
        &self,
        task: &NewGradingTask,
    ) -> anyhow::Result<OffsetDateTime> {
        match task {
            NewGradingTask::Internal {
                user_assignment_id,
                user_provider_name,
                repository,
                grader_repository,
            } => {
                self.upsert_grading_task_internal(
                    *user_assignment_id,
                    user_provider_name,
                    repository,
                    grader_repository,
                )
                .await
            }
            NewGradingTask::External {
                assignment_uuid,
                user_uuid,
            } => {
                self.upsert_grading_task_external(assignment_uuid, user_uuid)
                    .await
            }
        }
    }

    async fn upsert_grading_task_internal(
        &self,
        user_assignment_id: i32,
        user_provider_name: &str,
        repository: &str,
        grader_repository: &str,
    ) -> anyhow::Result<OffsetDateTime> {
        const QUERY: &str = "INSERT INTO grading_task
          (user_assignment_id, user_provider_login, status, repository, grader_repository, updated_at)
        VALUES ($1, $2, 'queued', $3, $4, NOW())
        ON CONFLICT (user_assignment_id, user_provider_login, status) DO UPDATE
        SET updated_at= NOW()
        RETURNING updated_at";

        Ok(sqlx::query_scalar(QUERY)
            .bind(user_assignment_id)
            .bind(user_provider_name)
            .bind(repository)
            .bind(grader_repository)
            .fetch_one(&self.pool)
            .await?)
    }

    async fn upsert_grading_task_external(
        &self,
        assignment_uuid: &str,
        user_uuid: &str,
    ) -> anyhow::Result<OffsetDateTime> {
        const QUERY: &str = "INSERT INTO grading_task
          (user_assignment_id, user_provider_login, status, repository, grader_repository, updated_at)
        SELECT ua.id, u.provider_login, 'queued', a.repository_name, a.grader_url, NOW()
        FROM user_assignment ua, \"user\" u, assignment a
        WHERE
          ua.user_id = u.id
          AND ua.assignment_id = a.id
          AND a.uuid::varchar = $1
          AND u.uuid::varchar = $2
        ON CONFLICT (user_assignment_id, user_provider_login, status) DO UPDATE
        SET updated_at = NOW()
        RETURNING updated_at";

        Ok(sqlx::query_scalar(QUERY)
            .bind(assignment_uuid)
            .bind(user_uuid)
            .fetch_one(&self.pool)
            .await?)
    }

    pub async fn get_grading_tasks(
        &self,
        page: i32,
        per_page: i32,
    ) -> anyhow::Result<Vec<GradingTask>> {
        const QUERY: &str = "\
            SELECT
              m.uuid::varchar as module_uuid,
              a.uuid::varchar as assignment_uuid,
              gt.user_provider_login as provider_login,
              gt.status,
              gt.created_at,
              gt.updated_at,
              a.repository_name,
              (count(*) OVER ())::integer as total_count
            FROM grading_task gt, user_assignment ua, assignment a, module m
            WHERE
              gt.user_assignment_id = ua.id
              AND ua.assignment_id = a.id
              AND a.module_id = m.id
            ORDER BY created_at DESC
            LIMIT $1
            OFFSET $2
        ";

        let offset = if page == 1 { 0 } else { (page - 1) * per_page };

        sqlx::query_as::<_, GradingTask>(QUERY)
            .bind(per_page)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|err| anyhow!("get_unparseable_webhooks({page}, {per_page}): {:?}", &err))
    }

    pub async fn reserve_grading_tasks_to_execute<'e, 'c: 'e, E>(
        min_execution_interval_in_secs: i32,
        max_tasks: i32,
        transaction: E,
    ) -> anyhow::Result<Vec<GitHubGradingTask>>
    where
        E: 'e + Executor<'c, Database = Postgres>,
    {
        const QUERY: &str = "\
            WITH max_tasks as (
              SELECT
                gt.id,
                gt.uuid::varchar as uuid,
                ua.uuid::varchar as user_assignment_uuid,
                gt.user_provider_login as provider_login,
                gt.status,
                gt.created_at,
                gt.updated_at,
                gt.version,
                a.repository_name,
                u.installation_id,
                a.grader_url as grader_url
              FROM grading_task gt, user_assignment ua, assignment a, module m, \"user\" u
              WHERE gt.user_assignment_id = ua.id
              AND ua.assignment_id = a.id
              AND a.module_id = m.id
              AND ua.user_id = u.id
              AND (ua.graded_last_at IS NULL OR ua.graded_last_at < NOW() - interval '$1 seconds')
              AND gt.status IN ('queued', 'ordered', 'running')
              ORDER BY gt.created_at ASC
              LIMIT $2
            )
            UPDATE grading_task gt SET status = 'reserved'
            FROM max_tasks mt
            WHERE mt.id = gt.id
            AND mt.status = 'queued'
            RETURNING mt.*
        ";

        sqlx::query_as::<_, GitHubGradingTask>(QUERY)
            .bind(min_execution_interval_in_secs)
            .bind(max_tasks)
            .fetch_all(transaction)
            .await
            .map_err(|err| anyhow!("reserve_grading_tasks_to_execute(min_execution_interval_in_secs={min_execution_interval_in_secs}, max_tasks={max_tasks}): {err:?}"))
    }

    pub async fn update_grading_task_status<'e, 'c: 'e, E>(
        uuid: &str,
        status: &Status,
        message: &Option<String>,
        transaction: E,
    ) -> anyhow::Result<()>
    where
        E: 'e + Executor<'c, Database = Postgres>,
    {
        const QUERY: &str = "\
            UPDATE grading_task
            SET status = $2, message = $3
            WHERE uuid::varchar = $1
        ";

        sqlx::query(QUERY)
            .bind(uuid)
            .bind(status.to_string())
            .bind(message)
            .execute(transaction)
            .await
            .map_err(|err| anyhow!("update_grading_task_status(uuid={uuid}, status={status}, message={message:?}: {err:?}"))?;
        Ok(())
    }

    pub async fn get_grading_tasks_to_execute(
        &self,
        min_execution_interval_in_secs: i32,
        max_tasks: i32,
    ) -> anyhow::Result<Vec<GitHubGradingTask>> {
        const QUERY: &str = "\
            WITH max_tasks as (
              SELECT
                aua.uuid::varchar as user_assignment_uuid,
                gt.user_provider_login as provider_login,
                gt.status,
                gt.created_at,
                gt.updated_at,
                gt.version,
                a.repository_name,
                u.installation_id,
                a.grader_url as grader_url
              FROM grading_task gt, user_assignment ua, assignment a, module m, \"user\" u
              WHERE gt.user_assignment_id = ua.id
              AND ua.assignment_id = a.id
              AND a.module_id = m.id
              AND ua.user_id = u.id
              AND (ua.graded_last_at IS NULL OR ua.graded_last_at < NOW() - interval '$1 seconds')
              AND gt.status IN ('queued', 'ordered', 'running')
              ORDER BY gt.created_at DESC
              LIMIT $2
            )
            SELECT * FROM max_tasks
            WHERE status = 'queued'
        ";

        sqlx::query_as::<_, GitHubGradingTask>(QUERY)
            .bind(min_execution_interval_in_secs)
            .bind(max_tasks)
            .fetch_all(&self.pool)
            .await
            .map_err(|err| anyhow!("get_grading_tasks_to_execute(min_execution_interval_in_secs={min_execution_interval_in_secs}, max_tasks={max_tasks}): {err:?}"))
    }
}
