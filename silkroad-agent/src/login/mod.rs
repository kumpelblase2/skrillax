use crate::login::job_distribution::{update_job_distribution, JobDistribution};
use crate::LoginQueue;
use bevy_app::{App, CoreStage, Plugin};
use charselect::charselect;

pub mod character_loader;
pub mod charselect;
pub mod job_distribution;
pub mod web;

pub(crate) struct LoginPlugin {
    login_queue: LoginQueue,
}

impl Plugin for LoginPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.login_queue.clone())
            .insert_resource(JobDistribution::default())
            .add_system_to_stage(CoreStage::PostUpdate, update_job_distribution)
            .add_system(charselect);
    }
}

impl LoginPlugin {
    pub fn new(login_queue: LoginQueue) -> Self {
        Self { login_queue }
    }
}
