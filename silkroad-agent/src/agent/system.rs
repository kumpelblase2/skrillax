use crate::agent::event::{ActionFinished, MovementFinished};
use crate::agent::states::{Action, Idle, MovementGoal, Moving, Sitting, StateTransitionQueue};
use crate::comp::net::Client;
use crate::comp::pos::Position;
use crate::input::PlayerInput;
use bevy_ecs::prelude::*;
use cgmath::Vector3;
use silkroad_game_base::{Heading, LocalPosition};
use silkroad_protocol::movement::MovementTarget;
use tracing::debug;

pub(crate) fn transition_to_idle(
    mut query: Query<Entity, (Without<Idle>, Without<Moving>, Without<Action>, Without<Sitting>)>,
    mut cmd: Commands,
) {
    for entity in query.iter_mut() {
        cmd.entity(entity).insert(Idle);
    }
}

macro_rules! auto_transition {
    ($system:ident, $component:ty) => {
        pub(crate) fn $system(
            mut query: Query<(Entity, &mut StateTransitionQueue), With<$component>>,
            mut cmd: Commands,
        ) {
            for (entity, mut transitions) in query.iter_mut() {
                transitions.transition_to_new_state::<$component>(entity, &mut cmd);
            }
        }
    };
    ($system:ident, $component:ty, $event:tt) => {
        pub(crate) fn $system(
            mut query: Query<(Entity, &mut StateTransitionQueue), With<$component>>,
            mut writer: EventWriter<$event>,
            mut cmd: Commands,
        ) {
            for (entity, mut transitions) in query.iter_mut() {
                if transitions.transition_to_new_state::<$component>(entity, &mut cmd) {
                    writer.send($event(entity));
                }
            }
        }
    };
}

auto_transition!(transition_from_idle, Idle);
auto_transition!(transition_from_sitting, Sitting);
auto_transition!(transition_from_moving, Moving, MovementFinished);

pub(crate) fn transition_from_attacking(
    mut query: Query<(Entity, &mut StateTransitionQueue, &Action)>,
    mut cmd: Commands,
    mut writer: EventWriter<ActionFinished>,
) {
    for (entity, mut transitions, action) in query.iter_mut() {
        if transitions.transition_to_higher_state::<Action>(entity, &mut cmd) {
            writer.send(ActionFinished(entity));
        } else {
            // TODO: check if action finished
        }
    }
}

pub(crate) fn movement_input(mut query: Query<(&Client, &PlayerInput, &mut StateTransitionQueue, &Position)>) {
    for (client, input, mut agent, position) in query.iter_mut() {
        if let Some(kind) = input.movement {
            match kind {
                MovementTarget::TargetLocation { region, x, y, z } => {
                    let local_position = position.position().to_local();
                    let target_pos = LocalPosition(region.into(), Vector3::new(x.into(), y.into(), z.into()));
                    debug!(id = ?client.id(), "Movement: {} -> {}", local_position, target_pos);
                    agent.request_transition(Moving(MovementGoal::Location(target_pos.to_global())));
                },
                MovementTarget::Direction { unknown, angle } => {
                    let direction = Heading::from(angle);
                    debug!(id = ?client.id(), "Movement: {} / {}({})", unknown, direction.0, angle);
                    agent.request_transition(Moving(MovementGoal::Direction(direction)));
                },
            }
        }
    }
}
