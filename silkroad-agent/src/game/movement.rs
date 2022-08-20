use crate::comp::monster::{Monster, RandomStroll};
use crate::comp::net::MovementInput;
use crate::comp::player::{Agent, AgentAction, MovementState, MovementTarget, SkillState};
use crate::comp::pos::{GlobalLocation, GlobalPosition, Heading, LocalPosition, Position};
use crate::comp::sync::{MovementUpdate, Synchronize};
use crate::comp::Client;
use crate::ext::{Vector2Ext, Vector3Ext};
use bevy_core::{Time, Timer};
use bevy_ecs::prelude::*;
use cgmath::num_traits::Pow;
use cgmath::{Deg, InnerSpace, MetricSpace, Quaternion, Rotation3, Vector2, Vector3};
use pk2::Pk2;
use rand::random;
use silkroad_data::skilldata::RefSkillData;
use silkroad_navmesh::NavmeshLoader;
use silkroad_protocol::world::{PlayerMovementRequest, Rotation};
use silkroad_protocol::ClientPacket;
use std::mem;
use std::time::Duration;
use tracing::debug;

const EPSYLON: f32 = 1.0;

pub(crate) fn movement_input(mut query: Query<(&Client, &mut MovementInput, &mut Agent, &Position, &mut Synchronize)>) {
    for (client, mut input, mut agent, position, mut sync) in query.iter_mut() {
        for packet in mem::take(&mut input.inputs) {
            match packet {
                ClientPacket::PlayerMovementRequest(PlayerMovementRequest { kind }) => match kind {
                    silkroad_protocol::world::MovementTarget::TargetLocation { region, x, y, z } => {
                        let local_position = position.location.to_local();
                        let target_pos = LocalPosition(region.into(), Vector3::new(x as f32, y as f32, z as f32));
                        debug!(id = ?client.0.id(), "Movement: {} -> {}", local_position, target_pos);
                        sync.movement = Some(MovementUpdate::StartMove(local_position, target_pos.clone()));
                        agent.move_to(target_pos);
                    },
                    silkroad_protocol::world::MovementTarget::Direction { unknown, angle } => {
                        let direction = Heading::from(angle);
                        debug!(id = ?client.0.id(), "Movement: {} / {}({})", unknown, direction.0, angle);
                        let local_position = position.location.to_local();
                        sync.movement = Some(MovementUpdate::StartMoveTowards(local_position, direction.clone()));
                        agent.move_in_direction(direction);
                    },
                },
                ClientPacket::Rotation(Rotation { heading }) => {
                    let heading = Heading::from(heading);
                    agent.turn(heading);
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
        if let Some(mut action) = agent.current_action.as_mut() {
            match action {
                AgentAction::Movement(target) => match target {
                    MovementTarget::Direction(direction) => {
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

                        position.location = target_location.to_global().with_y(height);
                    },
                    MovementTarget::Turn(heading) => {
                        position.rotation = heading.clone();
                        agent.finish_current_action();
                    },
                    MovementTarget::Location(location) => {
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
                            agent.finish_current_action();
                            let local = new_pos.to_local();
                            sync.movement = Some(MovementUpdate::StopMove(local, position.rotation.clone()));
                        }

                        let angle = Deg::from(direction.angle(Vector2::unit_x()));
                        position.rotation = Heading(angle.0);
                        position.location = new_pos;
                    },
                },
                AgentAction::Skill {
                    reference,
                    target,
                    state,
                } => {
                    let next_state = match state {
                        SkillState::Before => {
                            // TODO: send skill start info
                            if reference.timings.preparation_time > 0 {
                                SkillState::Prepare(Timer::new(
                                    Duration::from_millis(reference.timings.preparation_time as u64),
                                    false,
                                ))
                            } else if reference.timings.cast_time > 0 {
                                SkillState::Cast(Timer::new(
                                    Duration::from_millis(reference.timings.preparation_time as u64),
                                    false,
                                ))
                            } else {
                                SkillState::Default(Timer::new(
                                    Duration::from_millis(reference.timings.preparation_time as u64),
                                    false,
                                ))
                            }
                        },
                        SkillState::Prepare(timer) => {
                            timer.tick(delta.delta());
                            if timer.finished() {
                                if reference.timings.cast_time > 0 {
                                    SkillState::Cast(Timer::new(
                                        Duration::from_millis(reference.timings.preparation_time as u64),
                                        false,
                                    ))
                                } else {
                                    SkillState::Default(Timer::new(
                                        Duration::from_millis(reference.timings.preparation_time as u64),
                                        false,
                                    ))
                                }
                            } else {
                                SkillState::Prepare(mem::take(timer))
                            }
                        },
                        SkillState::Cast(timer) => {
                            timer.tick(delta.delta());
                            if timer.finished() {
                                SkillState::Default(Timer::new(
                                    Duration::from_millis(reference.timings.preparation_time as u64),
                                    false,
                                ))
                            } else {
                                SkillState::Cast(mem::take(timer))
                            }
                        },
                        _ => todo!(),
                    };
                    let _ = mem::replace(state, next_state);
                },
                AgentAction::Attack {
                    current_destination,
                    reference,
                    target,
                    ..
                } => {
                    let reference = *reference;
                    let target = *target;
                    let location = current_destination;
                    let old_position = position.location.clone();
                    let current_location_2d = old_position.0.to_flat_vec2();
                    let to_target = location.0.to_flat_vec2() - current_location_2d;
                    let direction = to_target.normalize();
                    let target_location = location.0.to_flat_vec2();

                    let current_location_2d = old_position.0.to_flat_vec2();
                    let movement = direction * (agent.movement_speed * delta.delta_seconds_f64() as f32);
                    let next_step = current_location_2d + movement;
                    let (next_step, finished) =
                        if current_location_2d.distance2(target_location) <= current_location_2d.distance2(next_step) {
                            (target_location, true)
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
                        agent.movement_state = MovementState::Standing;
                        agent.current_action = Some(AgentAction::Skill {
                            reference,
                            target: Some(target),
                            state: SkillState::Before,
                        });
                        let local = new_pos.to_local();
                        sync.movement = Some(MovementUpdate::StopMove(local, position.rotation.clone()));
                    }

                    let angle = Deg::from(direction.angle(Vector2::unit_x()));
                    position.rotation = Heading(angle.0);
                    position.location = new_pos;
                },
            }
        }

        if agent
            .current_action
            .as_ref()
            .map(|action| action.can_cancel())
            .unwrap_or(true)
        {
            if let Some(mut next_action) = agent.next_action.take() {
                if matches!(
                    agent.current_action,
                    Some(AgentAction::Movement(MovementTarget::Direction(_)))
                ) && !matches!(next_action, AgentAction::Movement(_))
                {
                    sync.movement = Some(MovementUpdate::StopMove(
                        position.location.to_local(),
                        position.rotation,
                    ));
                }

                match &next_action {
                    AgentAction::Movement(target) => {
                        let update = match target {
                            MovementTarget::Location(pos) => {
                                MovementUpdate::StartMove(position.location.to_local(), pos.to_local())
                            },
                            MovementTarget::Direction(dir) => {
                                position.rotation = *dir;
                                MovementUpdate::StartMoveTowards(position.location.to_local(), *dir)
                            },
                            MovementTarget::Turn(heading) => MovementUpdate::Turn(heading.clone()),
                        };
                        sync.movement = Some(update);
                    },
                    AgentAction::Skill {
                        reference,
                        target,
                        state,
                    } => todo!("send client packet"),
                    AgentAction::Attack {
                        current_destination,
                        reference,
                        target,
                        ..
                    } => {
                        if current_destination.0.distance2(position.location.0) > EPSYLON {
                            sync.movement = Some(MovementUpdate::StartMove(
                                position.location.to_local(),
                                current_destination.to_local(),
                            ));
                        } else {
                            next_action = AgentAction::Skill {
                                reference,
                                target: Some(*target),
                                state: SkillState::Before,
                            };
                        }
                    },
                }
                agent.current_action = Some(next_action);
            }
        }
    }
}

pub(crate) fn update_attack_location(
    mut selector: Query<(&mut Agent, &Position, &mut Synchronize)>,
    target_query: Query<&Position>,
    time: Res<Time>,
) {
    for (mut agent, position, mut sync) in selector.iter_mut() {
        if let Some(current_action) = agent.current_action.as_mut() {
            match current_action {
                AgentAction::Attack {
                    current_destination,
                    target,
                    range,
                    reference,
                    refresh_timer,
                } => {
                    refresh_timer.tick(time.delta());
                    if refresh_timer.finished() {
                        let target_position = target_query.get(*target).unwrap();
                        let current_distance = current_destination.0.distance2(target_position.location.0);
                        if current_distance > (range.pow(2) - EPSYLON) {
                            let direction = target_position.location.0 - position.location.0;
                            let new_pos = direction.normalize() * (*range - EPSYLON);
                            let target_position = GlobalPosition(new_pos);
                            sync.movement = Some(MovementUpdate::StartMove(
                                position.location.to_local(),
                                target_position.to_local(),
                            ));
                            let new = AgentAction::Attack {
                                target: *target,
                                range: *range,
                                reference,
                                current_destination: target_position,
                                refresh_timer: mem::take(refresh_timer),
                            };
                            let _ = mem::replace(current_action, new);
                        }
                    }
                },
                _ => continue,
            }
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
        if !agent.is_in_action() {
            stroll.check_timer.tick(delta);
            if stroll.check_timer.finished() && random::<f32>() <= 0.1 {
                let new_location = GlobalLocation(stroll.origin.0.random_in_radius(stroll.radius));
                let new_y = navmesh
                    .load_navmesh(new_location.to_local().0)
                    .ok()
                    .and_then(|mesh| mesh.heightmap().height_at_position(new_location.0.x, new_location.0.y))
                    .unwrap_or(pos.location.0.y);
                agent.move_to(new_location.with_y(new_y));
            }
        } else {
            stroll.check_timer.reset();
        }
    }
}
