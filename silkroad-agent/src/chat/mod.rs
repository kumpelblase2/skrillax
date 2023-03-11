mod system;

use crate::chat::system::{handle_chat, handle_gm_commands};
use bevy_app::{App, CoreSet, Plugin};
use bevy_ecs::prelude::*;

pub(crate) struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems((handle_chat, handle_gm_commands).in_base_set(CoreSet::Update));
    }
}
