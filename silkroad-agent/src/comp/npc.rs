use crate::agent::component::Agent;
use crate::comp::pos::Position;
use crate::comp::GameEntity;
use bevy::prelude::*;
use silkroad_game_base::{Heading, LocalPosition};

#[allow(clippy::upper_case_acronyms)]
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
        let position = Position::new(position.to_global(), Heading(0.0)); // TODO: need to get NPC rotation
        Self {
            game_entity: GameEntity { unique_id, ref_id },
            npc: NPC,
            agent,
            position,
        }
    }
}
