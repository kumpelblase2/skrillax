use crate::event::PlayerLevelUp;
use crate::game::chat::ChatPlugin;
use crate::game::drop::tick_drop;
use crate::game::entity_sync::{clean_sync, sync_changes_others, update_client};
use crate::game::levelup::notify_levelup;
use crate::game::movement::movement;
use crate::game::visibility::{player_visibility_update, visibility_update};
use bevy_app::{App, CoreStage, Plugin};

mod chat;
mod drop;
mod entity_sync;
mod levelup;
mod movement;
mod visibility;

pub(crate) struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ChatPlugin)
            .add_event::<PlayerLevelUp>()
            .add_system(visibility_update)
            .add_system(movement)
            .add_system(tick_drop)
            .add_system_to_stage(CoreStage::PostUpdate, sync_changes_others)
            .add_system_to_stage(CoreStage::PostUpdate, player_visibility_update)
            .add_system_to_stage(CoreStage::PostUpdate, update_client)
            .add_system_to_stage(CoreStage::PostUpdate, notify_levelup)
            .add_system_to_stage(CoreStage::Last, clean_sync);
    }
}
