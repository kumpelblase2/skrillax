use crate::agent::goal::{AgentGoal, GoalTracker};
use crate::agent::state::Idle;
use crate::comp::{
    game_entity::GameEntity, // Added
    net::Client,             // Added
    player::Player,          // Added
};
use crate::comp::monster::RandomStroll;
use crate::comp::pos::Position;
use crate::ext::Navmesh;
use crate::session::Session; // Added
use bevy::prelude::*;
use rand::{rng, Rng}; // Corrected: rng should be Rng as used below, or rand::thread_rng().gen_range
use silkroad_game_base::{GlobalLocation, Vector2Ext};
use silkroad_protocol::{
    movement::{MovementDestination, PlayerMovementResponse}, // Added
    CLIENT_BLOWFISH,                                       // Added
};
use std::time::Duration;
use tracing::{debug, error}; // Added

pub(crate) fn movement_monster(
    mut query: Query<(&mut RandomStroll, &mut GoalTracker, &Position), With<Idle>>,
    delta: Res<Time>,
    navmesh: Res<Navmesh>,
) {
    let delta = delta.delta();
    for (mut stroll, mut goal, pos) in query.iter_mut() {
        if goal.has_goal() {
            continue;
        }

        if stroll.check_timer.tick(delta).just_finished() {
            let new_location = GlobalLocation(stroll.origin.0.random_in_radius(stroll.radius));
            let new_y = navmesh.height_for(new_location).unwrap_or(pos.position().0.y);
            goal.switch_goal(AgentGoal::moving_to(new_location.with_y(new_y)));
            let mut rng = rand::thread_rng(); // More explicit Rng instance
            let next_move_duration =
                Duration::from_secs(rng.gen_range(stroll.movement_timer_range.clone()));
            stroll.check_timer = Timer::new(next_move_duration, TimerMode::Once);
        }
    }
}

// System to broadcast player movements
pub(crate) fn player_movement_broadcast_system(
    moved_players_query: Query<
        (&GameEntity, &Position, &Client), // Added Client here to potentially avoid self-broadcast early
        (With<Player>, Changed<Position>), // Only players whose position changed
    >,
    other_players_query: Query<(&GameEntity, &Client), With<Player>>, // All players
) {
    for (moved_game_entity, moved_position, _moved_player_client) in moved_players_query.iter() {
        let moved_player_unique_id = moved_game_entity.unique_id;

        // Construct the destination from the player's current position
        // The GlobalPosition contains region_id, x, y, z.
        // The Position component contains angle (heading).
        let destination = MovementDestination::Location {
            region: moved_position.position().0.region_id,
            x: moved_position.position().0.x,
            // Silkroad y is our z
            y: moved_position.position().0.z,
            // Silkroad z is our y
            z: moved_position.position().0.y,
            heading: moved_position.angle(),
        };

        debug!(
            player_id = moved_player_unique_id,
            destination = ?destination,
            "Player moved. Broadcasting to other players."
        );

        let movement_packet = PlayerMovementResponse {
            player_id: moved_player_unique_id,
            destination,
            source_position: None, // As per requirement
        };

        for (other_game_entity, other_player_client) in other_players_query.iter() {
            // Don't send movement updates to the player who moved
            if other_game_entity.unique_id == moved_player_unique_id {
                continue;
            }

            debug!(
                "Sending movement of player {} to player {}",
                moved_player_unique_id, other_game_entity.unique_id
            );

            let mut session = Session::new(other_player_client.0.clone(), CLIENT_BLOWFISH.clone());
            if let Err(e) = session.send(movement_packet.clone()) {
                // Clone packet as send might consume it or be part of a retry loop later
                error!(
                    "Failed to send PlayerMovementResponse to unique_id {}: {:?}",
                    other_game_entity.unique_id, e
                );
            }
        }
    }
}
