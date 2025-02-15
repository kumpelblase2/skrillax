mod system;

use crate::chat::system::{handle_chat, handle_gm_commands};
use bevy::prelude::*;

pub(crate) struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_chat, handle_gm_commands));
    }
}
