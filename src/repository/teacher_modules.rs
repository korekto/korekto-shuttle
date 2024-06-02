use anyhow::anyhow;
use tracing::{debug, error};

use crate::entities;
use crate::entities::{Module, NewModule, StudentGrades, User};

use super::Repository;

impl Repository {
    pub async fn find_modules(&self, teacher: &User) -> anyhow::Result<Vec<entities::ModuleDesc>> {
        const QUERY: &str = "
            SELECT
              m.id,
              m.uuid::varchar as uuid,
              m.name,
              m.start,
              m.stop,
              count(a.id) as assignment_count
            FROM module m
            JOIN teacher_module tm ON tm.module_id = m.id
            LEFT JOIN assignment a ON a.module_id = m.id
            WHERE tm.teacher_id = $1
            GROUP BY m.id, m.uuid, m.name, m.start, m.stop";

        sqlx::query_as::<_, entities::ModuleDesc>(QUERY)
            .bind(teacher.id)
            .fetch_all(&self.pool)
            .await
            .map_err(|err| anyhow!("find_modules(): {:?}", &err))
    }

    pub async fn create_module(
        &self,
        module: &NewModule,
        teacher: &User,
    ) -> anyhow::Result<Module> {
        const MODULE_QUERY: &str = "
            INSERT INTO module
              (name, description, start, stop, unlock_key, source_url)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING
              id,
              uuid::varchar as uuid,
              name,
              description,
              start,
              stop,
              unlock_key,
              source_url,
              '[]'::jsonb AS assignments
            ";
        const TEACHER_RELATION_QUERY: &str = "
            INSERT INTO teacher_module (module_id, teacher_id)
            VALUES ($1, $2)
            ";

        let mut transaction = self.start_transaction().await?;

        let row = sqlx::query_as::<_, Module>(MODULE_QUERY)
            .bind(&module.name)
            .bind(&module.description)
            .bind(module.start)
            .bind(module.stop)
            .bind(&module.unlock_key)
            .bind(&module.source_url)
            .fetch_one(&mut *transaction)
            .await?;

        sqlx::query(TEACHER_RELATION_QUERY)
            .bind(row.id)
            .bind(teacher.id)
            .execute(&mut *transaction)
            .await?;

        transaction.commit().await?;

        Ok(row)
    }

    pub async fn find_module(&self, uuid: &str, teacher: &User) -> anyhow::Result<Module> {
        const QUERY: &str = "
            WITH ORDERED_ASSIGNMENTS AS (
                SELECT *
                FROM assignment
                ORDER BY id asc
            )
            SELECT
                m.id,
                m.uuid::varchar as uuid,
                m.name,
                m.description,
                m.start,
                m.stop,
                m.unlock_key,
                m.source_url,
                a.assignments
            FROM module m
            JOIN teacher_module tm ON tm.module_id = m.id
            LEFT JOIN LATERAL (
                SELECT
                    coalesce(jsonb_agg(to_jsonb(a.*) || jsonb_build_object('a_type', a.type)), '[]'::jsonb) AS assignments
                FROM ORDERED_ASSIGNMENTS A
                WHERE m.id = A.module_id
            ) AS a ON TRUE
            WHERE m.uuid::varchar = $1
            AND tm.teacher_id = $2";

        debug!("Loading module: {uuid}");

        let row = sqlx::query_as::<_, Module>(QUERY)
            .bind(uuid)
            .bind(teacher.id)
            .fetch_one(&self.pool)
            .await?;

        Ok(row)
    }

    pub async fn update_module(
        &self,
        uuid: &str,
        module: &NewModule,
        teacher: &User,
    ) -> anyhow::Result<Module> {
        const QUERY: &str = "\
            WITH ORDERED_ASSIGNMENTS AS (
                SELECT *
                FROM ASSIGNMENT
                ORDER BY id asc
            )
            UPDATE module AS m SET
                name = $2,
                description = $3,
                start = $4,
                stop = $5,
                unlock_key = $6,
                source_url = $7
            FROM module AS m2
            JOIN teacher_module tm ON tm.module_id = m2.id
            LEFT JOIN LATERAL (
                SELECT
                    coalesce(jsonb_agg(to_jsonb(a.*) || jsonb_build_object('a_type', a.type)), '[]'::jsonb) AS assignments
                FROM ORDERED_ASSIGNMENTS A
                WHERE m2.id = A.module_id
            ) AS a ON TRUE
            WHERE m.uuid::varchar = $1
                AND m2.uuid::varchar = $1
                AND m.id = m2.id
                AND tm.teacher_id = $8
            RETURNING m.*,
                m.uuid::varchar as uuid,
                a.assignments
        ";

        debug!("Updating module: {uuid}");

        let row = sqlx::query_as::<_, Module>(QUERY)
            .bind(uuid)
            .bind(&module.name)
            .bind(&module.description)
            .bind(module.start)
            .bind(module.stop)
            .bind(&module.unlock_key)
            .bind(&module.source_url)
            .bind(teacher.id)
            .fetch_one(&self.pool)
            .await?;

        Ok(row)
    }

    pub async fn delete_modules(&self, uuids: &Vec<String>, teacher: &User) -> anyhow::Result<u64> {
        const QUERY: &str = "
            DELETE FROM module m
            USING teacher_module tm
            WHERE tm.module_id = m.id
              AND m.uuid::varchar = ANY($1)
              AND tm.teacher_id = $2
        ";

        match sqlx::query(QUERY)
            .bind(uuids)
            .bind(teacher.id)
            .execute(&self.pool)
            .await
        {
            Err(err) => {
                error!("delete_modules({:?}): {:?}", uuids, &err);
                Err(err.into())
            }
            Ok(query_result) => Ok(query_result.rows_affected()),
        }
    }

    pub async fn get_grades(
        &self,
        uuid: &str,
        teacher: &User,
    ) -> anyhow::Result<Vec<StudentGrades>> {
        const QUERY: &str = "\
            WITH enhanced_assignment AS (
                SELECT
                  a.id,
                  a.module_id,
                  a.type,
                  a.name,
                  a.description,
                  a.factor_percentage,
                  COALESCE(ua.normalized_grade, 0) as grade,
                  ua.user_id
                FROM assignment a
                LEFT JOIN user_assignment ua ON ua.assignment_id = a.id
                ORDER BY a.id
            )
            SELECT
              u.first_name,
              u.last_name,
              u.school_email,
              json_agg(
                json_build_object(
                  'type', ea.type,
                  'name', ea.name,
                  'description', ea.description,
                  'factor_percentage', ea.factor_percentage,
                  'grade', ea.grade
                ) ORDER BY ea.id ASC
              ) as grades,
              COALESCE(SUM(ea.grade * ea.factor_percentage / 100), 0)::real as total
            FROM \"user\" u
            JOIN user_module um ON um.user_id = u.id
            JOIN module m ON m.id = um.module_id
            JOIN enhanced_assignment ea ON ea.module_id = m.id AND (ea.user_id = u.id OR ea.user_id IS NULL)
            WHERE m.uuid::varchar = $1
            GROUP BY u.id
        ";

        sqlx::query_as::<_, StudentGrades>(QUERY)
            .bind(uuid)
            .bind(teacher.id)
            .fetch_all(&self.pool)
            .await
            .map_err(|err| anyhow!("get_grades(uuid={uuid}, teacher={teacher}): {:?}", &err))
    }
}
