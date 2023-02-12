use bevy_ecs_macros::Component;
use std::time::Duration;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub(crate) struct Sitting(pub Duration);
