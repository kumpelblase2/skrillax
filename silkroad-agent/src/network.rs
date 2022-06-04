use crate::sys::in_game::in_game;
use crate::sys::net::{accept, connected, disconnected, receive};
use bevy_app::{App, CoreStage, Plugin};
use silkroad_network::server::SilkroadServer;

pub(crate) struct NetworkPlugin {
    server: SilkroadServer,
}

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.server.clone())
            .add_system_to_stage(CoreStage::PreUpdate, accept)
            .add_system_to_stage(CoreStage::PreUpdate, receive)
            .add_system_to_stage(CoreStage::PreUpdate, disconnected)
            .add_system_to_stage(CoreStage::PreUpdate, connected)
            .add_system(in_game);
    }
}

impl NetworkPlugin {
    pub fn new(server: SilkroadServer) -> Self {
        Self { server }
    }
}
