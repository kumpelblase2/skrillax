use crate::comp::sync::Synchronize;
use crate::comp::visibility::Visibility;
use crate::comp::{Client, GameEntity, LastAction, Login};
use crate::event::{ClientConnectedEvent, ClientDisconnectedEvent};
use crate::GameSettings;
use bevy_core::Time;
use bevy_ecs::prelude::*;
use silkroad_network::server::SilkroadServer;
use silkroad_network::stream::StreamError;
use silkroad_protocol::ClientPacket;
use std::collections::VecDeque;
use tracing::{debug, warn};

pub(crate) fn accept(
    mut events: EventWriter<ClientConnectedEvent>,
    network: Res<SilkroadServer>,
    time: Res<Time>,
    mut cmd: Commands,
) {
    while let Some(client) = network.connected() {
        debug!(id = ?client.id(), "Accepted client");

        let entity = cmd
            .spawn()
            .insert(Client(client, VecDeque::new()))
            .insert(LastAction(time.last_update().unwrap()))
            .insert(Login)
            .id();

        events.send(ClientConnectedEvent(entity));
    }
}

pub(crate) fn receive(
    mut events: EventWriter<ClientDisconnectedEvent>,
    settings: Res<GameSettings>,
    time: Res<Time>,
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
                },
                Ok(None) => break,
                Err(StreamError::StreamClosed) => {
                    events.send(ClientDisconnectedEvent(entity));
                    continue 'query;
                },
                Err(e) => {
                    warn!(id = ?client.0.id(), "Error when receiving. {:?}", e);
                },
            }
        }

        let last_tick_time = time.last_update().unwrap();
        if has_activity {
            last_action.0 = last_tick_time.clone();
        }

        if last_tick_time.duration_since(last_action.0).as_secs() > settings.client_timeout as u64 {
            events.send(ClientDisconnectedEvent(entity));
        }
    }
}

pub(crate) fn disconnected(
    mut events: EventReader<ClientDisconnectedEvent>,
    mut cmd: Commands,
    query: Query<(&Visibility, &GameEntity)>,
    mut others: Query<&mut Synchronize>,
) {
    for event in events.iter() {
        let entity = event.0;
        if let Ok((visibility, game_entity)) = query.get(entity) {
            debug!("Handling client disconnect.");
            let visibility: &Visibility = visibility;
            let game_entity: &GameEntity = game_entity;
            for to_notify in visibility.entities_in_radius.iter() {
                if let Ok(mut synchronize) = others.get_mut(*to_notify) {
                    debug!("Sending despawned!.");
                    synchronize.despawned.push(game_entity.unique_id);
                }
            }
            cmd.entity(entity).despawn();
        }
    }
}

pub(crate) fn connected(mut events: EventReader<ClientConnectedEvent>) {
    for _ in events.iter() {
        // ..
    }
}
