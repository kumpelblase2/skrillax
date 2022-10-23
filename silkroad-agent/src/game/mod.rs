use crate::event::{LoadingFinishedEvent, PlayerLevelUp, UniqueKilledEvent};
use crate::game::chat::ChatPlugin;
use crate::game::daylight::{advance_daylight, DaylightCycle};
use crate::game::drop::tick_drop;
use crate::game::entity_sync::{clean_sync, sync_changes_others, update_client};
use crate::game::inventory::handle_inventory_input;
use crate::game::join::load_finished;
use crate::game::levelup::notify_levelup;
use crate::game::movement::{movement, movement_input, movement_monster, update_attack_location};
use crate::game::player_activity::{reset_player_activity, update_player_activity, PlayerActivity};
use crate::game::unique::{unique_killed, unique_spawned};
use crate::game::visibility::{clear_visibility, player_visibility_update, visibility_update};
use crate::game::world::{finish_logout, handle_world_input};
use bevy_app::{App, CoreStage, Plugin};

mod chat;
mod daylight;
mod drop;
mod entity_sync;
mod gm;
mod gold;
mod inventory;
mod join;
mod levelup;
mod movement;
pub(crate) mod player_activity;
mod unique;
mod visibility;
mod world;

pub(crate) struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ChatPlugin)
            .insert_resource(PlayerActivity::default())
            .insert_resource(DaylightCycle::official())
            .add_event::<PlayerLevelUp>()
            .add_event::<LoadingFinishedEvent>()
            .add_event::<UniqueKilledEvent>()
            .add_system_to_stage(CoreStage::PreUpdate, update_player_activity)
            .add_system(handle_world_input)
            .add_system(movement_input)
            .add_system(handle_inventory_input)
            .add_system(visibility_update)
            .add_system(movement)
            .add_system(update_attack_location)
            .add_system(movement_monster)
            .add_system(tick_drop)
            .add_system(finish_logout)
            .add_system_to_stage(CoreStage::PostUpdate, sync_changes_others)
            .add_system_to_stage(CoreStage::PostUpdate, player_visibility_update)
            .add_system_to_stage(CoreStage::PostUpdate, clear_visibility)
            .add_system_to_stage(CoreStage::PostUpdate, update_client)
            .add_system_to_stage(CoreStage::PostUpdate, notify_levelup)
            .add_system_to_stage(CoreStage::PostUpdate, load_finished)
            .add_system_to_stage(CoreStage::PostUpdate, unique_spawned)
            .add_system_to_stage(CoreStage::PostUpdate, unique_killed)
            .add_system_to_stage(CoreStage::PostUpdate, advance_daylight)
            .add_system_to_stage(CoreStage::Last, clean_sync)
            .add_system_to_stage(CoreStage::Last, reset_player_activity);
    }
}
