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
//! │error││successful│ When the grading payload is received (these status are virtual, terminal status events delete matching tasks)
//! └─────┘└──────────┘
//! (source https://arthursonzogni.com/Diagon/#GraphDAG)
//! queued -> reserved -> ordered -> started -> successful
//! started -> error
//! queued -> error
//! reserved  -> error
//! ```

use crate::entities::{GitHubGradingTask, GradingTask, NewGradingTask, RawGradingTask};
use crate::repository::Repository;
use anyhow::anyhow;
use serde::Serialize;
use sqlx::{Executor, Postgres};
use std::fmt;
use std::str::FromStr;
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum GradingStatus {
    QUEUED,
    RESERVED,
    ORDERED,
    STARTED,
    ERROR,
    SUCCESSFUL,
}

pub trait Terminal {
    fn is_terminal(&self) -> bool;
}

impl Terminal for GradingStatus {
    fn is_terminal(&self) -> bool {
        match self {
            Self::QUEUED | Self::RESERVED | Self::ORDERED | Self::STARTED => false,
            Self::ERROR | Self::SUCCESSFUL => true,
        }
    }
}

impl FromStr for GradingStatus {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "QUEUED" => Ok(Self::QUEUED),
            "RESERVED" => Ok(Self::RESERVED),
            "ORDERED" => Ok(Self::ORDERED),
            "STARTED" => Ok(Self::STARTED),
            "ERROR" => Ok(Self::ERROR),
            "SUCCESSFUL" => Ok(Self::SUCCESSFUL),
            _ => Err(()),
        }
    }
}

impl fmt::Display for GradingStatus {
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
        VALUES ($1, $2, $3, $4, $5, NOW())
        ON CONFLICT (user_assignment_id, user_provider_login, status) DO UPDATE
        SET updated_at = NOW()
        RETURNING updated_at";

        Ok(sqlx::query_scalar(QUERY)
            .bind(user_assignment_id)
            .bind(user_provider_name)
            .bind(GradingStatus::QUEUED.to_string())
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
        SELECT ua.id, u.provider_login, $3, a.repository_name, a.grader_url, NOW()
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
            .bind(GradingStatus::QUEUED.to_string())
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

    pub async fn reserve_grading_tasks_to_execute_transact<'e, 'c: 'e, E>(
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
                ua.id as user_assignment_id,
                gt.user_provider_login as provider_login,
                gt.status,
                gt.created_at,
                gt.updated_at,
                a.repository_name,
                u.installation_id,
                a.grader_url as grader_url
              FROM grading_task gt, user_assignment ua, assignment a, module m, \"user\" u
              WHERE gt.user_assignment_id = ua.id
              AND ua.assignment_id = a.id
              AND a.module_id = m.id
              AND ua.user_id = u.id
              AND ua.grading_in_progress IS FALSE
              AND (ua.graded_last_at IS NULL OR ua.graded_last_at < NOW() - interval '1 seconds' * $1)
              AND gt.status = ANY ($3)
              ORDER BY gt.created_at ASC
              LIMIT $2
            ),
            grading_task_update as (
              UPDATE grading_task gt SET status = $5
              FROM max_tasks mt
              WHERE mt.id = gt.id
              AND mt.status = $4
              RETURNING mt.*
            )
            UPDATE user_assignment ua SET
              grading_in_progress = TRUE,
              previous_grading_error = NULL
            FROM grading_task_update gtu
            WHERE gtu.user_assignment_id = ua.id
            RETURNING gtu.*
        ";

        sqlx::query_as::<_, GitHubGradingTask>(QUERY)
            .bind(min_execution_interval_in_secs)
            .bind(max_tasks)
            .bind(&[GradingStatus::QUEUED.to_string(), GradingStatus::RESERVED.to_string(), GradingStatus::ORDERED.to_string(), GradingStatus::STARTED.to_string()])
            .bind(GradingStatus::QUEUED.to_string())
            .bind(GradingStatus::RESERVED.to_string())
            .fetch_all(transaction)
            .await
            .map_err(|err| anyhow!("reserve_grading_tasks_to_execute(min_execution_interval_in_secs={min_execution_interval_in_secs}, max_tasks={max_tasks}): {err:?}"))
    }

    pub async fn delete_grading_task(
        &self,
        uuid: &str,
        error_message: Option<String>,
    ) -> anyhow::Result<RawGradingTask> {
        Self::delete_grading_task_transact(uuid, error_message, &self.pool).await
    }

    pub async fn delete_grading_task_transact<'e, 'c: 'e, E>(
        uuid: &str,
        error_message: Option<String>,
        transaction: E,
    ) -> anyhow::Result<RawGradingTask>
    where
        E: 'e + Executor<'c, Database = Postgres>,
    {
        const QUERY: &str = "\
            WITH deleted_grading_task AS (
                DELETE FROM grading_task WHERE uuid::varchar = $1
                RETURNING *, uuid::varchar as uuid
            )
            UPDATE user_assignment ua
            SET
              grading_in_progress = FALSE,
              graded_last_at = NOW(),
              previous_grading_error = $2,
              running_grading_metadata = NULL
            FROM deleted_grading_task dgt
            WHERE dgt.user_assignment_id = ua.id
            RETURNING dgt.*
        ";

        sqlx::query_as::<_, RawGradingTask>(QUERY)
            .bind(uuid)
            .bind(error_message)
            .fetch_one(transaction)
            .await
            .map_err(|err| anyhow!("delete_grading_task_transact(uuid={uuid}): {err:?}"))
    }

    pub async fn update_grading_task_non_terminal_status(
        &self,
        uuid: &str,
        status: &GradingStatus,
    ) -> anyhow::Result<RawGradingTask> {
        Self::update_grading_task_non_terminal_status_transact(uuid, status, &self.pool).await
    }

    pub async fn update_grading_task_non_terminal_status_transact<'e, 'c: 'e, E>(
        uuid: &str,
        status: &GradingStatus,
        transaction: E,
    ) -> anyhow::Result<RawGradingTask>
    where
        E: 'e + Executor<'c, Database = Postgres>,
    {
        const QUERY: &str = "\
            UPDATE grading_task
            SET status = $2
            WHERE uuid::varchar = $1
            RETURNING *, uuid::varchar as uuid
        ";

        if status.is_terminal() {
            Err(anyhow!(
                "Cannot update grading_task with given terminal status"
            ))?;
        }

        sqlx::query_as::<_, RawGradingTask>(QUERY)
            .bind(uuid)
            .bind(status.to_string())
            .fetch_one(transaction)
            .await
            .map_err(|err| anyhow!("update_grading_task_non_terminal_status_transact(uuid={uuid}, status={status}): {err:?}"))
    }

    pub async fn timeout_grading_tasks(
        &self,
        status: &GradingStatus,
        min_creation_interval_in_secs: i32,
    ) -> anyhow::Result<i32> {
        const QUERY: &str = "\
            WITH deleted_grading_task AS (
                DELETE FROM grading_task
                WHERE
                  status = $1
                  AND created_at < NOW() - interval '1 seconds' * $2
                RETURNING *
            ), updated_user_assignment AS (
                UPDATE user_assignment ua
                SET
                  grading_in_progress = FALSE,
                  graded_last_at = NOW(),
                  previous_grading_error = 'Status ' || $1 || ' timed out after ' || $2 || ' secs',
                  running_grading_metadata = NULL
                FROM deleted_grading_task dgt
                WHERE dgt.user_assignment_id = ua.id
                RETURNING ua.*
            )
            SELECT count(dgt.*)::integer
            FROM deleted_grading_task dgt
            LEFT JOIN updated_user_assignment uua ON uua.id = dgt.user_assignment_id
        ";

        sqlx::query_scalar(QUERY)
            .bind(status.to_string())
            .bind(min_creation_interval_in_secs)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| anyhow!("timeout_grading_tasks(status={status}, min_creation_interval_in_secs={min_creation_interval_in_secs}): {err:?}"))
    }
}
