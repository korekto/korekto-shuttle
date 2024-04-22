use sqlx::PgPool;

mod db;
mod delete_users_by_id;
mod find_user;
mod find_users;
mod grading_task;
mod migration;
mod set_user_admin;
mod set_users_teacher;
mod teacher_assignments;
mod teacher_modules;
mod unparseable_webhook;
mod update_installation_id;
mod upsert_user;
mod user_asignments;
mod user_modules;

#[derive(Debug, Clone)]
pub struct Repository {
    pool: PgPool,
}

impl Repository {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
