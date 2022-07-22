use crate::comp::pos::Position;
use crate::comp::visibility::Visibility;
use crate::comp::{GameEntity, Health};
use crate::settings::SpawnSettings;
use bevy_ecs::prelude::*;
use silkroad_protocol::world::EntityRarity;
use std::time::Instant;

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
}

#[derive(Component)]
pub struct Spawner {
    pub active: bool,
    pub radius: f32,
    pub ref_id: u32,
    pub target_amount: usize,
    pub current_amount: usize,
    pub last_spawn_check: Instant,
}

impl Spawner {
    pub(crate) fn new(settings: &SpawnSettings, spawned: u32) -> Self {
        Spawner {
            active: false,
            radius: settings.radius,
            target_amount: settings.amount,
            ref_id: spawned,
            current_amount: 0,
            last_spawn_check: Instant::now(),
        }
    }

    pub fn has_spots_available(&self) -> bool {
        self.current_amount < self.target_amount
    }
}
