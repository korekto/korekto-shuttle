use sqlx::{PgPool, Postgres, Transaction};

mod db;
mod delete_users_by_id;
mod error;
mod find_user;
mod find_users;
pub mod grading_task;
mod migration;
mod set_user_admin;
mod set_users_teacher;
mod teacher_assignments;
mod teacher_modules;
mod unparseable_webhook;
mod update_installation_id;
mod upsert_user;
mod user_assignments;
mod user_modules;

pub type PgTransaction<'a> = Transaction<'a, Postgres>;

#[derive(Debug, Clone)]
pub struct Repository {
    pub pool: PgPool,
}

impl Repository {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn start_transaction(&self) -> Result<PgTransaction, sqlx::Error> {
        self.pool.begin().await
    }
}
