use anyhow::anyhow;
use sqlx::types::Json;
use time::OffsetDateTime;
use tracing::{debug, error};

use crate::entities;
use crate::entities::{EmbeddedAssignmentDesc, NewModule};

use super::Repository;

#[derive(sqlx::FromRow)]
struct JsonedIntermediateModule {
    pub id: i32,
    pub uuid: String,
    pub name: String,
    pub start: OffsetDateTime,
    pub stop: OffsetDateTime,
    pub unlock_key: String,
    pub assignments: Json<Vec<EmbeddedAssignmentDesc>>,
}

impl Repository {
    pub async fn find_modules(&self) -> anyhow::Result<Vec<entities::ModuleDesc>> {
        const QUERY: &str = "SELECT
            m.id,
            m.uuid::varchar as uuid,
            m.name,
            m.start,
            m.stop,
            count(a.id) as assignment_count
            FROM \"module\" m
            LEFT JOIN assignment a ON a.module_id = m.id
            GROUP BY m.id, m.uuid, m.name, m.start, m.stop";

        sqlx::query_as::<_, entities::ModuleDesc>(QUERY)
            .fetch_all(&self.pool)
            .await
            .map_err(|err| anyhow!("find_modules(): {:?}", &err))
    }

    pub async fn create_module(&self, module: &NewModule) -> anyhow::Result<entities::Module> {
        const QUERY: &str = "INSERT INTO \"module\"
            (name, start, stop, unlock_key)
            VALUES ($1, $2, $3, $4)
            RETURNING uuid::varchar";

        let uuid: String = sqlx::query_scalar(QUERY)
            .bind(&module.name)
            .bind(module.start)
            .bind(module.stop)
            .bind(&module.unlock_key)
            .fetch_one(&self.pool)
            .await?;
        self.find_module(&uuid).await
    }

    pub async fn find_module(&self, uuid: &str) -> anyhow::Result<entities::Module> {
        const QUERY: &str = "
            WITH ORDERED_ASSIGNMENTS AS (
                SELECT *
                FROM ASSIGNMENT
                ORDER BY id asc
            )
            SELECT
                m.id,
                m.uuid::varchar as \"uuid\",
                m.name,
                m.start,
                m.stop,
                m.unlock_key,
                a.assignments
            FROM \"module\" m
            LEFT JOIN LATERAL (
                SELECT
                    coalesce(jsonb_agg(to_jsonb(a.*) || jsonb_build_object('a_type', a.type, 'id', a.uuid)), '[]'::jsonb) AS assignments
                FROM ORDERED_ASSIGNMENTS A
                WHERE m.id = A.module_id
            ) AS a ON TRUE
            WHERE m.uuid::varchar = $1";

        debug!("Loading module: {uuid}");

        let row = sqlx::query_as::<_, JsonedIntermediateModule>(QUERY)
            .bind(uuid)
            .fetch_one(&self.pool)
            .await?;

        Ok(row.into())
    }

    pub async fn update_module(
        &self,
        uuid: &str,
        module: &NewModule,
    ) -> anyhow::Result<entities::Module> {
        const QUERY: &str = "\
            WITH ORDERED_ASSIGNMENTS AS (
                SELECT *
                FROM ASSIGNMENT
                ORDER BY id asc
            )
            UPDATE \"module\" AS m SET
                name = $2,
                start = $3,
                stop = $4,
                unlock_key = $5
            FROM \"module\" AS m2
            LEFT JOIN LATERAL (
                SELECT
                    coalesce(jsonb_agg(to_jsonb(a.*) || jsonb_build_object('a_type', a.type, 'id', a.uuid)), '[]'::jsonb) AS assignments
                FROM ORDERED_ASSIGNMENTS A
                WHERE m2.id = A.module_id
            ) AS a ON TRUE
            WHERE m.uuid::varchar = $1
                AND m2.uuid::varchar = $1
            RETURNING m.id,
                m.uuid::varchar as \"uuid\",
                m.name,
                m.start,
                m.stop,
                m.unlock_key,
                a.assignments
        ";

        debug!("Updating module: {uuid}");

        let row = sqlx::query_as::<_, JsonedIntermediateModule>(QUERY)
            .bind(uuid)
            .bind(&module.name)
            .bind(module.start)
            .bind(module.stop)
            .bind(&module.unlock_key)
            .fetch_one(&self.pool)
            .await?;

        Ok(row.into())
    }

    pub async fn delete_modules(&self, uuids: &Vec<String>) -> anyhow::Result<u64> {
        const QUERY: &str = "DELETE FROM \"module\" WHERE uuid::varchar = ANY($1)";

        match sqlx::query(QUERY).bind(uuids).execute(&self.pool).await {
            Err(err) => {
                error!("delete_modules({:?}): {:?}", uuids, &err);
                Err(err.into())
            }
            Ok(query_result) => Ok(query_result.rows_affected()),
        }
    }
}

impl From<JsonedIntermediateModule> for entities::Module {
    fn from(value: JsonedIntermediateModule) -> Self {
        Self {
            id: value.id,
            uuid: value.uuid,
            name: value.name,
            start: value.start,
            stop: value.stop,
            unlock_key: value.unlock_key,
            assignments: value.assignments.0,
        }
    }
}
