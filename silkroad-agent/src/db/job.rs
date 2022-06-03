use sqlx::PgPool;

pub(crate) async fn fetch_job_spread(pool: PgPool, shard: u16) -> (u32, u32) {
    let result: Vec<(i32, i16)> =
        sqlx::query_as("SELECT COUNT(job), job FROM user_servers WHERE job <> 0 AND server_id = $1 GROUP BY job")
            .bind(shard as i32)
            .fetch_all(&pool)
            .await
            .unwrap();

    let thieves = result
        .iter()
        .find(|(_, job)| *job == 1)
        .map(|(count, _)| *count)
        .unwrap_or(0) as u32;
    let hunters = result
        .iter()
        .find(|(_, job)| *job == 2)
        .map(|(count, _)| *count)
        .unwrap_or(0) as u32;

    (hunters, thieves)
}
