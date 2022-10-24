use crate::config::GameConfig;
use crate::event::{ClientConnectedEvent, ClientDisconnectedEvent};
use crate::GameSettings;
use bevy_app::{App, Plugin, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use std::ops::Div;
use std::time::Duration;

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
        let settings: GameSettings = self.configuration.clone().into();

        if settings.desired_ticks > 0 {
            app.insert_resource(ScheduleRunnerSettings::run_loop(
                Duration::from_secs(1).div(settings.desired_ticks),
            ))
            .add_plugin(ScheduleRunnerPlugin);
        }

        app.insert_resource(settings)
            .insert_resource(ServerId(self.server_id))
            .add_event::<ClientDisconnectedEvent>()
            .add_event::<ClientConnectedEvent>();
    }
}
