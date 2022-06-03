use crate::db::job::fetch_job_spread;
use crate::ext::AsyncTaskCreate;
use crate::server_plugin::ServerId;
use bevy_core::prelude::*;
use bevy_ecs::prelude::*;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::oneshot::error::TryRecvError;
use tokio::sync::oneshot::Receiver;

const REFRESH_INTERVAL: u64 = 60 * 60;

pub(crate) struct JobDistribution {
    hunters: u32,
    thieves: u32,
    refresh_timer: Timer,
    refresh_result: Option<Receiver<(u32, u32)>>,
}

impl JobDistribution {
    pub fn new(timer_duration: u64) -> Self {
        Self {
            hunters: 0,
            thieves: 0,
            refresh_timer: Timer::new(Duration::from_secs(timer_duration), true),
            refresh_result: None,
        }
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

impl Default for JobDistribution {
    fn default() -> Self {
        Self::new(REFRESH_INTERVAL)
    }
}

pub(crate) fn update_job_distribution(
    pool: Res<PgPool>,
    server_id: Res<ServerId>,
    time: Res<Time>,
    task_runtime: Res<Arc<Runtime>>,
    mut job: ResMut<JobDistribution>,
) {
    job.refresh_timer.tick(time.delta());
    if job.refresh_timer.just_finished() {
        let pool = pool.clone();
        let server_id = server_id.0;
        let receiver = task_runtime.create_task(fetch_job_spread(pool, server_id));
        job.refresh_result = Some(receiver);
    }

    if let Some(receive) = job.refresh_result.as_mut() {
        match receive.try_recv() {
            Ok((hunter, thief)) => {
                job.hunters = hunter;
                job.thieves = thief;
                job.refresh_result = None;
            },
            Err(TryRecvError::Empty) => {},
            _ => {
                job.refresh_result = None;
            },
        }
    }
}
