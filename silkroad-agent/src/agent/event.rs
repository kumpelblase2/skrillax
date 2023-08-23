use bevy_ecs::entity::Entity;
use bevy_ecs_macros::Event;

#[derive(Event)]
pub(crate) struct MovementFinished(pub Entity);

#[derive(Event)]
pub(crate) struct ActionFinished(pub Entity);
