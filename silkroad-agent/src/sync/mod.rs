use crate::comp::exp::{Experienced, Leveled};
use crate::comp::mastery::MasteryKnowledge;
use crate::comp::player::StatPoints;
use crate::comp::pos::Position;
use crate::comp::{Health, Mana};
use crate::sync::reset::AppResetExt;
use crate::sync::system::{
    collect_alives, collect_body_states, collect_deaths, collect_gold_changes, collect_mastery_changes,
    collect_movement_speed_change, collect_movement_update, collect_pickup_animation, collect_stat_changes,
    synchronize_updates, system_collect_bars_update, system_collect_exp_update, system_collect_level_up,
    system_collect_sp_update,
};
use bevy::prelude::*;
use derive_more::From;
pub(crate) use reset::Reset;
use silkroad_protocol::character::CharacterStatsMessage;
use silkroad_protocol::combat::ReceiveExperience;
use silkroad_protocol::movement::{EntityMovementInterrupt, PlayerMovementResponse};
use silkroad_protocol::skill::LevelUpMasteryResponse;
use silkroad_protocol::world::{
    CharacterPointsUpdate, EntityBarsUpdate, EntityUpdateState, LevelUpEffect, PlayerPickupAnimation,
};
use skrillax_stream::packet::{AsPacket, OutgoingPacket};
use std::sync::{mpsc, Mutex};
use system::{collect_movement_starts, collect_movement_transitions};

mod reset;
mod system;

pub(crate) struct Update {
    source: Entity,
    change_self: Option<SelfUpdate>,
    change_others: Option<OtherUpdate>,
}

impl Update {
    pub(crate) fn self_update<T: Into<SelfUpdate>>(entity: Entity, update: T) -> Self {
        Update {
            source: entity,
            change_self: Some(update.into()),
            change_others: None,
        }
    }

    pub(crate) fn update_all<T: Into<SelfUpdate> + Into<OtherUpdate> + Clone>(entity: Entity, update: T) -> Self {
        Update {
            source: entity,
            change_self: Some(update.clone().into()),
            change_others: Some(update.into()),
        }
    }
}

#[derive(From)]
pub(crate) enum SelfUpdate {
    EntityBarsUpdate(EntityBarsUpdate),
    CharacterPointsUpdate(CharacterPointsUpdate),
    ReceiveExperience(ReceiveExperience),
    LevelUpEffect(LevelUpEffect),
    CharacterStatsMessage(CharacterStatsMessage),
    EntityMovementInterrupt(EntityMovementInterrupt),
    PlayerMovementResponse(PlayerMovementResponse),
    EntityUpdateState(EntityUpdateState),
    PlayerPickupAnimation(PlayerPickupAnimation),
    LevelUpMasteryResponse(LevelUpMasteryResponse),
}

impl AsPacket for SelfUpdate {
    fn as_packet(&self) -> OutgoingPacket {
        match self {
            SelfUpdate::EntityBarsUpdate(p) => p.as_packet(),
            SelfUpdate::CharacterPointsUpdate(p) => p.as_packet(),
            SelfUpdate::ReceiveExperience(p) => p.as_packet(),
            SelfUpdate::LevelUpEffect(p) => p.as_packet(),
            SelfUpdate::CharacterStatsMessage(p) => p.as_packet(),
            SelfUpdate::EntityMovementInterrupt(p) => p.as_packet(),
            SelfUpdate::PlayerMovementResponse(p) => p.as_packet(),
            SelfUpdate::EntityUpdateState(p) => p.as_packet(),
            SelfUpdate::PlayerPickupAnimation(p) => p.as_packet(),
            SelfUpdate::LevelUpMasteryResponse(p) => p.as_packet(),
        }
    }
}

#[derive(From, Clone)]
pub(crate) enum OtherUpdate {
    EntityBarsUpdate(EntityBarsUpdate),
    LevelUpEffect(LevelUpEffect),
    EntityMovementInterrupt(EntityMovementInterrupt),
    PlayerMovementResponse(PlayerMovementResponse),
    EntityUpdateState(EntityUpdateState),
    PlayerPickupAnimation(PlayerPickupAnimation),
}

impl AsPacket for OtherUpdate {
    fn as_packet(&self) -> OutgoingPacket {
        match self {
            OtherUpdate::EntityBarsUpdate(p) => p.as_packet(),
            OtherUpdate::LevelUpEffect(p) => p.as_packet(),
            OtherUpdate::EntityMovementInterrupt(p) => p.as_packet(),
            OtherUpdate::PlayerMovementResponse(p) => p.as_packet(),
            OtherUpdate::EntityUpdateState(p) => p.as_packet(),
            OtherUpdate::PlayerPickupAnimation(p) => p.as_packet(),
        }
    }
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
            .configure_sets(
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
                    collect_movement_transitions,
                    collect_movement_speed_change,
                    collect_movement_starts,
                    collect_pickup_animation,
                    collect_deaths,
                    collect_alives,
                    collect_body_states,
                    collect_stat_changes,
                    collect_gold_changes,
                    collect_mastery_changes,
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
            .reset::<StatPoints>()
            .reset::<Leveled>()
            .reset::<MasteryKnowledge>();
    }
}
