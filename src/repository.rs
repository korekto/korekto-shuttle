use sqlx::PgPool;

mod db;
mod find_user_by_id;
mod migration;
mod set_user_admin;
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
