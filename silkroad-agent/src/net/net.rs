use crate::comp::net::{Client, LastAction};
use crate::comp::player::Player;
use crate::db::character::CharacterData;
use crate::event::{ClientConnectedEvent, ClientDisconnectedEvent};
use crate::ext::{DbPool, ServerResource};
use crate::input::LoginInput;
use crate::tasks::TaskCreator;
use bevy_ecs::prelude::*;
use bevy_time::{Real, Time};
use std::time::Instant;
use tracing::debug;

pub(crate) fn accept(
    mut events: EventWriter<ClientConnectedEvent>,
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
                LoginInput::default(),
            ))
            .id();

        events.send(ClientConnectedEvent(entity));
    }
}

pub(crate) fn disconnected(
    mut events: EventReader<ClientDisconnectedEvent>,
    mut cmd: Commands,
    task_creator: Res<TaskCreator>,
    pool: Res<DbPool>,
    query: Query<&Player>,
) {
    for event in events.read() {
        let entity = event.0;
        debug!("Handling client disconnect.");
        if let Ok(player) = query.get(event.0) {
            let id = player.character.id;
            task_creator.spawn(CharacterData::update_last_played_of(id, pool.clone()));
        }
        cmd.entity(entity).despawn();
    }
}

pub(crate) fn connected(mut events: EventReader<ClientConnectedEvent>) {
    for _ in events.read() {
        // ..
    }
}
