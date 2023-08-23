use crate::chat::command::Command;
use crate::comp::{EntityReference, GameEntity};
use bevy_ecs::prelude::*;
use silkroad_data::skilldata::RefSkillData;
use silkroad_game_base::GlobalPosition;

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

pub(crate) struct AttackDefinition {
    pub skill: &'static RefSkillData,
    pub instance: u32,
}

#[derive(Event)]
pub(crate) struct DamageReceiveEvent {
    pub source: EntityReference,
    pub target: EntityReference,
    pub attack: AttackDefinition,
    pub amount: u32,
}

#[derive(Event)]
pub(crate) struct PlayerCommandEvent(pub Entity, pub Command);

#[derive(Event)]
pub(crate) struct PlayerTeleportEvent(pub Entity, pub GlobalPosition);

#[derive(Event)]
pub(crate) struct EntityDeath {
    pub died: Entity,
    pub killer: Option<Entity>,
}
