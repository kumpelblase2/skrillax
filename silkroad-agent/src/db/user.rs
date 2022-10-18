use chrono::{DateTime, Utc};
use sqlx::{Error, PgPool};

#[derive(sqlx::FromRow)]
pub(crate) struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub passcode: Option<String>,
    pub invalid_passcode_count: i32,
}

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

pub(crate) async fn fetch_user(pool: &PgPool, id: u32) -> Result<Option<User>, Error> {
    sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id as i32)
        .fetch_optional(pool)
        .await
}

pub(crate) async fn fetch_server_user(pool: &PgPool, id: u32, server: u16) -> Result<Option<ServerUser>, Error> {
    sqlx::query_as!(
        ServerUser,
        "SELECT users.id, users.username, user_servers.job, user_servers.premium_type, user_servers.premium_end FROM users LEFT JOIN user_servers on users.id = user_servers.user_id WHERE id = $1 and server_id = $2",
        id as i32,
        server as i32
    )
            .fetch_optional(pool)
            .await
}
