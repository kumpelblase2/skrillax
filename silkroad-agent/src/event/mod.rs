use crate::comp::monster::SpawnedBy;
use crate::comp::{EntityReference, GameEntity};
use bevy::prelude::*;
use silkroad_data::skilldata::RefSkillData;
use silkroad_definitions::TypeId;
use silkroad_game_base::GlobalLocation;

#[derive(Message)]
pub(crate) struct ClientConnectedEvent(pub Entity);

#[derive(Message)]
pub(crate) struct ClientDisconnectedEvent(pub Entity);

#[derive(Message)]
pub(crate) struct PlayerLevelUp(pub Entity, pub u8);

#[derive(Message)]
pub(crate) struct UniqueKilledEvent {
    pub player: String,
    pub unique: GameEntity,
}

pub(crate) struct SkillDefinition {
    pub skill: &'static RefSkillData,
    pub instance: u32,
}

#[derive(Message)]
pub(crate) struct DamageReceiveEvent {
    pub source: EntityReference,
    pub target: EntityReference,
    pub attack: SkillDefinition,
    pub amount: u32,
}

#[derive(Message)]
pub(crate) struct EntityDeath {
    pub died: EntityReference,
    pub killer: Option<EntityReference>,
}

#[derive(Message)]
pub(crate) struct SpawnMonster {
    pub ref_id: u32,
    pub location: GlobalLocation,
    pub spawner: Option<SpawnedBy>,
    pub with_ai: bool,
}

#[derive(Message)]
pub(crate) struct ConsumeItemEvent {
    pub player: Entity,
    pub item: TypeId,
    pub amount: u16,
}
