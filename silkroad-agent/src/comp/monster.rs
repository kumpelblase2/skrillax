use crate::comp::player::Agent;
use crate::comp::pos::{GlobalLocation, Position};
use crate::comp::sync::Synchronize;
use crate::comp::visibility::Visibility;
use crate::comp::{GameEntity, Health};
use crate::config::SpawnOptions;
use bevy_ecs::prelude::*;
use bevy_time::{Timer, TimerMode};
use rand::random;
use silkroad_protocol::world::EntityRarity;
use std::time::{Duration, Instant};

#[derive(Component)]
pub struct Monster {
    pub target: Option<Entity>,
    pub rarity: EntityRarity,
}

#[derive(Component)]
pub struct SpawnedBy {
    pub spawner: Entity,
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
    pub(crate) sync: Synchronize,
    pub(crate) stroll: RandomStroll,
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

#[derive(Component)]
pub struct Spawner {
    pub active: bool,
    pub radius: f32,
    pub ref_id: u32,
    pub target_amount: usize,
    pub current_amount: usize,
    last_spawn_check: Instant,
    spawn_check_duration: Duration,
}

impl Spawner {
    pub(crate) fn new(settings: &SpawnOptions, spawned: u32) -> Self {
        Spawner {
            active: false,
            radius: settings.radius,
            target_amount: settings.amount,
            ref_id: spawned,
            current_amount: 0,
            last_spawn_check: Instant::now(),
            spawn_check_duration: Duration::from_secs(1),
        }
    }

    pub fn has_spots_available(&self) -> bool {
        self.current_amount < self.target_amount
    }

    pub fn should_spawn(&mut self, now: Instant) -> bool {
        if now.duration_since(self.last_spawn_check) > self.spawn_check_duration {
            self.last_spawn_check = now;
            return random::<f32>() > 0.5;
        }
        false
    }
}
