use super::{ServerJobDistribution, ServerUser, UserReadError};
use sqlx::PgPool;

pub(super) async fn load_server_user(
    pool: &PgPool,
    id: u32,
    server_id: u16,
) -> Result<Option<ServerUser>, UserReadError> {
    let Some(user) = sqlx::query!("SELECT username FROM users WHERE id = $1", id as i32)
        .fetch_optional(pool)
        .await?
    else {
        return Ok(None);
    };

    let server_data = sqlx::query!(
        "SELECT job, premium_type, premium_end FROM user_servers WHERE user_id = $1 AND server_id = $2",
        id as i32,
        server_id as i32,
    )
    .fetch_optional(pool)
    .await?;

    let server_user = match server_data {
        Some(data) => ServerUser {
            id: id as i32,
            username: user.username,
            job: data.job,
            premium_type: data.premium_type,
            premium_end: data.premium_end,
        },
        None => {
            sqlx::query!(
                "INSERT INTO user_servers(user_id, server_id) values($1, $2)",
                id as i32,
                server_id as i32,
            )
            .execute(pool)
            .await?;

            ServerUser {
                id: id as i32,
                username: user.username,
                job: 0,
                premium_type: 0,
                premium_end: None,
            }
        },
    };

    Ok(Some(server_user))
}

pub(super) async fn load_job_distribution(
    pool: &PgPool,
    server_id: u16,
) -> Result<ServerJobDistribution, UserReadError> {
    let result = sqlx::query!(
        "SELECT COUNT(job) as \"count!\", job FROM user_servers WHERE job <> 0 AND server_id = $1 GROUP BY job",
        server_id as i32,
    )
    .fetch_all(pool)
    .await?;

    let thieves = result.iter().find(|row| row.job == 1).map(|row| row.count).unwrap_or(0) as u32;
    let hunters = result.iter().find(|row| row.job == 2).map(|row| row.count).unwrap_or(0) as u32;

    Ok(ServerJobDistribution { hunters, thieves })
}
