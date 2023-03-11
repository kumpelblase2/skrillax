use crate::agent::event::{ActionFinished, MovementFinished};
use crate::agent::states::{
    action, dead, move_to_action, move_to_pickup, movement, pickup, turning, update_action_destination,
    update_target_location,
};
use crate::agent::system::{
    broadcast_action_stop, broadcast_movement_begin, broadcast_movement_from_action, broadcast_movement_from_pickup,
    broadcast_movement_stop, movement_input, transition_from_attacking, transition_from_idle,
    transition_from_move_to_action, transition_from_move_to_pickup, transition_from_moving, transition_from_sitting,
    transition_to_idle,
};
use bevy_app::{App, CoreSet, Plugin};
use bevy_ecs::prelude::*;
pub(crate) use component::*;

mod component;
mod event;
pub(crate) mod states;
mod system;

pub(crate) struct AgentPlugin;

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub(crate) enum AgentSet {
    Input,
    Transition,
    Execute,
    Broadcast,
}

impl Plugin for AgentPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MovementFinished>()
            .add_event::<ActionFinished>()
            .configure_set(AgentSet::Input.in_base_set(CoreSet::PreUpdate))
            .configure_set(
                AgentSet::Transition
                    .in_base_set(CoreSet::PreUpdate)
                    .after(AgentSet::Input),
            )
            .configure_set(AgentSet::Execute.in_base_set(CoreSet::Update))
            .configure_set(AgentSet::Broadcast.in_base_set(CoreSet::PostUpdate))
            .add_system(movement_input.in_set(AgentSet::Input))
            .add_systems(
                (
                    transition_to_idle,
                    transition_from_idle,
                    transition_from_moving,
                    transition_from_sitting,
                    transition_from_attacking,
                    transition_from_move_to_pickup,
                    transition_from_move_to_action,
                )
                    .in_set(AgentSet::Transition),
            )
            .add_systems(
                (
                    broadcast_movement_stop,
                    broadcast_movement_begin,
                    broadcast_movement_from_pickup,
                    broadcast_movement_from_action,
                    broadcast_action_stop,
                )
                    .in_set(AgentSet::Broadcast),
            )
            .add_systems(
                (
                    update_target_location,
                    movement.after(update_target_location),
                    update_action_destination,
                    move_to_action.after(update_action_destination),
                    move_to_pickup,
                    pickup,
                    action,
                    turning,
                    dead,
                )
                    .in_set(AgentSet::Execute),
            );
    }
}
