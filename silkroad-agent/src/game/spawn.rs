use bevy_ecs::prelude::*;
use silkroad_game_api_client::Client;
use silkroad_protocol::{
    protocol::character::{CharacterSpawnItemData, GuildInformationData}, // Updated for GuildInformationData
    spawn::{EntitySpawn, EntityTypeSpawnData, EntityMovementState}, // Added EntityMovementState
    CLIENT_BLOWFISH, // Keep this if Session::new requires it directly
};
use tracing::{error, info, warn}; // Added warn

use crate::{
    game::components::{GameEntity, Player, PlayerBundle, PlayerInventory, Position},
    session::Session,
};

// System to broadcast player spawns
pub fn player_spawn_broadcast_system(
    // Query for new players: entities with PlayerBundle added, and all required components
    new_players_query: Query<
        (
            &GameEntity,
            &Player,
            &Position,
            &PlayerInventory,
            &Client, // Client is associated with the new player
        ),
        Added<PlayerBundle>, // Filter for entities where PlayerBundle was just added
    >,
    // Query for all existing players (including the new ones, so we'll filter out self-spawning to self)
    existing_players_query: Query<(
        &GameEntity,
        &Player,
        &Position,
        &PlayerInventory,
        &Client, // Client is associated with each existing player
    )>,
) {
    // Iterate over each new player that has just joined
    for (
        new_player_game_entity,
        new_player_component,
        new_player_position,
        new_player_inventory,
        new_player_client, // This is the client of the new player
    ) in new_players_query.iter()
    {
        info!(
            "New player {} ({}) joined. Broadcasting spawn.",
            new_player_component.name, new_player_game_entity.unique_id
        );

        // Iterate over all players currently in the game (including the new one)
        for (
            existing_player_game_entity,
            existing_player_component,
            existing_player_position,
            existing_player_inventory,
            existing_player_client, // This is the client of the existing player
        ) in existing_players_query.iter()
        {
            // Avoid sending a player's spawn information to themselves in this initial broadcast loop.
            // The new player should not send their own spawn to themselves here.
            // An existing player should not send their own spawn to themselves when a new player joins.
            if new_player_game_entity.unique_id == existing_player_game_entity.unique_id {
                continue;
            }

            // 1. Send existing player's spawn data to the NEW player
            let existing_player_equipment: Vec<CharacterSpawnItemData> = existing_player_inventory
                .iter_equipped()
                .filter_map(|item_info| {
                    if let Some(item) = &item_info.item {
                        Some(CharacterSpawnItemData {
                            ref_id: item.ref_id,
                            plus_level: item.plus_level.unwrap_or(0),
                        })
                    } else {
                        warn!(
                            "Equipped item missing item data for player {}",
                            existing_player_component.name
                        );
                        None
                    }
                })
                .collect();

            let existing_player_spawn_data = EntityTypeSpawnData::Character {
                unique_id: existing_player_game_entity.unique_id,
                ref_id: existing_player_component.ref_id,
                position: existing_player_position.as_protocol_vec3(),
                angle: existing_player_position.angle,
                movement_state: EntityMovementState::Standing, // Default state
                name: existing_player_component.name.clone(),
                scale: 100, // Default scale
                pvp_state: 0, // Default
                appearance_state: 0, // Default
                movement_speed_walk: existing_player_component.walk_speed,
                movement_speed_run: existing_player_component.run_speed,
                life_state: 0, // Default (Alive)
                ref_job_id: 0, // Default
                job_level: 0,  // Default
                pvp_flag: 0,   // Default
                in_combat_vehicle_unique_id: 0, // Default
                scroll_mode: 0, // Default (Normal)
                interaction_flag: 0, // Default
                stall_name: String::new(), // Default (no stall)
                equipment: existing_player_equipment,
                guild: None, // Default (no guild info for now) - Placeholder for GuildInformationData
                job_equipment: Vec::new(), // Default (no job equipment for now)
            };

            let spawn_packet_for_new_player = EntitySpawn {
                data: existing_player_spawn_data,
            };

            // Send to the NEW player's client
            let mut new_player_session = Session::new(new_player_client.clone(), CLIENT_BLOWFISH.clone());
            if let Err(e) = new_player_session.send(spawn_packet_for_new_player) {
                error!(
                    "Failed to send spawn of existing player {} ({}) to new player {} ({}): {:?}",
                    existing_player_component.name,
                    existing_player_game_entity.unique_id,
                    new_player_component.name,
                    new_player_game_entity.unique_id,
                    e
                );
            } else {
                info!(
                    "Sent spawn of existing player {} ({}) to new player {} ({})",
                    existing_player_component.name,
                    existing_player_game_entity.unique_id,
                    new_player_component.name,
                    new_player_game_entity.unique_id
                );
            }

            // 2. Send new player's spawn data to the EXISTING player
            let new_player_equipment: Vec<CharacterSpawnItemData> = new_player_inventory
                .iter_equipped()
                .filter_map(|item_info| {
                    if let Some(item) = &item_info.item {
                        Some(CharacterSpawnItemData {
                            ref_id: item.ref_id,
                            plus_level: item.plus_level.unwrap_or(0),
                        })
                    } else {
                        warn!(
                            "Equipped item missing item data for new player {}",
                            new_player_component.name
                        );
                        None
                    }
                })
                .collect();

            let new_player_spawn_data = EntityTypeSpawnData::Character {
                unique_id: new_player_game_entity.unique_id,
                ref_id: new_player_component.ref_id,
                position: new_player_position.as_protocol_vec3(),
                angle: new_player_position.angle,
                movement_state: EntityMovementState::Standing, // Default state
                name: new_player_component.name.clone(),
                scale: 100, // Default scale
                pvp_state: 0, // Default
                appearance_state: 0, // Default
                movement_speed_walk: new_player_component.walk_speed,
                movement_speed_run: new_player_component.run_speed,
                life_state: 0, // Default (Alive)
                ref_job_id: 0, // Default
                job_level: 0,  // Default
                pvp_flag: 0,   // Default
                in_combat_vehicle_unique_id: 0, // Default
                scroll_mode: 0, // Default (Normal)
                interaction_flag: 0, // Default
                stall_name: String::new(), // Default (no stall)
                equipment: new_player_equipment,
                guild: None, // Default (no guild info for now) - Placeholder for GuildInformationData
                job_equipment: Vec::new(), // Default (no job equipment for now)
            };

            let spawn_packet_for_existing_player = EntitySpawn {
                data: new_player_spawn_data,
            };

            // Send to the EXISTING player's client
            let mut existing_player_session = Session::new(existing_player_client.clone(), CLIENT_BLOWFISH.clone());
            if let Err(e) = existing_player_session.send(spawn_packet_for_existing_player) {
                error!(
                    "Failed to send spawn of new player {} ({}) to existing player {} ({}): {:?}",
                    new_player_component.name,
                    new_player_game_entity.unique_id,
                    existing_player_component.name,
                    existing_player_game_entity.unique_id,
                    e
                );
            } else {
                info!(
                    "Sent spawn of new player {} ({}) to existing player {} ({})",
                    new_player_component.name,
                    new_player_game_entity.unique_id,
                    existing_player_component.name,
                    existing_player_game_entity.unique_id
                );
            }
        }
    }
}
