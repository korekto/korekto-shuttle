use crate::entities::{ModuleDesc, User, UserModule, UserModuleDesc};
use crate::repository::Repository;
use crate::service::ObfuscatedStr;
use anyhow::Context;
use const_format::formatcp;

const MATCHING_ASSIGNMENTS_CTE: &str = "\
        matching_assignment AS (
          SELECT
            a.id,
            a.uuid::varchar as uuid,
            a.module_id,
            a.name,
            a.description,
            a.start,
            a.stop,
            a.type as a_type,
            a.factor_percentage,
            a.subject_url,
            a.grader_url,
            a.repository_name,
            COALESCE(ua.repository_linked, FALSE) as repo_linked,
            COALESCE(ua.normalized_grade, 0) as grade,
            ua.updated_at
          FROM assignment a
          JOIN user_module um ON um.module_id = a.module_id
		  JOIN \"user\" u ON u.id = um.user_id
          LEFT JOIN user_assignment ua ON ua.assignment_id = a.id AND ua.user_id = u.id
          WHERE u.id = $1
            AND a.hidden_by_teacher IS NOT TRUE
          ORDER BY a.id asc
        )
    ";

impl Repository {
    pub async fn create_user_module(&self, user: &User, module_id: i32) -> anyhow::Result<()> {
        const QUERY: &str = "INSERT INTO user_module
        (user_id, module_id)
        VALUES ($1, $2)";

        sqlx::query(QUERY)
            .bind(user.id)
            .bind(module_id)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .context(format!(
                "[sql] create_user_module(user={user}, module_id={module_id:?})"
            ))
    }

    pub async fn find_module_by_key(
        &self,
        key: &ObfuscatedStr,
    ) -> anyhow::Result<Option<ModuleDesc>> {
        const QUERY: &str = "SELECT
            m.id,
            m.uuid::varchar as uuid,
            m.name,
            m.start,
            m.stop,
            count(a.id) as assignment_count
            FROM \"module\" m
            LEFT JOIN assignment a ON a.module_id = m.id AND a.hidden_by_teacher IS NOT TRUE
            WHERE m.unlock_key = $1
            GROUP BY m.id";

        sqlx::query_as::<_, ModuleDesc>(QUERY)
            .bind(&key.0)
            .fetch_optional(&self.pool)
            .await
            .context(format!("[sql] find_module_by_key(key={key:?})"))
    }

    pub async fn list_modules(&self, user: &User) -> anyhow::Result<Vec<UserModuleDesc>> {
        const QUERY: &str = formatcp!(
            "\
            WITH {MATCHING_ASSIGNMENTS_CTE}
            SELECT
              m.id,
              m.uuid::varchar as uuid,
              m.name,
              m.start,
              m.stop,
              SUM(CASE WHEN ma.repo_linked = TRUE THEN 1 ELSE 0 END)::int linked_repo_count,
              COUNT(ma.id)::int assignment_count,
              COALESCE(SUM(ma.grade * ma.factor_percentage / 100), 0)::real as grade,
              MAX(ma.updated_at) as latest_update
            FROM module m
            INNER JOIN user_module um ON um.module_id = m.id
            LEFT JOIN matching_assignment ma ON ma.module_id = m.id
            WHERE um.user_id = $1
            GROUP BY m.id, m.uuid, m.name, m.start, m.stop
            "
        );

        sqlx::query_as::<_, UserModuleDesc>(QUERY)
            .bind(user.id)
            .fetch_all(&self.pool)
            .await
            .context(format!("[sql] list_modules(user={user})"))
    }

    pub async fn get_module(
        &self,
        user: &User,
        module_uuid: &str,
    ) -> anyhow::Result<Option<UserModule>> {
        const QUERY: &str = formatcp!(
            "\
            WITH {MATCHING_ASSIGNMENTS_CTE}
            SELECT
                m.id,
                m.uuid::varchar as uuid,
                m.name,
                m.description,
                m.start,
                m.stop,
                m.source_url,
                MAX(ma.updated_at) as latest_update,
                coalesce(json_agg(to_jsonb(ma.*) ORDER BY ma.id asc) FILTER (WHERE ma.id IS NOT NULL), '[]'::json) AS assignments
            FROM module m
            INNER JOIN user_module um ON um.module_id = m.id
            LEFT JOIN matching_assignment ma ON ma.module_id = m.id
            WHERE um.user_id = $1
              AND m.uuid::varchar = $2
            GROUP BY m.id
        "
        );

        sqlx::query_as::<_, UserModule>(QUERY)
            .bind(user.id)
            .bind(module_uuid)
            .fetch_optional(&self.pool)
            .await
            .context(format!(
                "[sql] get_module(user={user}, module_uuid={module_uuid:?})"
            ))
    }
}
