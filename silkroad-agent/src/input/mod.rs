mod component;
mod system;

use crate::input::system::{receive_game_inputs, receive_login_inputs, reset};
use bevy::prelude::*;
pub(crate) use component::*;

pub(crate) struct ReceivePlugin;

impl Plugin for ReceivePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(First, (receive_game_inputs, receive_login_inputs))
            .add_systems(Last, reset);
    }
}
