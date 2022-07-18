use crate::comp::player::{Agent, MovementState, MovementTarget};
use crate::comp::pos::{GlobalLocation, Heading, Position};
use crate::comp::sync::{MovementUpdate, Synchronize};
use crate::ext::Vector3Ext;
use bevy_core::Time;
use bevy_ecs::prelude::*;
use cgmath::{Deg, InnerSpace, MetricSpace, Quaternion, Rotation3, Vector2, Vector3};
use pk2::Pk2;
use silkroad_navmesh::NavmeshLoader;

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
