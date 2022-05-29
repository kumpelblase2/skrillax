use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::time::Duration;
use tokio::time::Instant;

#[derive(sqlx::FromRow, Clone)]
pub(crate) struct News {
    pub(crate) title: String,
    pub(crate) body: String,
    pub(crate) date: DateTime<Utc>,
}

pub(crate) struct NewsCacheAsync {
    pool: PgPool,
    cache_time: Duration,
    news: Vec<News>,
    last_cache_time: Instant,
}

impl NewsCacheAsync {
    pub async fn new(pool: PgPool, cache_time: Duration) -> Self {
        let news = Self::load_available_news(&pool).await;

        NewsCacheAsync {
            pool,
            news,
            cache_time,
            last_cache_time: Instant::now(),
        }
    }

    pub async fn get_news(&mut self) -> &Vec<News> {
        if self.should_refresh() {
            self.news = Self::load_available_news(&self.pool).await;
        }
        &self.news
    }

    fn should_refresh(&self) -> bool {
        Instant::now().duration_since(self.last_cache_time) > self.cache_time
    }

    pub async fn load_available_news(pool: &PgPool) -> Vec<News> {
        sqlx::query_as("SELECT title, body, date FROM news WHERE visible = true ORDER BY date ASC")
            .fetch_all(pool)
            .await
            .unwrap()
    }
}
