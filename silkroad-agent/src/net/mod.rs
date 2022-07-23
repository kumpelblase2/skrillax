mod login;
mod net;

use crate::net::login::login;
use crate::net::net::{accept, connected, disconnected, receive};
use bevy_app::{App, CoreStage, Plugin};
use silkroad_network::server::SilkroadServer;

pub struct NetworkPlugin {
    server: SilkroadServer,
}

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.server.clone())
            .add_system_to_stage(CoreStage::PreUpdate, accept)
            .add_system_to_stage(CoreStage::PreUpdate, receive)
            .add_system_to_stage(CoreStage::PreUpdate, disconnected)
            .add_system_to_stage(CoreStage::PreUpdate, connected)
            .add_system_to_stage(CoreStage::PreUpdate, login);
    }
}

impl NetworkPlugin {
    pub fn new(server: SilkroadServer) -> Self {
        Self { server }
    }
}
