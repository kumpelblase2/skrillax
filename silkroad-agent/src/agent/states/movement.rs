use crate::agent::event::MovementFinished;
use crate::agent::states::Idle;
use crate::agent::{Agent, MovementState};
use crate::comp::pos::Position;
use crate::comp::sync::Synchronize;
use crate::config::GameConfig;
use crate::ext::Navmesh;
use crate::input::PlayerInput;
use bevy_ecs::prelude::*;
use bevy_ecs::query::QueryEntityError;
use bevy_time::Time;
use cgmath::num_traits::Pow;
use cgmath::{Array, Deg, InnerSpace, Quaternion, Rotation3, Vector3, Zero};
use silkroad_game_base::{GlobalLocation, GlobalPosition, Heading, Vector3Ext};
use std::ops::Deref;
use tracing::debug;

const EPSYLON: f32 = 1.0;

pub(crate) enum MovementGoal {
    Location(GlobalPosition),
    Direction(Heading),
    Entity(Entity, GlobalPosition, f32),
}

#[derive(Component)]
pub(crate) struct Moving(pub MovementGoal);

pub(crate) fn update_target_location(
    mut query: Query<(Entity, &mut Moving, &Position)>,
    target_query: Query<&Position>,
    settings: Res<GameConfig>,
    mut navmesh: ResMut<Navmesh>,
    mut cmd: Commands,
    mut stopped: EventWriter<MovementFinished>,
) {
    for (itself, mut moving, own_position) in query.iter_mut() {
        if let MovementGoal::Entity(target, old, distance) = &moving.0 {
            match target_query.get(*target) {
                Ok(pos) => {
                    if own_position.distance_to(pos) > settings.max_follow_distance.pow(2) {
                        cmd.entity(itself).remove::<Moving>().insert(Idle);
                        stopped.send(MovementFinished(itself));
                    } else {
                        let dir_vector = pos.location.to_flat_vec2() - own_position.location.to_flat_vec2();
                        let target_vector = dir_vector.normalize() * (*distance);
                        let target_location = GlobalLocation(pos.location.to_flat_vec2() - target_vector);
                        let height = navmesh.height_for(target_location).unwrap_or(old.y);
                        moving.0 = MovementGoal::Entity(*target, target_location.with_y(height), *distance);
                    }
                },
                Err(QueryEntityError::NoSuchEntity(_)) => {
                    cmd.entity(itself).remove::<Moving>().insert(Idle);
                    stopped.send(MovementFinished(itself));
                },
                Err(e) => {
                    debug!("Could not update entity position: {:?}", e);
                },
            }
        }
    }
}

pub(crate) fn turning(mut query: Query<(&mut Synchronize, &mut Position, &PlayerInput), With<Idle>>) {
    for (mut sync, mut pos, input) in query.iter_mut() {
        if let Some(ref rotate) = input.rotation {
            pos.rotation = Heading::from(rotate.heading);
            sync.rotation = Some(pos.rotation);
        }
    }
}

pub(crate) fn movement(
    mut query: Query<(Entity, &mut Position, &Agent, &Moving, &MovementState)>,
    time: Res<Time>,
    mut cmd: Commands,
    mut navmesh: ResMut<Navmesh>,
    mut finish_movement: EventWriter<MovementFinished>,
) {
    let delta = time.delta_seconds_f64() as f32;
    for (entity, mut pos, agent, movement, speed_state) in query.iter_mut() {
        let speed = agent.get_speed_value(*speed_state.deref());
        let (next_location, heading, finished) = match movement.0 {
            MovementGoal::Location(location) => {
                get_next_step(delta, pos.location.to_location(), speed, location.to_location())
            },
            MovementGoal::Direction(direction) => {
                let current_location_2d = pos.location.0.to_flat_vec2();
                let direction_vec = Quaternion::from_angle_y(Deg(direction.0)) * Vector3::unit_x();
                let direction_vec = direction_vec.to_flat_vec2().normalize();
                let movement = direction_vec * (speed * delta);
                (GlobalLocation(current_location_2d + movement), direction, false)
            },
            MovementGoal::Entity(_, location, _) => {
                get_next_step(delta, pos.location.to_location(), speed, location.to_location())
            },
        };

        move_with_step(&mut navmesh, &mut pos, next_location, heading);

        if finished {
            cmd.entity(entity).remove::<Moving>().insert(Idle);
            finish_movement.send(MovementFinished(entity));
        }
    }
}

pub(crate) fn move_with_step(navmesh: &mut Navmesh, pos: &mut Position, target: GlobalLocation, heading: Heading) {
    let target_location = target.to_local();
    let height = navmesh.height_for(target_location).unwrap_or(pos.location.0.y);

    pos.location = target.with_y(height);
    pos.rotation = heading;
}

pub(crate) fn get_next_step(
    time_delta: f32,
    current_location: GlobalLocation,
    speed: f32,
    target: GlobalLocation,
) -> (GlobalLocation, Heading, bool) {
    let direction = target.0 - current_location.0;
    let distance_travelled = speed * time_delta * direction.normalize();
    let angle = Heading::from(direction);
    if !direction.is_finite()
        || direction.is_zero()
        || direction.magnitude2() < EPSYLON
        || distance_travelled.magnitude2() > direction.magnitude2()
    {
        (target, angle, true)
    } else {
        let new_location = current_location + distance_travelled;
        (new_location, angle, false)
    }
}
