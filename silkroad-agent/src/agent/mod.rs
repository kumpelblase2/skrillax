use crate::agent::goal::apply_goal;
use crate::agent::state::{run_transitions, StateTransitionEvent};
use crate::agent::system::{action, movement, movement_input, pickup, turning};
use bevy_app::{App, Plugin, PostUpdate, PreUpdate, Update};
use bevy_ecs::prelude::*;

pub mod component;
pub mod goal;
pub mod state;
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
        app.configure_sets(PreUpdate, AgentSet::Input)
            .configure_sets(Update, (AgentSet::Transition, AgentSet::Execute).chain())
            .configure_sets(PostUpdate, AgentSet::Broadcast)
            .add_systems(PreUpdate, (movement_input, turning).in_set(AgentSet::Input))
            .add_systems(
                Update,
                (apply_goal, run_transitions).chain().in_set(AgentSet::Transition),
            )
            .add_systems(Update, (pickup, movement, action).in_set(AgentSet::Execute));
        app.add_event::<StateTransitionEvent>();
    }
}
