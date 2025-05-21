// This file will contain the event handler systems for gameplay events.

use bevy::log::info; // Using bevy's info for logging as per example
use bevy::prelude::{Entity, EventReader, EventWriter}; // Added Entity, EventWriter

use crate::event::{
    ClientDisconnectedEvent, // Added ClientDisconnectedEvent
    PlayerActionRequestEvent, PlayerChatEvent, PlayerLogoutRequestEvent, // Added PlayerLogoutRequestEvent
    PlayerMovementRequestEvent, PlayerTargetEntityEvent, PlayerUntargetEntityEvent,
};

// Handler for PlayerMovementRequestEvent
pub fn handle_player_movement_request(
    mut event_reader: EventReader<PlayerMovementRequestEvent>,
) {
    for event in event_reader.read() {
        info!(
            "Received PlayerMovementRequestEvent for entity {:?} with request {:?}",
            event.player_entity, event.request
        );
        // Future game logic for movement will go here
    }
}

// Handler for PlayerChatEvent
pub fn handle_player_chat(mut event_reader: EventReader<PlayerChatEvent>) {
    for event in event_reader.read() {
        info!(
            "Received PlayerChatEvent for entity {:?} with message {:?}",
            event.player_entity, event.message
        );
        // Future game logic for chat will go here
    }
}

// Handler for PlayerActionRequestEvent
pub fn handle_player_action_request(
    mut event_reader: EventReader<PlayerActionRequestEvent>,
) {
    for event in event_reader.read() {
        info!(
            "Received PlayerActionRequestEvent for entity {:?} with action {:?}",
            event.player_entity, event.action
        );
        // Future game logic for actions will go here
    }
}

// Handler for PlayerTargetEntityEvent
pub fn handle_player_target_entity(
    mut event_reader: EventReader<PlayerTargetEntityEvent>,
) {
    for event in event_reader.read() {
        info!(
            "Received PlayerTargetEntityEvent for entity {:?} with target request {:?}",
            event.player_entity, event.target_request
        );
        // Future game logic for targeting will go here
    }
}

// Handler for PlayerUntargetEntityEvent
pub fn handle_player_untarget_entity(
    mut event_reader: EventReader<PlayerUntargetEntityEvent>,
) {
    for event in event_reader.read() {
        info!(
            "Received PlayerUntargetEntityEvent for entity {:?} with untarget request {:?}",
            event.player_entity, event.untarget_request
        );
        // Future game logic for untargeting will go here
    }
}

// Handler for PlayerLogoutRequestEvent
pub fn handle_player_logout_request(
    mut logout_request_reader: EventReader<PlayerLogoutRequestEvent>,
    mut client_disconnected_writer: EventWriter<ClientDisconnectedEvent>,
    // TODO: In the future, inject resources/queries needed for an immediate, targeted save
    // e.g., world: &mut World (if we need to run systems directly or access many things)
    // or specific Res<PersistedComponents>, Res<TaskCreator>, Res<DbPool>, Query<&Player> etc.
) {
    for event in logout_request_reader.read() {
        let player_entity = event.0;
        info!(entity = ?player_entity, "Received PlayerLogoutRequestEvent. Initiating logout process.");

        // Step 1: Trigger Save (Conceptual for now, relying on ClientDisconnectedEvent)
        // Ideally, we would have a function like:
        // `persistence_plugin.save_entity_now(player_entity, world_or_resources);`
        // For now, we log the intent and rely on the existing persistence systems
        // that are triggered by ClientDisconnectedEvent.
        info!(entity = ?player_entity, "Attempting to persist data for player before disconnect. (Currently relies on ClientDisconnectedEvent for actual save)");

        // Step 2: Send Logout Confirmation to Client (Actual packet sending is omitted here)
        // Example: client.send(LogoutSuccessPacket::new()).unwrap_or_default();
        info!(entity = ?player_entity, "Logout confirmation would be sent to client.");

        // Step 3: Initiate Client Disconnect
        // This will trigger net::disconnected system, which in turn handles despawn
        // and also triggers the persistence systems (apply_changes_exit, apply_changes_combined).
        info!(entity = ?player_entity, "Sending ClientDisconnectedEvent to complete logout.");
        client_disconnected_writer.send(ClientDisconnectedEvent(player_entity));
    }
}
