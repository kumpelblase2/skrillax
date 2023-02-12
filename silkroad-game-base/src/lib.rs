mod character;
mod inventory;
mod movement;
mod pos;
mod skill;
mod stats;
mod vec;

pub use character::*;
pub use inventory::*;
pub use movement::*;
pub use pos::*;
pub use skill::*;
pub use stats::*;
pub use vec::*;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Race {
    European,
    Chinese,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum SpawningState {
    Loading,
    Spawning,
    Finished,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum MovementState {
    Standing,
    Sitting,
    Running,
    Walking,
}
