use crate::game::entity_sync::{clean_sync, sync_changes};
use crate::game::movement::movement;
use crate::game::visibility::{player_visibility_update, visibility_update};
use bevy_app::{App, CoreStage, Plugin};

mod entity_sync;
mod movement;
mod visibility;

pub(crate) struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(visibility_update)
            .add_system(movement)
            .add_system_to_stage(CoreStage::PostUpdate, sync_changes)
            .add_system_to_stage(CoreStage::PostUpdate, player_visibility_update)
            .add_system_to_stage(CoreStage::Last, clean_sync);
    }
}
