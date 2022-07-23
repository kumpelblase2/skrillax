use crate::comp::monster::{Monster, RandomStroll};
use crate::comp::net::MovementInput;
use crate::comp::player::{Agent, MovementState, MovementTarget};
use crate::comp::pos::{GlobalLocation, Heading, LocalPosition, Position};
use crate::comp::sync::{MovementUpdate, Synchronize};
use crate::comp::Client;
use crate::ext::Vector3Ext;
use crate::math::random_point_in_circle;
use bevy_core::Time;
use bevy_ecs::prelude::*;
use cgmath::{Deg, InnerSpace, MetricSpace, Quaternion, Rotation3, Vector2, Vector3};
use pk2::Pk2;
use rand::random;
use silkroad_navmesh::NavmeshLoader;
use silkroad_protocol::world::{PlayerMovementRequest, Rotation};
use silkroad_protocol::ClientPacket;
use std::mem::take;
use tracing::debug;

pub(crate) fn movement_input(mut query: Query<(&Client, &mut MovementInput, &mut Agent, &Position, &mut Synchronize)>) {
    for (client, mut input, mut agent, position, mut sync) in query.iter_mut() {
        for packet in take(&mut input.inputs) {
            match packet {
                ClientPacket::PlayerMovementRequest(PlayerMovementRequest { kind }) => match kind {
                    silkroad_protocol::world::MovementTarget::TargetLocation { region, x, y, z } => {
                        let local_position = position.location.to_local();
                        let target_pos = LocalPosition(region.into(), Vector3::new(x as f32, y as f32, z as f32));
                        debug!(id = ?client.0.id(), "Movement: {} -> {}", local_position, target_pos);
                        sync.movement = Some(MovementUpdate::StartMove(local_position, target_pos.clone()));
                        agent.movement_target = Some(MovementTarget::Location(target_pos.to_global()));
                        agent.movement_state = MovementState::Moving;
                    },
                    silkroad_protocol::world::MovementTarget::Direction { unknown, angle } => {
                        let direction = Heading::from(angle);
                        debug!(id = ?client.0.id(), "Movement: {} / {}({})", unknown, direction.0, angle);
                        let local_position = position.location.to_local();
                        sync.movement = Some(MovementUpdate::StartMoveTowards(local_position, direction.clone()));
                        agent.movement_target = Some(MovementTarget::Direction(direction));
                    },
                },
                ClientPacket::Rotation(Rotation { heading }) => {
                    let heading = Heading::from(heading);
                    if agent.movement_target.is_none() {
                        agent.movement_target = Some(MovementTarget::Turn(heading));
                    }
                },
                _ => {},
            }
        }
    }
}

pub(crate) fn movement(
    mut query: Query<(&mut Agent, &mut Position, &mut Synchronize)>,
    mut navmesh: ResMut<NavmeshLoader<Pk2>>,
    delta: Res<Time>,
) {
    for (mut agent, mut position, mut sync) in query.iter_mut() {
        match &agent.movement_target {
            Some(MovementTarget::Direction(direction)) => {
                let old_position = position.location.clone();
                let current_location_2d = old_position.0.to_flat_vec2();
                let direction_vec = Quaternion::from_angle_y(Deg(direction.0)) * Vector3::unit_x();
                let direction_vec = direction_vec.to_flat_vec2().normalize();
                let movement = direction_vec * (agent.movement_speed * delta.delta_seconds_f64() as f32);
                let next_step = current_location_2d + movement;
                let target_location = GlobalLocation(next_step).to_local();
                let mesh = navmesh.load_navmesh(target_location.0).unwrap();
                let height = mesh
                    .heightmap()
                    .height_at_position(target_location.1.x, target_location.1.y)
                    .unwrap_or(old_position.0.y);

                let new_pos = target_location.to_global().with_y(height);
                if !matches!(agent.movement_state, MovementState::Moving | MovementState::Walking) {
                    position.rotation = direction.clone();
                    agent.movement_state = MovementState::Moving;
                }
                position.location = new_pos;
            },
            Some(MovementTarget::Turn(heading)) => {
                position.rotation = heading.clone();
                sync.movement = Some(MovementUpdate::Turn(heading.clone()));
                agent.movement_target = None;
            },
            Some(MovementTarget::Location(location)) => {
                let old_position = position.location.clone();
                let current_location_2d = old_position.0.to_flat_vec2();
                let to_target = location.0.to_flat_vec2() - current_location_2d;
                let direction = to_target.normalize();
                let target = location.0.to_flat_vec2();

                let current_location_2d = old_position.0.to_flat_vec2();
                let movement = direction * (agent.movement_speed * delta.delta_seconds_f64() as f32);
                let next_step = current_location_2d + movement;
                let (next_step, finished) =
                    if current_location_2d.distance2(target) <= current_location_2d.distance2(next_step) {
                        (target, true)
                    } else {
                        (next_step, false)
                    };

                let target_location = GlobalLocation(next_step).to_local();

                let mesh = navmesh.load_navmesh(target_location.0).unwrap();
                let height = mesh
                    .heightmap()
                    .height_at_position(target_location.1.x, target_location.1.y)
                    .unwrap_or(old_position.0.y);

                let new_pos = target_location.to_global().with_y(height);
                if finished {
                    let local = new_pos.to_local();
                    agent.movement_target = None;
                    agent.movement_state = MovementState::Standing;
                    sync.movement = Some(MovementUpdate::StopMove(local, position.rotation.clone()));
                } else if !matches!(agent.movement_state, MovementState::Moving | MovementState::Walking) {
                    let angle = Deg::from(direction.angle(Vector2::unit_x()));
                    position.rotation = Heading(angle.0);
                    agent.movement_state = MovementState::Moving;
                }

                position.location = new_pos;
            },
            _ => {},
        }
    }
}

pub(crate) fn movement_monster(
    mut query: Query<(&mut Agent, &mut RandomStroll, &Position), With<Monster>>,
    delta: Res<Time>,
    mut navmesh: ResMut<NavmeshLoader<Pk2>>,
) {
    let delta = delta.delta();
    for (mut agent, mut stroll, pos) in query.iter_mut() {
        if agent.movement_target.is_none() && matches!(agent.movement_state, MovementState::Standing) {
            stroll.check_timer.tick(delta);
            if stroll.check_timer.finished() && random::<f32>() <= 0.1 {
                let new_location = GlobalLocation(random_point_in_circle(stroll.origin.0, stroll.radius));
                let new_y = navmesh
                    .load_navmesh(new_location.to_local().0)
                    .ok()
                    .and_then(|mesh| mesh.heightmap().height_at_position(new_location.0.x, new_location.0.y))
                    .unwrap_or(pos.location.0.y);
                let new_location = new_location.with_y(new_y);
                agent.movement_target = Some(MovementTarget::Location(new_location));
            }
        }
    }
}
