use bevy_ecs::entity::Entity;
use bevy_ecs_macros::Event;

#[derive(Event)]
pub struct MallOpenRequestEvent(pub Entity);
