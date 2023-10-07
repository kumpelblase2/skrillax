use crate::agent::AgentSet;
use crate::chat::ChatPlugin;
use crate::event::{DamageReceiveEvent, EntityDeath, LoadingFinishedEvent, PlayerLevelUp, UniqueKilledEvent};
use crate::game::action::handle_action;
use crate::game::attack::AttackInstanceCounter;
use crate::game::damage::handle_damage;
use crate::game::daylight::{advance_daylight, DaylightCycle};
use crate::game::drop::{create_drops, tick_drop, SpawnDrop};
use crate::game::entity_sync::{clean_sync, sync_changes_others, update_client};
use crate::game::exp::{distribute_experience, receive_experience, ReceiveExperienceEvent};
use crate::game::inventory::handle_inventory_input;
use crate::game::join::load_finished;
use crate::game::logout::{handle_logout, tick_logout};
use crate::game::mastery::{handle_mastery_levelup, learn_skill};
use crate::game::mind::MindPlugin;
use crate::game::movement::movement_monster;
use crate::game::player_activity::{update_player_activity, PlayerActivity};
use crate::game::stats::increase_stats;
use crate::game::target::player_update_target;
use crate::game::unique::{unique_killed, unique_spawned};
use crate::game::visibility::{clear_visibility, player_visibility_update, visibility_update};
use bevy_app::{App, Last, Plugin, PostUpdate, PreUpdate, Update};
use bevy_ecs::prelude::*;

mod action;
pub(crate) mod attack;
mod damage;
mod daylight;
pub(crate) mod drop;
mod entity_sync;
pub(crate) mod exp;
mod gold;
pub(crate) mod inventory;
mod join;
pub(crate) mod logout;
mod mastery;
pub(crate) mod mind;
mod movement;
pub(crate) mod player_activity;
mod stats;
pub(crate) mod target;
mod unique;
mod visibility;

pub(crate) struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ChatPlugin)
            .add_plugins(MindPlugin)
            .insert_resource(PlayerActivity::default())
            .insert_resource(DaylightCycle::official())
            .insert_resource(AttackInstanceCounter::default())
            .add_event::<PlayerLevelUp>()
            .add_event::<LoadingFinishedEvent>()
            .add_event::<UniqueKilledEvent>()
            .add_event::<SpawnDrop>()
            .add_event::<DamageReceiveEvent>()
            .add_event::<EntityDeath>()
            .add_event::<ReceiveExperienceEvent>()
            .add_systems(PreUpdate, update_player_activity)
            .add_systems(
                Update,
                (
                    handle_inventory_input,
                    increase_stats,
                    visibility_update,
                    movement_monster,
                    tick_drop,
                    handle_logout,
                    handle_action,
                    tick_logout,
                    player_update_target,
                    handle_damage,
                    distribute_experience.after(handle_damage),
                    receive_experience.after(distribute_experience),
                    handle_mastery_levelup,
                    learn_skill,
                ),
            )
            .add_systems(
                PostUpdate,
                (sync_changes_others, update_client).after(AgentSet::Broadcast),
            )
            .add_systems(
                PostUpdate,
                (
                    player_visibility_update,
                    load_finished,
                    unique_spawned,
                    unique_killed,
                    advance_daylight,
                    create_drops,
                ),
            )
            .add_systems(Last, (clean_sync, clear_visibility));
    }
}
