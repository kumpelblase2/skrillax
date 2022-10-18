use sqlx::PgPool;

pub(crate) async fn fetch_job_spread(pool: PgPool, shard: u16) -> (u32, u32) {
    let result = sqlx::query!(
        "SELECT COUNT(job) as \"count!\", job FROM user_servers WHERE job <> 0 AND server_id = $1 GROUP BY job",
        shard as i32
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    let thieves = result.iter().find(|res| res.job == 1).map(|res| res.count).unwrap_or(0) as u32;
    let hunters = result.iter().find(|res| res.job == 2).map(|res| res.count).unwrap_or(0) as u32;

    (hunters, thieves)
}
