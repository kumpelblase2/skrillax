use crate::comp::exp::{Experienced, Leveled};
use crate::comp::pos::Position;
use crate::comp::{Health, Mana};
use crate::sync::reset::AppResetExt;
use crate::sync::system::{
    collect_alives, collect_body_states, collect_deaths, collect_movement_speed_change, collect_movement_update,
    collect_pickup_animation, synchronize_updates, system_collect_bars_update, system_collect_exp_update,
    system_collect_level_up, system_collect_sp_update,
};
use bevy_app::{App, Plugin, PostUpdate};
use bevy_ecs::prelude::*;
use bevy_ecs_macros::{Resource, SystemSet};
pub(crate) use reset::Reset;
use silkroad_protocol::ServerPacket;
use std::sync::{mpsc, Mutex};

mod reset;
mod system;

pub(crate) struct Update {
    source: Entity,
    change_self: Option<ServerPacket>,
    change_others: Option<ServerPacket>,
}

#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub(crate) enum SynchronizationStage {
    Collection,
    Distribution,
}

#[derive(Resource)]
pub(crate) struct SynchronizationCollector {
    receiver: Mutex<mpsc::Receiver<Update>>,
    sender: mpsc::Sender<Update>,
}

impl SynchronizationCollector {
    fn send_update(&self, update: Update) {
        self.sender.send(update).expect("The receiver should still be alive.");
    }

    fn collect_updates(&mut self) -> Vec<Update> {
        self.receiver
            .lock()
            .expect("Should not be poisoned")
            .try_iter()
            .collect::<Vec<_>>()
    }
}

impl Default for SynchronizationCollector {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        SynchronizationCollector {
            receiver: Mutex::new(receiver),
            sender,
        }
    }
}

pub(crate) struct SynchronizationPlugin;

impl Plugin for SynchronizationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SynchronizationCollector>()
            .configure_set(
                PostUpdate,
                SynchronizationStage::Distribution.after(SynchronizationStage::Collection),
            )
            .add_systems(
                PostUpdate,
                (
                    system_collect_bars_update,
                    system_collect_sp_update,
                    system_collect_exp_update,
                    system_collect_level_up,
                    collect_movement_update,
                    collect_movement_speed_change,
                    collect_pickup_animation,
                    collect_deaths,
                    collect_alives,
                    collect_body_states,
                )
                    .in_set(SynchronizationStage::Collection),
            )
            .add_systems(
                PostUpdate,
                synchronize_updates.in_set(SynchronizationStage::Distribution),
            )
            .reset::<Position>()
            .reset::<Health>()
            .reset::<Mana>()
            .reset::<Experienced>()
            .reset::<Leveled>();
    }
}
