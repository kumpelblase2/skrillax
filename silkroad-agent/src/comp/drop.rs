use crate::comp::pos::Position;
use crate::comp::{Despawn, EntityReference, GameEntity};
use bevy_ecs::prelude::*;

#[derive(Copy, Clone)]
pub enum Item {
    Gold(u32),
    Consumable(u32),
    Equipment { upgrade: u8 },
}

impl Item {
    pub fn amount(&self) -> u32 {
        match self {
            Item::Gold(amount) | Item::Consumable(amount) => *amount,
            Item::Equipment { .. } => 1,
        }
    }
}

#[derive(Component)]
pub(crate) struct ItemDrop {
    pub owner: Option<EntityReference>,
    pub item: Item,
}

#[derive(Bundle)]
pub(crate) struct DropBundle {
    pub(crate) drop: ItemDrop,
    pub(crate) position: Position,
    pub(crate) game_entity: GameEntity,
    pub(crate) despawn: Despawn,
}
