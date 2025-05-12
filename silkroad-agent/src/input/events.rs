use bevy::prelude::*;

#[derive(Message)]
pub struct PlayerInputEvent<T> {
    pub player: Entity,
    pub input: T,
}

impl<T> PlayerInputEvent<T> {
    pub fn new(player: Entity, input: T) -> Self {
        PlayerInputEvent { player, input }
    }
}
