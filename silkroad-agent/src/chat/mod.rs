pub(crate) mod command;
mod system;

use crate::chat::command::system::{handle_command, handle_teleport};
use crate::chat::system::{handle_chat, handle_gm_commands};
use crate::event::{PlayerCommandEvent, PlayerTeleportEvent};
use bevy_app::{App, CoreSet, Plugin};
use bevy_ecs::prelude::*;

pub(crate) struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (
                handle_chat,
                handle_gm_commands,
                handle_command.after(handle_chat),
                handle_teleport.after(handle_command),
            )
                .in_base_set(CoreSet::Update),
        )
        .add_event::<PlayerCommandEvent>()
        .add_event::<PlayerTeleportEvent>();
    }
}
