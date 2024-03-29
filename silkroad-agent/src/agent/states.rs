mod action;
mod dead;
mod idle;
mod movement;
mod sitting;

pub(crate) use action::*;
use bevy_ecs::prelude::*;
pub(crate) use dead::*;
use derive_more::{Deref, DerefMut};
pub(crate) use idle::*;
pub(crate) use movement::*;
pub(crate) use sitting::*;
use std::collections::VecDeque;

pub(crate) trait State {
    const ORDER: usize;
    const INTERRUPTABLE: bool;
}

pub(crate) enum StateChange {
    Idle(Idle),
    Sitting(Sitting),
    Moving(Moving),
    Action(Action),
    Pickup(Pickup),
    Dead(Dead),
}

impl StateChange {
    pub fn apply(self, entity: Entity, cmd: &mut Commands) {
        // TODO: make this not necessary. We need this because otherwise the entity could be despawned,
        //  e.g. due to being out of range of a player, while trying to apply a new state.
        //  Instead we should maybe ensure that despawns happen at specific 'checkpoints' which don't interfer
        //  with this kind of code.
        let Some(mut entity_cmd) = cmd.get_entity(entity) else {
            return;
        };
        match self {
            StateChange::Idle(inner) => entity_cmd.insert(inner),
            StateChange::Sitting(inner) => entity_cmd.insert(inner),
            StateChange::Moving(inner) => entity_cmd.insert(inner),
            StateChange::Action(inner) => entity_cmd.insert(inner),
            StateChange::Dead(inner) => entity_cmd.insert(inner),
            StateChange::Pickup(inner) => entity_cmd.insert(inner),
        };
    }
}

macro_rules! impl_state {
    ($kind:tt, $prio:literal, $inter:literal) => {
        impl From<$kind> for StateChange {
            fn from(state: $kind) -> StateChange {
                StateChange::$kind(state)
            }
        }

        impl State for $kind {
            const ORDER: usize = $prio;
            const INTERRUPTABLE: bool = $inter;
        }
    };
}
impl_state!(Idle, 0, true);
impl_state!(Moving, 1, true);
impl_state!(Action, 2, false);
impl_state!(Sitting, 1, true);
impl_state!(Pickup, 2, false);
impl_state!(Dead, 3, false);

pub(crate) struct StateTransition {
    data: StateChange,
    priority: usize,
}

impl StateTransition {
    pub(crate) fn new<T: State + Into<StateChange>>(component: T) -> Self {
        Self {
            data: component.into(),
            priority: T::ORDER,
        }
    }

    pub(crate) fn replace<T: Component>(self, target: Entity, cmd: &mut Commands) {
        cmd.entity(target).remove::<T>();
        self.data.apply(target, cmd);
    }
}

#[derive(Component, Deref, DerefMut, Default)]
pub(crate) struct StateTransitionQueue(VecDeque<StateTransition>);

impl StateTransitionQueue {
    pub(crate) fn transition_to_new_state<T: Component>(&mut self, entity: Entity, cmd: &mut Commands) -> bool {
        if self.len() > 0 {
            let mut transitions = self.drain(..).collect::<Vec<_>>();
            transitions.sort_by(|a, b| b.priority.cmp(&a.priority));
            let chosen_transition = transitions.remove(0);
            chosen_transition.replace::<T>(entity, cmd);
            true
        } else {
            false
        }
    }

    pub(crate) fn transition_to_higher_state<T: Component + State>(
        &mut self,
        entity: Entity,
        cmd: &mut Commands,
    ) -> bool {
        if self.len() > 0 {
            let mut transitions = self
                .drain(..)
                .filter(|trans| trans.priority > T::ORDER)
                .collect::<Vec<_>>();
            transitions.sort_by(|a, b| b.priority.cmp(&a.priority));
            if !transitions.is_empty() {
                let chosen_transition = transitions.remove(0);
                chosen_transition.replace::<T>(entity, cmd);
                return true;
            }
        }
        false
    }

    pub(crate) fn request_transition<T: State + Into<StateChange>>(&mut self, component: T) {
        self.0.push_back(StateTransition::new(component));
    }
}
