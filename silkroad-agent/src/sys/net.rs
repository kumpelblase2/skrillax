use crate::comp::{Client, LastAction, Login};
use crate::event::ServerEvent;
use crate::resources::CurrentTime;
use crate::GameSettings;
use bevy_ecs::prelude::*;
use silkroad_network::server::SilkroadServer;
use silkroad_network::stream::StreamError;
use silkroad_protocol::ClientPacket;
use std::collections::VecDeque;
use tracing::{debug, warn};

pub(crate) fn accept(
    mut events: EventWriter<ServerEvent>,
    network: Res<SilkroadServer>,
    current_time: Res<CurrentTime>,
    mut cmd: Commands,
) {
    while let Some(client) = network.connected() {
        debug!(id = ?client.id(), "Accepted client");

        events.send(ServerEvent::ClientConnected);

        cmd.spawn()
            .insert(Client(client, VecDeque::new()))
            .insert(LastAction(current_time.0))
            .insert(Login);
    }
}

pub(crate) fn receive(
    mut events: EventWriter<ServerEvent>,
    settings: Res<GameSettings>,
    current_time: Res<CurrentTime>,
    mut cmd: Commands,
    mut query: Query<(Entity, &mut Client, &mut LastAction)>,
) {
    'query: for (entity, mut client, mut last_action) in query.iter_mut() {
        let mut has_activity = false;
        loop {
            match client.0.received() {
                Ok(Some(packet)) => {
                    has_activity = true;
                    // Already handle keep-alives to not clog other systems
                    if !matches!(packet, ClientPacket::KeepAlive(_)) {
                        client.1.push_back(packet);
                    }
                }
                Ok(None) => break,
                Err(StreamError::StreamClosed) => {
                    cmd.entity(entity).despawn();
                    events.send(ServerEvent::ClientDisconnected);
                    continue 'query;
                }
                Err(e) => {
                    warn!(id = ?client.0.id(), "Error when receiving. {:?}", e);
                }
            }
        }

        if has_activity {
            last_action.0 = current_time.0;
        }

        if current_time.0.duration_since(last_action.0).as_secs() > settings.client_timeout as u64 {
            cmd.entity(entity).despawn();
            events.send(ServerEvent::ClientDisconnected);
        }
    }
}

pub(crate) fn disconnected(
    mut events: EventWriter<ServerEvent>,
    mut cmd: Commands,
    mut query: Query<(Entity, &Client)>,
) {
    for (entity, client) in query.iter() {
        if client.0.is_disconnected() {
            events.send(ServerEvent::ClientDisconnected);
            cmd.entity(entity).despawn();
        }
    }
}
