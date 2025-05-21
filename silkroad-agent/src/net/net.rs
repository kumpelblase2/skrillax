use crate::comp::game_entity::GameEntity; // Added
use crate::comp::net::{Client, LastAction};
use crate::comp::player::Player;
use crate::db::character::CharacterData;
use crate::session::Session; // Added
use silkroad_protocol::spawn::EntityDespawn; // Added
use silkroad_protocol::CLIENT_BLOWFISH; // Added
use crate::event::{ClientConnectedEvent, ClientDisconnectedEvent};
use crate::ext::{DbPool, ServerResource};
use crate::input::LoginInput;
use crate::tasks::TaskCreator;
use bevy::prelude::*;
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
    disconnecting_player_query: Query<(&Player, &GameEntity)>, // Modified query
    other_players_query: Query<(Entity, &Client), With<Player>>, // Added query for other players
) {
    for event in events.read() {
        let entity_being_disconnected = event.0;
        debug!(entity = ?entity_being_disconnected, "Handling client disconnect event.");

        // Check if the disconnecting entity is a player and has a GameEntity
        if let Ok((disconnected_player, game_entity)) =
            disconnecting_player_query.get(entity_being_disconnected)
        {
            debug!(player_name = %disconnected_player.character.name, unique_id = %game_entity.unique_id, "Player is disconnecting. Broadcasting despawn.");

            // 1. Broadcast EntityDespawn to other players
            let unique_id_to_despawn = game_entity.unique_id;
            let despawn_packet = EntityDespawn::new(unique_id_to_despawn);

            for (other_player_entity, other_player_client) in other_players_query.iter() {
                // Don't send to the player who is disconnecting
                if other_player_entity == entity_being_disconnected {
                    continue;
                }

                debug!(
                    "Sending EntityDespawn for unique_id {} to player entity {:?}",
                    unique_id_to_despawn, other_player_entity
                );

                let mut session = Session::new(other_player_client.0.clone(), CLIENT_BLOWFISH.clone());
                if let Err(e) = session.send(despawn_packet.clone()) {
                    // It's important to clone despawn_packet if Session::send consumes it or if sending can be retried.
                    // Assuming send might be fallible and we want to log errors.
                    error!(
                        "Failed to send EntityDespawn packet to {:?}: {:?}",
                        other_player_entity, e
                    );
                }
            }

            // 2. Update last played time (existing logic)
            // This is a direct DB call for a specific, session-related timestamp.
            let character_id = disconnected_player.character.id;
            task_creator.spawn(CharacterData::update_last_played_of(
                character_id,
                pool.clone(),
            ));
            // Note: Other character component data (Position, Health, Gold, Masteries, Inventory, etc.)
            // is saved by the PersistencePlugin systems, which are also triggered by ClientDisconnectedEvent.
        } else {
            debug!(entity = ?entity_being_disconnected, "Disconnecting entity is not a player character (or missing GameEntity). No despawn broadcast needed.");
        }

        // 3. Despawn the entity (existing logic, now after broadcast)
        // Despawning the entity will remove its components. If any persistence systems rely on
        // component removal (though current ones primarily use ClientDisconnectedEvent), this is the point.
        if let Some(mut entity_commands) = cmd.get_entity(entity_being_disconnected) {
            debug!(entity = ?entity_being_disconnected, "Despawning entity.");
            entity_commands.despawn();
        } else {
            warn!(entity = ?entity_being_disconnected, "Failed to get entity for despawning. It might have been despawned already.");
        }
    }
}

pub(crate) fn connected(mut events: EventReader<ClientConnectedEvent>) {
    for _ in events.read() {
        // ..
    }
}
