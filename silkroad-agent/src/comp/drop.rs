use crate::comp::{EntityReference, GameEntity};
use bevy_core::Timer;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub(crate) struct ItemDrop {
    pub despawn_timer: Timer,
    pub owner: Option<EntityReference>,
    pub amount: u32,
}
