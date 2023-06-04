use crate::chat::command::Command;
use crate::comp::{EntityReference, GameEntity};
use bevy_ecs::prelude::*;
use silkroad_data::skilldata::RefSkillData;
use silkroad_game_base::GlobalPosition;

pub(crate) struct ClientConnectedEvent(pub Entity);

pub(crate) struct ClientDisconnectedEvent(pub Entity);

pub(crate) struct PlayerLevelUp(pub Entity, pub u8);

pub(crate) struct LoadingFinishedEvent(pub Entity);

pub(crate) struct UniqueKilledEvent {
    pub player: String,
    pub unique: GameEntity,
}

pub(crate) struct AttackDefinition {
    pub skill: &'static RefSkillData,
    pub instance: u32,
}

pub(crate) struct DamageReceiveEvent {
    pub source: EntityReference,
    pub target: EntityReference,
    pub attack: AttackDefinition,
    pub amount: u32,
}

pub(crate) struct PlayerCommandEvent(pub Entity, pub Command);

pub(crate) struct PlayerTeleportEvent(pub Entity, pub GlobalPosition);

pub(crate) struct EntityDeath(pub Entity);
