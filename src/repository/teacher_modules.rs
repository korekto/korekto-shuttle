use sqlx::types::Json;
use time::OffsetDateTime;
use tracing::debug;

use crate::entities;
use crate::entities::{EmbeddedAssignmentDesc, NewModule};

use super::Repository;

impl Repository {
    pub async fn find_modules(&self) -> anyhow::Result<Vec<entities::ModuleDesc>> {
        const QUERY: &str = "SELECT \
            m.uuid::varchar as id, \
            m.name, \
            m.start, \
            m.stop, \
            count(a.id) as assignment_count
            FROM \"module\" m \
            LEFT JOIN assignment a ON a.module_id = m.id \
            GROUP BY m.uuid, m.name, m.start, m.stop";

        Ok(sqlx::query_as::<_, entities::ModuleDesc>(QUERY)
            .fetch_all(&self.pool)
            .await?)
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
        #[derive(sqlx::FromRow)]
        struct JsonedIntermediateModule {
            pub id: String,
            pub name: String,
            pub start: OffsetDateTime,
            pub stop: OffsetDateTime,
            pub unlock_key: String,
            pub assignments: Json<Vec<EmbeddedAssignmentDesc>>,
        }

        const QUERY: &str = "SELECT \
            m.uuid::varchar as \"id\", \
            m.name, \
            m.start, \
            m.stop, \
            m.unlock_key, \
            a.assignments \
            FROM \"module\" m \
            LEFT JOIN LATERAL (
                SELECT
                    coalesce(json_agg(a.*), '[]'::json) AS assignments
                FROM ASSIGNMENT A
                WHERE m.id = A.module_id
            ) AS a ON TRUE
            WHERE m.uuid::varchar = $1";

        debug!("Loading module: {uuid}");

        let row = sqlx::query_as::<_, JsonedIntermediateModule>(QUERY)
            .bind(uuid)
            .fetch_one(&self.pool)
            .await?;

        Ok(entities::Module {
            id: row.id,
            name: row.name,
            start: row.start,
            stop: row.stop,
            unlock_key: row.unlock_key,
            assignments: row.assignments.0,
        })
    }
}
