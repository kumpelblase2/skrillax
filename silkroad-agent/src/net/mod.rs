use crate::event::{ClientConnectedEvent, ClientDisconnectedEvent};
use crate::ext::ServerResource;
use crate::net::net::{accept, connected, disconnected};
use bevy_app::{App, First, Plugin};
use skrillax_server::Server;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::runtime::Runtime;

mod net;

pub struct NetworkPlugin {
    server: SocketAddr,
    runtime: Arc<Runtime>,
}

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        // Need to run this inside a `block_on` to ensure we're inside tokio and can
        // `spawn()` more tasks.
        let server = self.runtime.block_on(async {
            Server::new(self.server)
                .expect("Should be able to create the server")
                .into()
        });

        app.insert_resource::<ServerResource>(server)
            .add_systems(First, (accept, disconnected, connected))
            .add_event::<ClientDisconnectedEvent>()
            .add_event::<ClientConnectedEvent>();
    }
}

impl NetworkPlugin {
    pub fn new(server: SocketAddr, runtime: Arc<Runtime>) -> Self {
        Self { server, runtime }
    }
}
