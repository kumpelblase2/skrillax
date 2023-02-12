use crate::agent::Agent;
use crate::comp::pos::Position;
use crate::comp::GameEntity;
use bevy_ecs::prelude::*;
use silkroad_game_base::{Heading, LocalPosition};

#[derive(Component)]
pub(crate) struct NPC;

#[derive(Bundle)]
pub(crate) struct NpcBundle {
    game_entity: GameEntity,
    npc: NPC,
    agent: Agent,
    position: Position,
}

impl NpcBundle {
    pub fn new(unique_id: u32, ref_id: u32, position: LocalPosition, agent: Agent) -> Self {
        Self {
            game_entity: GameEntity { unique_id, ref_id },
            npc: NPC,
            agent,
            position: Position {
                location: position.to_global(),
                rotation: Heading(0.0), // FIXME: We need to load the npc rotations from somewhere
            },
        }
    }
}
