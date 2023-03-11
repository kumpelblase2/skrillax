mod component;
mod system;

use crate::input::system::{receive_game_inputs, receive_login_inputs, reset};
use bevy_app::{App, CoreSet, Plugin};
use bevy_ecs::prelude::*;
pub(crate) use component::*;

pub(crate) struct ReceivePlugin;

impl Plugin for ReceivePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems((receive_game_inputs, receive_login_inputs).in_base_set(CoreSet::First))
            .add_system(reset.in_base_set(CoreSet::Last));
    }
}
