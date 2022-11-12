mod login;
mod net;

use crate::event::{ClientConnectedEvent, ClientDisconnectedEvent};
use crate::net::login::login;
use crate::net::net::{accept, connected, disconnected, receive};
use bevy_app::{App, CoreStage, Plugin};
use bevy_ecs::prelude::*;
use silkroad_network::server::SilkroadServer;

pub struct NetworkPlugin {
    server: SilkroadServer,
}

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.server.clone())
            .add_system_to_stage(CoreStage::PreUpdate, accept)
            .add_system_to_stage(CoreStage::PreUpdate, receive.before(disconnected))
            .add_system_to_stage(CoreStage::PreUpdate, disconnected)
            .add_system_to_stage(CoreStage::PreUpdate, connected)
            .add_system_to_stage(CoreStage::PreUpdate, login)
            .add_event::<ClientDisconnectedEvent>()
            .add_event::<ClientConnectedEvent>();
    }
}

impl NetworkPlugin {
    pub fn new(server: SilkroadServer) -> Self {
        Self { server }
    }
}
