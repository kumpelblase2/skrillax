use chrono::{DateTime, Utc};
use sqlx::{Error, PgPool};
use std::borrow::Borrow;

#[derive(sqlx::FromRow, Clone)]
pub(crate) struct ServerUser {
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

impl ServerUser {
    pub async fn fetch<T: Borrow<PgPool>>(id: u32, server: u16, pool: T) -> Result<Option<ServerUser>, Error> {
        sqlx::query_as!(ServerUser,
            "SELECT users.id, users.username, user_servers.job, user_servers.premium_type, user_servers.premium_end FROM users LEFT JOIN user_servers on users.id = user_servers.user_id WHERE id = $1 and server_id = $2",
            id as i32,
            server as i32
        )
                .fetch_optional(pool.borrow())
                .await
    }

    pub async fn fetch_job_distribution<T: Borrow<PgPool>>(shard: u16, pool: T) -> (u32, u32) {
        let result = sqlx::query!(
            "SELECT COUNT(job) as \"count!\", job FROM user_servers WHERE job <> 0 AND server_id = $1 GROUP BY job",
            shard as i32
        )
        .fetch_all(pool.borrow())
        .await
        .unwrap();

        let thieves = result.iter().find(|res| res.job == 1).map(|res| res.count).unwrap_or(0) as u32;
        let hunters = result.iter().find(|res| res.job == 2).map(|res| res.count).unwrap_or(0) as u32;

        (hunters, thieves)
    }
}
