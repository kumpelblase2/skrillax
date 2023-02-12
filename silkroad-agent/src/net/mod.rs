use crate::event::{ClientConnectedEvent, ClientDisconnectedEvent};
use crate::ext::ServerResource;
use crate::net::net::{accept, connected, disconnected};
use bevy_app::{App, CoreStage, Plugin};
use silkroad_network::server::SilkroadServer;

mod net;

pub struct NetworkPlugin {
    server: SilkroadServer,
}

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource::<ServerResource>(self.server.clone().into())
            .add_system_to_stage(CoreStage::PreUpdate, accept)
            .add_system_to_stage(CoreStage::PreUpdate, disconnected)
            .add_system_to_stage(CoreStage::PreUpdate, connected)
            .add_event::<ClientDisconnectedEvent>()
            .add_event::<ClientConnectedEvent>();
    }
}

impl NetworkPlugin {
    pub fn new(server: SilkroadServer) -> Self {
        Self { server }
    }
}
