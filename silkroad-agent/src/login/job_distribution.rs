use crate::db::user::ServerUser;
use crate::ext::DbPool;
use crate::server_plugin::ServerId;
use crate::tasks::TaskCreator;
use bevy_ecs::prelude::*;
use bevy_time::{Time, Timer, TimerMode};
use std::time::Duration;
use tokio::sync::oneshot::error::TryRecvError;
use tokio::sync::oneshot::Receiver;

const REFRESH_INTERVAL: u64 = 60 * 60;

#[derive(Resource)]
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
            refresh_timer: Timer::new(Duration::from_secs(timer_duration), TimerMode::Repeating),
            refresh_result: None,
        }
    }

    pub fn spread(&self) -> (u8, u8) {
        let total = self.hunters + self.thieves;
        if total == 0 {
            (50, 50)
        } else if self.thieves == 0 {
            (100, 0)
        } else if self.hunters == 0 {
            (0, 100)
        } else {
            let hunter_percentage = (self.hunters / total) as u8;
            let thieves_percentage = 100 - hunter_percentage;
            (hunter_percentage, thieves_percentage)
        }
    }
}

impl Default for JobDistribution {
    fn default() -> Self {
        Self::new(REFRESH_INTERVAL)
    }
}

pub(crate) fn update_job_distribution(
    pool: Res<DbPool>,
    server_id: Res<ServerId>,
    time: Res<Time>,
    task_runtime: Res<TaskCreator>,
    mut job: ResMut<JobDistribution>,
) {
    if job.refresh_result.is_none() {
        job.refresh_timer.tick(time.delta());
        if job.refresh_timer.just_finished() {
            let pool = pool.clone();
            let server_id = server_id.0;
            let receiver = task_runtime.create_task(ServerUser::fetch_job_distribution(server_id, pool));
            job.refresh_result = Some(receiver);
        }
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
