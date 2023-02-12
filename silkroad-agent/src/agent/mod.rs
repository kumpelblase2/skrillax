use crate::agent::event::{ActionFinished, MovementFinished};
use crate::agent::states::{
    action, move_to_action, move_to_pickup, movement, pickup, turning, update_action_destination,
    update_target_location,
};
use crate::agent::system::{
    broadcast_action_stop, broadcast_movement_begin, broadcast_movement_from_action, broadcast_movement_from_pickup,
    broadcast_movement_stop, movement_input, transition_from_attacking, transition_from_idle,
    transition_from_move_to_action, transition_from_move_to_pickup, transition_from_moving, transition_from_sitting,
    transition_to_idle,
};
use bevy_app::{App, CoreStage, Plugin};
use bevy_ecs::prelude::*;
pub(crate) use component::*;

mod component;
mod event;
pub(crate) mod states;
mod system;

pub(crate) struct AgentPlugin;

#[derive(SystemLabel)]
pub(crate) struct AgentTransitionLabel;

#[derive(SystemLabel)]
pub(crate) struct AgentBroadcastLabel;

#[derive(SystemLabel)]
pub(crate) struct AgentInputLabel;

impl Plugin for AgentPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MovementFinished>()
            .add_event::<ActionFinished>()
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::new().label(AgentInputLabel).with_system(movement_input),
            )
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::new()
                    .label(AgentTransitionLabel)
                    .after(AgentInputLabel)
                    .with_system(transition_to_idle)
                    .with_system(transition_from_idle)
                    .with_system(transition_from_moving)
                    .with_system(transition_from_sitting)
                    .with_system(transition_from_attacking)
                    .with_system(transition_from_move_to_pickup)
                    .with_system(transition_from_move_to_action),
            )
            .add_system_set_to_stage(
                CoreStage::PostUpdate,
                SystemSet::new()
                    .label(AgentBroadcastLabel)
                    .with_system(broadcast_movement_stop)
                    .with_system(broadcast_movement_begin)
                    .with_system(broadcast_movement_from_pickup)
                    .with_system(broadcast_movement_from_action)
                    .with_system(broadcast_action_stop),
            )
            .add_system_set_to_stage(
                CoreStage::Update,
                SystemSet::new()
                    .with_system(update_target_location)
                    .with_system(movement.after(update_target_location))
                    .with_system(update_action_destination)
                    .with_system(move_to_action.after(update_action_destination))
                    .with_system(move_to_pickup)
                    .with_system(pickup)
                    .with_system(action)
                    .with_system(turning),
            );
    }
}
