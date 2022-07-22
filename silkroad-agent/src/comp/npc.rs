use crate::comp::pos::{GlobalPosition, Heading, LocalPosition, Position};
use crate::comp::GameEntity;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub(crate) struct NPC;

pub(crate) struct NpcBundle {
    game_entity: GameEntity,
    npc: NPC,
    position: Position,
}

impl NpcBundle {
    pub fn new(unique_id: u32, ref_id: u32, position: LocalPosition) -> Self {
        Self {
            game_entity: GameEntity { unique_id, ref_id },
            npc: NPC,
            position: Position {
                location: position.to_global(),
                rotation: Heading(0.0),
            },
        }
    }
}
