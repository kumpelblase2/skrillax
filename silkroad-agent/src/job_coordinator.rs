use crate::db::job::fetch_job_spread;
use sqlx::PgPool;

pub(crate) struct JobCoordinator {
    pool: PgPool,
    hunters: u32,
    thieves: u32,
    shard: u16,
}

impl JobCoordinator {
    pub fn new(pool: PgPool, shard: u16) -> Self {
        JobCoordinator {
            pool,
            shard,
            hunters: 0,
            thieves: 0,
        }
    }

    pub async fn load(&mut self) {
        let (hunter_count, thieve_count) = fetch_job_spread(&self.pool, self.shard).await;
        self.hunters = hunter_count;
        self.thieves = thieve_count;
    }

    pub fn spread(&self) -> (u8, u8) {
        let total = self.hunters + self.thieves;
        return if total == 0 {
            (50, 50)
        } else if self.thieves == 0 {
            (100, 0)
        } else if self.hunters == 0 {
            (0, 100)
        } else {
            let hunter_percentage = (self.hunters / total) as u8;
            let thieves_percentage = 100 - hunter_percentage;
            (hunter_percentage, thieves_percentage)
        };
    }
}
