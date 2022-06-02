use crate::comp::player::{Agent, MovementState, MovementTarget};
use crate::comp::pos::{GlobalLocation, Position};
use crate::comp::{Client, GameEntity};
use crate::ext::Vector3Ext;
use bevy_core::Time;
use bevy_ecs::prelude::*;
use cgmath::{InnerSpace, MetricSpace};
use pk2::Pk2;
use silkroad_navmesh::NavmeshLoader;
use silkroad_protocol::world::PlayerMovementResponse;
use silkroad_protocol::ServerPacket;

pub(crate) fn movement(
    mut query: Query<(&GameEntity, &mut Agent, &mut Position, &Client)>,
    mut navmesh: ResMut<NavmeshLoader<Pk2>>,
    delta: Res<Time>,
) {
    for (entity, mut agent, mut position, client) in query.iter_mut() {
        let mut agent: &mut Agent = &mut agent;
        let mut position: &mut Position = &mut position;
        let entity: &GameEntity = entity;
        match &agent.movement_target {
            Some(MovementTarget::Direction(direction)) => {
                todo!()
            },
            Some(MovementTarget::Turn(heading)) => {
                position.rotation = heading.clone();
            },
            Some(MovementTarget::Location(location)) => {
                // if location.0.distance2(position.location.0) > 1. {
                // TODO this does not properly take into account region boundaries
                let current_location_2d = position.location.0.to_flat_vec2();
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
                position.location = new_pos;
                if finished {
                    let local = new_pos.to_local();
                    client.send(ServerPacket::PlayerMovementResponse(PlayerMovementResponse::new(
                        entity.unique_id,
                        local.0.id(),
                        local.1.x as u16,
                        local.1.y as u16,
                        local.1.z as u16,
                        None,
                    )));
                    agent.movement_target = None;
                    agent.movement_state = MovementState::Standing;
                }
            },
            _ => {},
        }
    }
}
