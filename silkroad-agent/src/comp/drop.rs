use crate::comp::pos::Position;
use crate::comp::{Despawn, EntityReference, GameEntity};
use bevy::prelude::*;
use silkroad_game_base::Item;

#[derive(Component)]
pub(crate) struct Drop {
    pub owner: Option<EntityReference>,
    pub item: Item,
}

#[derive(Bundle)]
pub(crate) struct DropBundle {
    pub(crate) drop: Drop,
    pub(crate) position: Position,
    pub(crate) game_entity: GameEntity,
    pub(crate) despawn: Despawn,
}
