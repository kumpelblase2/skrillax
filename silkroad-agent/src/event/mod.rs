use crate::comp::monster::SpawnedBy;
use crate::comp::{EntityReference, GameEntity};
use bevy::prelude::*;
use silkroad_data::skilldata::RefSkillData;
use silkroad_protocol::{ // Added import block for protocol types
    chat::ChatClientProtocol,
    combat::PerformAction,
    movement::MovementTarget,
    world::{TargetEntity, UnTargetEntity},
};
use silkroad_definitions::TypeId;
use silkroad_game_base::GlobalLocation;

#[derive(Event)]
pub(crate) struct ClientConnectedEvent(pub Entity);

#[derive(Event)]
pub(crate) struct ClientDisconnectedEvent(pub Entity);

#[derive(Event)]
pub(crate) struct PlayerLevelUp(pub Entity, pub u8);

#[derive(Event)]
pub(crate) struct LoadingFinishedEvent(pub Entity);

#[derive(Event)]
pub(crate) struct UniqueKilledEvent {
    pub player: String,
    pub unique: GameEntity,
}

pub(crate) struct SkillDefinition {
    pub skill: &'static RefSkillData,
    pub instance: u32,
}

#[derive(Event)]
pub(crate) struct DamageReceiveEvent {
    pub source: EntityReference,
    pub target: EntityReference,
    pub attack: SkillDefinition,
    pub amount: u32,
}

#[derive(Event)]
pub(crate) struct EntityDeath {
    pub died: EntityReference,
    pub killer: Option<EntityReference>,
}

#[derive(Event)]
pub(crate) struct SpawnMonster {
    pub ref_id: u32,
    pub location: GlobalLocation,
    pub spawner: Option<SpawnedBy>,
    pub with_ai: bool,
}

#[derive(Event)]
pub(crate) struct ConsumeItemEvent {
    pub player: Entity,
    pub item: TypeId,
    pub amount: u16,
}

// --- New Gameplay Event Struct Definitions ---

/// Event triggered when a player requests to move.
#[derive(Event)]
pub struct PlayerMovementRequestEvent {
    pub player_entity: Entity,
    pub request: MovementTarget,
}

/// Event triggered when a player sends a chat message.
#[derive(Event)]
pub struct PlayerChatEvent {
    pub player_entity: Entity,
    pub message: ChatClientProtocol,
}

/// Event triggered when a player requests to perform an action (e.g., skill cast, attack).
#[derive(Event)]
pub struct PlayerActionRequestEvent {
    pub player_entity: Entity,
    pub action: PerformAction,
}

/// Event triggered when a player requests to target an entity.
#[derive(Event)]
pub struct PlayerTargetEntityEvent {
    pub player_entity: Entity,
    pub target_request: TargetEntity,
}

/// Event triggered when a player requests to untarget an entity.
#[derive(Event)]
pub struct PlayerUntargetEntityEvent {
    pub player_entity: Entity,
    pub untarget_request: UnTargetEntity,
}

/// Event triggered when a player requests to log out gracefully.
#[derive(Event)]
pub struct PlayerLogoutRequestEvent(pub Entity);
