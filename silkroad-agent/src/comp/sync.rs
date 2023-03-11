use bevy_ecs_macros::Component;
use silkroad_game_base::{Heading, LocalPosition, Vector3Ext};
use silkroad_protocol::world::UpdatedState;
use std::ops::{Deref, Sub};

#[derive(Component, Default)]
pub struct Synchronize {
    pub movement: Option<MovementUpdate>,
    pub rotation: Option<Heading>,
    pub state: Vec<UpdatedState>,
    pub actions: Vec<ActionAnimation>,
    pub health: Option<u32>,
}

pub enum MovementUpdate {
    StartMove(LocalPosition, LocalPosition),
    StartMoveTowards(LocalPosition, Heading),
    StopMove(LocalPosition, Heading),
    Turn(Heading),
}

impl MovementUpdate {
    pub fn rotation(&self) -> Heading {
        match self {
            MovementUpdate::StartMove(from, to) => {
                let dir = to.to_global().sub(from.to_global().deref());
                Heading::from(dir.to_flat_vec2())
            },
            MovementUpdate::StartMoveTowards(_, heading)
            | MovementUpdate::StopMove(_, heading)
            | MovementUpdate::Turn(heading) => *heading,
        }
    }
}

pub enum ActionAnimation {
    Pickup,
}

impl Synchronize {
    pub fn clear(&mut self) {
        self.movement = None;
        self.rotation = None;
        self.health = None;
        self.state.clear();
        self.actions.clear();
    }
}
