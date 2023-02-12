mod system;

use crate::chat::system::{handle_chat, handle_gm_commands};
use bevy_app::{App, CoreStage, Plugin};
use bevy_ecs::prelude::SystemSet;

pub(crate) struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            CoreStage::Update,
            SystemSet::new()
                .with_system(handle_chat)
                .with_system(handle_gm_commands),
        );
    }
}
