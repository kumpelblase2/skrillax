use crate::event::{ClientConnectedEvent, ClientDisconnectedEvent};
use crate::ext::ServerResource;
use crate::net::net::{accept, connected, disconnected};
use bevy_app::{App, Plugin, PreUpdate};
use silkroad_network::server::SilkroadServer;

mod net;

pub struct NetworkPlugin {
    server: SilkroadServer,
}

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource::<ServerResource>(self.server.clone().into())
            .add_systems(PreUpdate, (accept, disconnected, connected))
            .add_event::<ClientDisconnectedEvent>()
            .add_event::<ClientConnectedEvent>();
    }
}

impl NetworkPlugin {
    pub fn new(server: SilkroadServer) -> Self {
        Self { server }
    }
}
