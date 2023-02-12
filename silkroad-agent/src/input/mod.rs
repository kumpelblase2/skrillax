mod component;
mod system;

use crate::input::system::{receive_game_inputs, receive_login_inputs, reset};
use bevy_app::{App, CoreStage, Plugin};
pub(crate) use component::*;

pub(crate) struct ReceivePlugin;

impl Plugin for ReceivePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::First, receive_game_inputs)
            .add_system_to_stage(CoreStage::First, receive_login_inputs)
            .add_system_to_stage(CoreStage::Last, reset);
    }
}
