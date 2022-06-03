use crate::comp::player::{Agent, MovementState, MovementTarget};
use crate::comp::pos::{GlobalLocation, Position};
use crate::comp::sync::{MovementUpdate, Synchronize};
use crate::ext::Vector3Ext;
use bevy_core::Time;
use bevy_ecs::prelude::*;
use cgmath::{InnerSpace, MetricSpace};
use pk2::Pk2;
use silkroad_navmesh::NavmeshLoader;

pub(crate) fn movement(
    mut query: Query<(&mut Agent, &mut Position, &mut Synchronize)>,
    mut navmesh: ResMut<NavmeshLoader<Pk2>>,
    delta: Res<Time>,
) {
    for (mut agent, mut position, mut sync) in query.iter_mut() {
        let mut agent: &mut Agent = &mut agent;
        let mut position: &mut Position = &mut position;
        let mut sync: &mut Synchronize = &mut sync;
        match &agent.movement_target {
            Some(MovementTarget::Direction(_direction)) => {
                todo!()
            },
            Some(MovementTarget::Turn(heading)) => {
                position.rotation = heading.clone();
                sync.movement = Some(MovementUpdate::Turn(heading.clone()));
            },
            Some(MovementTarget::Location(location)) => {
                // if location.0.distance2(position.location.0) > 1. {
                let old_position = position.location.clone();
                let current_location_2d = old_position.0.to_flat_vec2();
                let to_target = location.0.to_flat_vec2() - current_location_2d;
                let direction = to_target.normalize();
                let movement = direction * (agent.movement_speed * delta.delta_seconds_f64() as f32);
                let next_step = current_location_2d + movement;
                let (next_step, finished) = if current_location_2d.distance2(location.0.to_flat_vec2())
                    <= current_location_2d.distance2(next_step)
                {
                    (location.0.to_flat_vec2(), true)
                } else {
                    (next_step, false)
                };

                let target_location = GlobalLocation(next_step).to_local();

                let mesh = navmesh.load_navmesh(target_location.0).unwrap();
                let height = mesh
                    .heightmap()
                    .height_at_position(target_location.1.x, target_location.1.y)
                    .unwrap_or(position.location.0.y);

                let new_pos = target_location.to_global().with_y(height);
                position.location = new_pos.clone();
                if finished {
                    let local = new_pos.to_local();
                    agent.movement_target = None;
                    agent.movement_state = MovementState::Standing;
                    sync.movement = Some(MovementUpdate::StopMove(local));
                } else if !matches!(agent.movement_state, MovementState::Moving | MovementState::Walking) {
                    let old_local = old_position.to_local();
                    agent.movement_state = MovementState::Moving;
                    sync.movement = Some(MovementUpdate::StartMove(old_local, new_pos.to_local()));
                }
            },
            _ => {},
        }
    }
}
