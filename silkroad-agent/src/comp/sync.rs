use crate::comp::pos::{Heading, LocalPosition};
use bevy_ecs_macros::Component;

#[derive(Component, Default)]
pub struct Synchronize {
    pub movement: Option<MovementUpdate>,
    pub damage: Vec<DamageReceived>,
}

pub enum MovementUpdate {
    StartMove(LocalPosition, LocalPosition),
    StopMove(LocalPosition),
    Turn(Heading),
}

impl Synchronize {
    pub fn clear(&mut self) {
        self.movement = None;
    }
}

pub struct DamageReceived {
    pub amount: u32,
    pub crit: bool,
}
