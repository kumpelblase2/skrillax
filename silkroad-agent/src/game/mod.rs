use crate::agent::AgentBroadcastLabel;
use crate::chat::ChatPlugin;
use crate::event::{LoadingFinishedEvent, PlayerLevelUp, UniqueKilledEvent};
use crate::game::action::handle_action;
use crate::game::daylight::{advance_daylight, DaylightCycle};
use crate::game::drop::{create_drops, tick_drop, SpawnDrop};
use crate::game::entity_sync::{clean_sync, sync_changes_others, update_client};
use crate::game::inventory::handle_inventory_input;
use crate::game::join::load_finished;
use crate::game::levelup::notify_levelup;
use crate::game::logout::{handle_logout, tick_logout};
use crate::game::movement::movement_monster;
use crate::game::player_activity::{update_player_activity, PlayerActivity};
use crate::game::target::player_update_target;
use crate::game::unique::{unique_killed, unique_spawned};
use crate::game::visibility::{clear_visibility, player_visibility_update, visibility_update};
use bevy_app::{App, CoreStage, Plugin};
use bevy_ecs::prelude::*;

mod action;
mod daylight;
pub(crate) mod drop;
mod entity_sync;
mod gold;
pub(crate) mod inventory;
mod join;
mod levelup;
pub(crate) mod logout;
mod movement;
pub(crate) mod player_activity;
mod target;
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
            .add_event::<SpawnDrop>()
            .add_system_to_stage(CoreStage::PreUpdate, update_player_activity)
            .add_system(handle_inventory_input)
            .add_system(visibility_update)
            .add_system(movement_monster)
            .add_system(tick_drop)
            .add_system(handle_logout)
            .add_system(handle_action)
            .add_system(tick_logout)
            .add_system(player_update_target)
            .add_system_set_to_stage(
                CoreStage::PostUpdate,
                SystemSet::new()
                    .after(AgentBroadcastLabel)
                    .with_system(sync_changes_others)
                    .with_system(update_client),
            )
            .add_system_to_stage(CoreStage::PostUpdate, player_visibility_update)
            .add_system_to_stage(CoreStage::PostUpdate, notify_levelup)
            .add_system_to_stage(CoreStage::PostUpdate, load_finished)
            .add_system_to_stage(CoreStage::PostUpdate, unique_spawned)
            .add_system_to_stage(CoreStage::PostUpdate, unique_killed)
            .add_system_to_stage(CoreStage::PostUpdate, advance_daylight)
            .add_system_to_stage(CoreStage::PostUpdate, create_drops)
            .add_system_to_stage(CoreStage::Last, clean_sync)
            .add_system_to_stage(CoreStage::Last, clear_visibility);
    }
}
