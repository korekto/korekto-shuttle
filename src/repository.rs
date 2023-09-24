use sqlx::PgPool;

mod db;
mod delete_users_by_id;
mod find_user_by_id;
mod find_users;
mod migration;
mod set_user_admin;
mod set_users_teacher;
mod teacher_modules;
mod update_installation_id;
mod upsert_user;

#[derive(Debug, Clone)]
pub struct Repository {
    pool: PgPool,
}

impl Repository {
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
