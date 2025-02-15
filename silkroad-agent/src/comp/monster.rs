use crate::agent::component::{Agent, MovementState};
use crate::agent::goal::GoalTracker;
use crate::agent::state::AgentStateQueue;
use crate::comp::damage::DamageReceiver;
use crate::comp::pos::Position;
use crate::comp::visibility::Visibility;
use crate::comp::{GameEntity, Health};
use bevy::prelude::*;
use rand::{thread_rng, Rng};
use silkroad_definitions::rarity::EntityRarity;
use silkroad_game_base::GlobalLocation;
use std::ops::Range;
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
    pub(crate) state_queue: AgentStateQueue,
    pub(crate) movement_state: MovementState,
    pub(crate) damage_receiver: DamageReceiver,
}

#[derive(Bundle)]
pub struct MonsterAiBundle {
    pub(crate) stroll: RandomStroll,
    pub(crate) goal: GoalTracker,
}

#[derive(Component)]
pub struct RandomStroll {
    pub(crate) origin: GlobalLocation,
    pub(crate) radius: f32,
    pub(crate) movement_timer_range: Range<u64>,
    pub(crate) check_timer: Timer,
}

impl RandomStroll {
    pub fn new(origin: GlobalLocation, radius: f32, timer_range: Range<u64>) -> Self {
        Self {
            origin,
            radius,
            movement_timer_range: timer_range.clone(),
            check_timer: Timer::new(
                Duration::from_secs(thread_rng().gen_range(timer_range)),
                TimerMode::Once,
            ),
        }
    }
}
