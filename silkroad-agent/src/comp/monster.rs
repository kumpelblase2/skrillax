use bevy_ecs::entity::Entity;
use bevy_ecs_macros::Component;
use silkroad_protocol::world::EntityRarity;

#[derive(Component)]
pub struct Monster {
    pub target: Option<Entity>,
    pub rarity: EntityRarity,
}
