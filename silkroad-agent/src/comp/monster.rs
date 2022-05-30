use bevy_ecs_macros::Component;
use silkroad_protocol::world::EntityRarity;

#[derive(Component)]
pub struct Monster {
    pub ref_id: u32,
    pub rarity: EntityRarity,
    pub max_health: usize,
    pub current_health: usize,
}
