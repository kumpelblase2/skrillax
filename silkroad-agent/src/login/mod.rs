use crate::login::charselect::{handle_auth, handle_join, handle_list_request};
use crate::login::job_distribution::{update_job_distribution, JobDistribution};
use crate::login::jobs::{
    handle_character_create, handle_character_delete, handle_character_list_received, handle_character_name_check,
    handle_character_restore,
};
use crate::LoginQueue;
use bevy_app::{App, CoreStage, Plugin};

pub mod character_loader;
mod charselect;
mod components;
pub mod job_distribution;
mod jobs;
pub mod web;

pub(crate) use components::*;

pub(crate) struct LoginPlugin {
    login_queue: LoginQueue,
}

impl Plugin for LoginPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.login_queue.clone())
            .insert_resource(JobDistribution::default())
            .add_system_to_stage(CoreStage::PostUpdate, update_job_distribution)
            .add_system(handle_character_create)
            .add_system(handle_character_restore)
            .add_system(handle_character_delete)
            .add_system(handle_character_name_check)
            .add_system(handle_character_list_received)
            .add_system(handle_join)
            .add_system(handle_auth)
            .add_system(handle_list_request);
    }
}

impl LoginPlugin {
    pub fn new(login_queue: LoginQueue) -> Self {
        Self { login_queue }
    }
}
