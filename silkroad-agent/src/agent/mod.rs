use crate::agent::event::{ActionFinished, MovementFinished};
use crate::agent::states::{action, broadcast_dead, dead, movement, pickup, turning, update_target_location};
use crate::agent::system::{
    broadcast_action_stop, broadcast_movement_begin, broadcast_movement_stop, movement_input,
    transition_from_attacking, transition_from_idle, transition_from_moving, transition_from_sitting,
    transition_to_idle,
};
use bevy_app::{App, Plugin, PostUpdate, PreUpdate, Update};
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
            .configure_set(PreUpdate, AgentSet::Input)
            .configure_set(Update, AgentSet::Transition)
            .configure_set(Update, AgentSet::Execute.after(AgentSet::Transition))
            .configure_set(PostUpdate, AgentSet::Broadcast)
            .add_systems(PreUpdate, movement_input.in_set(AgentSet::Input))
            .add_systems(
                Update,
                (
                    transition_to_idle,
                    transition_from_idle,
                    transition_from_moving,
                    transition_from_sitting,
                    transition_from_attacking,
                )
                    .in_set(AgentSet::Transition),
            )
            .add_systems(
                PostUpdate,
                (
                    broadcast_movement_stop,
                    broadcast_movement_begin,
                    broadcast_action_stop,
                    broadcast_dead,
                )
                    .in_set(AgentSet::Broadcast),
            )
            .add_systems(
                Update,
                (
                    update_target_location,
                    movement.after(update_target_location),
                    pickup,
                    action,
                    turning,
                    dead,
                )
                    .in_set(AgentSet::Execute),
            );
    }
}
