use crate::comp::pos::Position;
use crate::comp::GameEntity;
use crate::settings::SpawnSettings;
use bevy_ecs::prelude::*;
use silkroad_protocol::world::EntityRarity;
use std::time::Instant;

#[derive(Component)]
pub struct Monster {
    pub target: Option<Entity>,
    pub rarity: EntityRarity,
}

#[derive(Bundle)]
pub struct MonsterBundle {
    pub(crate) monster: Monster,
    pub(crate) position: Position,
    pub(crate) entity: GameEntity,
}

#[derive(Component)]
pub struct Spawner {
    pub radius: f32,
    pub ref_id: u32,
    pub target_amount: usize,
    pub current_amount: usize,
    pub last_spawn_check: Instant,
}

impl Spawner {
    pub fn new(settings: &SpawnSettings, spawned: u32) -> Self {
        Spawner {
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
