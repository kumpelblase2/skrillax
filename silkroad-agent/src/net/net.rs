use crate::comp::net::{Client, LastAction};
use crate::event::{ClientConnectedEvent, ClientDisconnectedEvent};
use crate::ext::ServerResource;
use bevy::prelude::*;
use std::time::Instant;
use tracing::debug;

pub(crate) fn accept(
    mut events: MessageWriter<ClientConnectedEvent>,
    network: Res<ServerResource>,
    time: Res<Time<Real>>,
    mut cmd: Commands,
) {
    for client in network.accepted_connections() {
        debug!(id = ?client.id(), "Accepted client");

        let entity = cmd
            .spawn((
                Client(client),
                LastAction(time.last_update().unwrap_or_else(Instant::now)),
            ))
            .id();

        events.write(ClientConnectedEvent(entity));
    }
}

pub(crate) fn disconnected(mut events: MessageReader<ClientDisconnectedEvent>, mut cmd: Commands) {
    for event in events.read() {
        debug!("Handling client disconnect.");
        if let Ok(mut cmd) = cmd.get_entity(event.0) {
            cmd.despawn();
        }
    }
}

pub(crate) fn connected(mut events: MessageReader<ClientConnectedEvent>) {
    for _ in events.read() {
        // ..
    }
}
