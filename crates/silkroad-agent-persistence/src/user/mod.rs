mod postgres;

use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Clone)]
pub struct UserPersistence {
    pool: PgPool,
}

impl UserPersistence {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn server_user(&self, id: u32, server_id: u16) -> Result<Option<ServerUser>, UserReadError> {
        postgres::load_server_user(&self.pool, id, server_id).await
    }

    pub async fn job_distribution(&self, server_id: u16) -> Result<ServerJobDistribution, UserReadError> {
        postgres::load_job_distribution(&self.pool, server_id).await
    }
}

#[derive(Clone, Debug)]
pub struct ServerUser {
    pub id: i32,
    pub username: String,
    pub job: i16,
    pub premium_type: i16,
    pub premium_end: Option<DateTime<Utc>>,
}

impl PartialEq for ServerUser {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.username == other.username
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServerJobDistribution {
    pub hunters: u32,
    pub thieves: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum UserReadError {
    #[error("user database query failed: {0}")]
    Database(#[from] sqlx::Error),
}
