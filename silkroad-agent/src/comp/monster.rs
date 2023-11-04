use crate::agent::states::StateTransitionQueue;
use crate::agent::{Agent, MovementState};
use crate::comp::damage::DamageReceiver;
use crate::comp::pos::Position;
use crate::comp::visibility::Visibility;
use crate::comp::{GameEntity, Health};
use crate::game::mind::Mind;
use bevy_ecs::prelude::*;
use bevy_time::{Timer, TimerMode};
use silkroad_definitions::rarity::EntityRarity;
use silkroad_game_base::GlobalLocation;
use std::time::Duration;

#[derive(Component, Copy, Clone)]
pub struct Monster {
    pub target: Option<Entity>,
    pub rarity: EntityRarity,
}

#[derive(Component, Copy, Clone)]
pub enum SpawnedBy {
    Spawner(Entity),
    Player(Entity),
    Monster(Entity),
    None,
}

#[derive(Bundle)]
pub struct MonsterBundle {
    pub(crate) monster: Monster,
    pub(crate) health: Health,
    pub(crate) position: Position,
    pub(crate) entity: GameEntity,
    pub(crate) visibility: Visibility,
    pub(crate) spawner: SpawnedBy,
    pub(crate) navigation: Agent,
    pub(crate) state_queue: StateTransitionQueue,
    pub(crate) movement_state: MovementState,
    pub(crate) damage_receiver: DamageReceiver,
}

#[derive(Bundle)]
pub struct MonsterAiBundle {
    pub(crate) stroll: RandomStroll,
    pub(crate) mind: Mind,
}

#[derive(Component)]
pub struct RandomStroll {
    pub(crate) origin: GlobalLocation,
    pub(crate) radius: f32,
    pub(crate) check_timer: Timer,
}

impl RandomStroll {
    pub fn new(origin: GlobalLocation, radius: f32, interval: Duration) -> Self {
        Self {
            origin,
            radius,
            check_timer: Timer::new(interval, TimerMode::Repeating),
        }
    }
}
