pub(crate) mod command;
mod system;

use crate::chat::command::system::{handle_command, handle_teleport};
use crate::chat::system::{handle_chat, handle_gm_commands};
use crate::event::{PlayerCommandEvent, PlayerTeleportEvent};
use bevy_app::{App, Plugin, Update};
use bevy_ecs::prelude::*;

pub(crate) struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_chat,
                handle_gm_commands,
                handle_command.after(handle_chat),
                handle_teleport.after(handle_command),
            ),
        )
        .add_event::<PlayerCommandEvent>()
        .add_event::<PlayerTeleportEvent>();
    }
}
