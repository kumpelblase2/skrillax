use crate::agent::AgentSet;
use crate::chat::ChatPlugin;
use crate::event::{DamageReceiveEvent, EntityDeath, LoadingFinishedEvent, PlayerLevelUp, UniqueKilledEvent};
use crate::game::action::handle_action;
use crate::game::attack::AttackInstanceCounter;
use crate::game::damage::handle_damage;
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
use bevy_app::{App, CoreSet, Plugin};
use bevy_ecs::prelude::*;

mod action;
pub(crate) mod attack;
mod damage;
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
pub(crate) mod target;
mod unique;
mod visibility;
mod world;

pub(crate) struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ChatPlugin)
            .insert_resource(PlayerActivity::default())
            .insert_resource(DaylightCycle::official())
            .insert_resource(AttackInstanceCounter::default())
            .add_event::<PlayerLevelUp>()
            .add_event::<LoadingFinishedEvent>()
            .add_event::<UniqueKilledEvent>()
            .add_event::<SpawnDrop>()
            .add_event::<DamageReceiveEvent>()
            .add_event::<EntityDeath>()
            .add_system(update_player_activity.in_base_set(CoreSet::PreUpdate))
            .add_system(handle_inventory_input)
            .add_system(visibility_update)
            .add_system(movement_monster)
            .add_system(tick_drop)
            .add_system(handle_logout)
            .add_system(handle_action)
            .add_system(tick_logout)
            .add_system(player_update_target)
            .add_systems(
                (sync_changes_others, update_client)
                    .in_base_set(CoreSet::PostUpdate)
                    .after(AgentSet::Broadcast),
            )
            .add_system(handle_damage)
            .add_systems(
                (
                    player_visibility_update,
                    notify_levelup,
                    load_finished,
                    unique_spawned,
                    unique_killed,
                    advance_daylight,
                    create_drops,
                )
                    .in_base_set(CoreSet::PostUpdate),
            )
            .add_systems((clean_sync, clear_visibility).in_base_set(CoreSet::Last));
    }
}
