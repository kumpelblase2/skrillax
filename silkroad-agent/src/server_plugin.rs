use crate::config::GameConfig;
use bevy_app::{App, Plugin, ScheduleRunnerPlugin};
use bevy_ecs_macros::Resource;
use std::ops::Div;
use std::time::Duration;

#[derive(Resource)]
pub(crate) struct ServerId(pub u16);

pub(crate) struct ServerPlugin {
    configuration: GameConfig,
    server_id: u16,
}

impl ServerPlugin {
    pub fn new(config: GameConfig, server_id: u16) -> Self {
        Self {
            configuration: config,
            server_id,
        }
    }
}

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        if self.configuration.desired_ticks > 0 {
            app.add_plugins(ScheduleRunnerPlugin::run_loop(
                Duration::from_secs(1).div(self.configuration.desired_ticks),
            ));
        }

        app.insert_resource(self.configuration.clone())
            .insert_resource(ServerId(self.server_id));
    }
}
