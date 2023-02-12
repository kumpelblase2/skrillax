use crate::agent::event::{ActionFinished, MovementFinished};
use crate::agent::states::{
    Action, Idle, MoveToAction, MoveToPickup, MovementGoal, Moving, Sitting, StateTransitionQueue,
};
use crate::comp::net::Client;
use crate::comp::pos::Position;
use crate::comp::sync::{MovementUpdate, Synchronize};
use crate::input::PlayerInput;
use bevy_ecs::prelude::*;
use cgmath::Vector3;
use silkroad_game_base::{Heading, LocalPosition};
use silkroad_protocol::world::MovementTarget;
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
                    writer.send($event(entity))
                }
            }
        }
    };
}

auto_transition!(transition_from_idle, Idle);
auto_transition!(transition_from_sitting, Sitting);
auto_transition!(transition_from_moving, Moving, MovementFinished);
auto_transition!(transition_from_move_to_action, MoveToAction, MovementFinished);
auto_transition!(transition_from_move_to_pickup, MoveToPickup, MovementFinished);

pub(crate) fn transition_from_attacking(
    mut query: Query<(Entity, &mut StateTransitionQueue, &Action)>,
    mut cmd: Commands,
    mut writer: EventWriter<ActionFinished>,
) {
    for (entity, mut transitions, action) in query.iter_mut() {
        if transitions.transition_to_higher_state::<Action>(entity, &mut cmd) {
            writer.send(ActionFinished(entity))
        } else {
            // TODO: check if action finished
        }
    }
}

pub(crate) fn broadcast_movement_begin(
    mut query: Query<(&mut Synchronize, &Position, &Moving), Or<(Added<Moving>, Changed<Moving>)>>,
) {
    for (mut sync, pos, moving) in query.iter_mut() {
        let update = match &moving.0 {
            MovementGoal::Location(dest) | MovementGoal::Entity(_, dest, _) => {
                MovementUpdate::StartMove(pos.location.to_local(), dest.to_local())
            },
            MovementGoal::Direction(direction) => MovementUpdate::StartMoveTowards(pos.location.to_local(), *direction),
        };

        sync.rotation = Some(update.rotation());
        sync.movement = Some(update);
    }
}

pub(crate) fn broadcast_movement_from_pickup(
    mut query: Query<(&mut Synchronize, &Position, &MoveToPickup), Added<MoveToPickup>>,
) {
    for (mut sync, pos, pickup) in query.iter_mut() {
        let update = MovementUpdate::StartMove(pos.location.to_local(), pickup.1.to_local());
        sync.rotation = Some(update.rotation());
        sync.movement = Some(update);
    }
}

pub(crate) fn broadcast_movement_from_action(
    mut query: Query<(&mut Synchronize, &Position, &MoveToAction), Added<MoveToAction>>,
) {
    for (mut sync, pos, action) in query.iter_mut() {
        let update = MovementUpdate::StartMove(pos.location.to_local(), action.1.to_local());
        sync.rotation = Some(update.rotation());
        sync.movement = Some(update);
    }
}

pub(crate) fn broadcast_movement_stop(
    mut query: Query<(&mut Synchronize, &Position)>,
    mut reader: EventReader<MovementFinished>,
) {
    for finished in reader.iter() {
        if let Ok((mut sync, pos)) = query.get_mut(finished.0) {
            sync.movement = Some(MovementUpdate::StopMove(pos.location.to_local(), pos.rotation));
        }
    }
}

pub(crate) fn broadcast_action_stop(mut query: Query<&mut Synchronize>, mut reader: EventReader<ActionFinished>) {
    for finished in reader.iter() {
        if let Ok(mut sync) = query.get_mut(finished.0) {
            // TODO: we don't have the packet data yet I believe
        }
    }
}

pub(crate) fn movement_input(mut query: Query<(&Client, &PlayerInput, &mut StateTransitionQueue, &Position)>) {
    for (client, input, mut agent, position) in query.iter_mut() {
        if let Some(ref kind) = input.movement {
            match kind {
                MovementTarget::TargetLocation { region, x, y, z } => {
                    let local_position = position.location.to_local();
                    let target_pos = LocalPosition((*region).into(), Vector3::new(*x as f32, *y as f32, *z as f32));
                    debug!(id = ?client.id(), "Movement: {} -> {}", local_position, target_pos);
                    agent.request_transition(Moving(MovementGoal::Location(target_pos.to_global())));
                },
                MovementTarget::Direction { unknown, angle } => {
                    let direction = Heading::from(*angle);
                    debug!(id = ?client.id(), "Movement: {} / {}({})", unknown, direction.0, angle);
                    agent.request_transition(Moving(MovementGoal::Direction(direction)));
                },
            }
        }
    }
}
