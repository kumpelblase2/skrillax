use crate::agent::component::{Agent, MovementState};
use crate::agent::goal::{AgentGoal, GoalTracker};
use crate::agent::state::{
    Idle, MovementTarget as AgentMovementTarget, Moving, PerformingSkill, PickingUp, SkillParameter,
    SkillProgressState, SkillTarget,
};
use crate::comp::gold::GoldPouch;
use crate::comp::inventory::PlayerInventory;
use crate::comp::net::Client;
use crate::comp::pos::Position;
use crate::comp::{drop, EntityReference, GameEntity, Health, Mana};
use crate::event::{ConsumeItemEvent, DamageReceiveEvent, SkillDefinition};
use crate::ext::{ActionIdCounter, Navmesh};
use crate::input::PlayerInput;
use crate::world::WorldData;
use bevy_ecs::prelude::*;
use bevy_ecs::query::QueryEntityError;
use bevy_time::{Time, Timer, TimerMode};
use cgmath::{Array, Deg, InnerSpace, Quaternion, Rotation3, Vector2, Vector3, Zero};
use silkroad_data::skilldata::SkillParam;
use silkroad_data::DataEntry;
use silkroad_definitions::type_id::{ObjectItem, ObjectType};
use silkroad_game_base::{GlobalLocation, Heading, ItemTypeData, LocalLocation, Vector3Ext};
use silkroad_protocol::combat::{DoActionResponseCode, PerformActionError, PerformActionResponse};
use silkroad_protocol::inventory::{InventoryItemContentData, InventoryOperationError, InventoryOperationResult};
use silkroad_protocol::movement::MovementTarget;
use std::ops::Deref;
use std::time::Duration;
use tracing::{debug, error, warn};

const EPSYLON: f32 = 1.0;

pub(crate) fn movement_input(
    mut query: Query<(&Client, &PlayerInput, &Position, &mut GoalTracker)>,
    navmesh: Res<Navmesh>,
) {
    for (client, input, position, mut goal) in query.iter_mut() {
        if let Some(kind) = input.movement {
            match kind {
                MovementTarget::TargetLocation { region, x, y: _, z } => {
                    let local_position = position.position().to_local();
                    let target_loc = LocalLocation(region.into(), Vector2::new(x.into(), z.into()));
                    let target_height = navmesh.height_for(target_loc).unwrap_or(position.position().y);
                    let target_pos = target_loc.with_y(target_height);
                    debug!(identifier = ?client.id(), "Movement: {} -> {}", local_position, target_pos);
                    goal.switch_goal_notified(AgentGoal::moving_to(target_pos.to_global()));
                },
                MovementTarget::Direction { unknown, angle } => {
                    let direction = Heading::from(angle);
                    debug!(identifier = ?client.id(), "Movement: {} / {}({})", unknown, direction.0, angle);
                    goal.switch_goal_notified(AgentGoal::moving_in_direction(direction));
                },
            }
        }
    }
}

pub(crate) fn pickup(
    mut query: Query<(Entity, &Client, &mut PickingUp, &mut PlayerInventory, &mut GoldPouch)>,
    time: Res<Time>,
    target_query: Query<&drop::Drop>,
    mut cmd: Commands,
) {
    let delta = time.delta();
    for (entity, client, mut pickup, mut inventory, mut gold) in query.iter_mut() {
        if let Some(cooldown) = pickup.cooldown.as_mut() {
            if cooldown.tick(delta).just_finished() {
                client.send(PerformActionResponse::Stop(PerformActionError::Completed));
                cmd.entity(entity).remove::<PickingUp>().try_insert(Idle);
            }
        } else {
            let drop = match target_query.get(pickup.parameter.target) {
                Ok(drop) => drop,
                Err(QueryEntityError::NoSuchEntity(_)) => {
                    client.send(PerformActionResponse::Stop(PerformActionError::InvalidTarget));
                    cmd.entity(entity).remove::<PickingUp>();
                    continue;
                },
                Err(e) => {
                    error!("Could not load target pickup item: {:?}", e);
                    cmd.entity(entity).remove::<PickingUp>();
                    continue;
                },
            };

            cmd.entity(pickup.parameter.target).despawn();
            pickup.cooldown = Some(Timer::from_seconds(1.0, TimerMode::Once));

            match &drop.item.type_data {
                ItemTypeData::Gold { amount } => {
                    gold.gain(u64::from(*amount));
                    client.send(PerformActionResponse::Do(DoActionResponseCode::Success));
                },
                _ => {
                    if let Some(slot) = inventory.add_item(drop.item) {
                        client.send(InventoryOperationResult::success_gain_item(
                            slot,
                            drop.item.reference.ref_id(),
                            InventoryItemContentData::Expendable {
                                stack_size: drop.item.stack_size(),
                            },
                        ));
                    } else {
                        client.send(InventoryOperationResult::Failure(
                            InventoryOperationError::InventoryFull,
                        ));
                    }
                    client.send(PerformActionResponse::Stop(PerformActionError::Completed));
                },
            }
        }
    }
}

pub(crate) fn action(
    mut query: Query<(
        Entity,
        &GameEntity,
        &mut PerformingSkill,
        Option<&mut Mana>,
        &mut Health,
        Option<&PlayerInventory>,
    )>,
    target_query: Query<&GameEntity>,
    time: Res<Time>,
    attack_instance_counter: Res<ActionIdCounter>,
    mut cmd: Commands,
) {
    let delta = time.delta();
    for (entity, game_entity, mut action, mana, mut health, inventory) in query.iter_mut() {
        if action.timer.tick(delta).just_finished() {
            let Some(next) = action.progress.next() else {
                if let Some(next_skill) = action.parameter.skill.next_in_chain {
                    *action = PerformingSkill::new(SkillParameter {
                        target: action.parameter.target,
                        skill: WorldData::skills()
                            .find_id(next_skill.into())
                            .expect("Next skill in chain should exist."),
                    });
                } else {
                    cmd.entity(entity).remove::<PerformingSkill>();
                }

                continue;
            };

            if next == SkillProgressState::Casting {
                if action.parameter.skill.consumed_mp > 0 {
                    let Some(mut mana) = mana else {
                        cmd.entity(entity).remove::<PerformingSkill>();
                        warn!("Tried to execute action consuming mana without having mana.");
                        continue;
                    };

                    if mana.current_mana < action.parameter.skill.consumed_mp {
                        cmd.entity(entity).remove::<PerformingSkill>();
                        debug!("Cancelling attack due to insufficient mana.");
                        // TODO: send info to client.
                        continue;
                    }
                    mana.spend(action.parameter.skill.consumed_mp);
                }

                if action.parameter.skill.consumed_hp > 0 {
                    if health.current_health < action.parameter.skill.consumed_hp {
                        cmd.entity(entity).remove::<PerformingSkill>();
                        debug!("Cancelling attack due to insufficient health.");
                        // TODO: send info to client.
                        continue;
                    }

                    health.reduce(action.parameter.skill.consumed_hp);
                }

                for param in &action.parameter.skill.params {
                    match param {
                        SkillParam::RequiredItem(item) => {
                            let type_id = ObjectType::Item(ObjectItem::Equippable(*item)).type_id();
                            let Some(inventory) = inventory else {
                                cmd.entity(entity).remove::<PerformingSkill>();
                                warn!("Tried to use skill with item, but doesn't have inventory.");
                                continue;
                            };
                            if !inventory
                                .equipment_items()
                                .any(|(_, equip_item)| type_id == equip_item.reference.common.type_id)
                            {
                                warn!("Missing requirement equipment");
                            }
                        },
                        SkillParam::ConsumeItem { kind, amount } => {
                            // TODO: handle non-ammo type
                            let amount = *amount as u16;
                            let type_id = ObjectType::Item(ObjectItem::Consumable(*kind)).type_id();
                            let Some(inventory) = inventory else {
                                cmd.entity(entity).remove::<PerformingSkill>();
                                warn!("Tried to use skill with item, but doesn't have inventory.");
                                continue;
                            };
                            if !inventory.equipment_items().any(|(_, equip_item)| {
                                equip_item.reference.common.type_id == type_id && equip_item.stack_size() >= amount
                            }) {
                                warn!("Can't consume item, because it's not equipped");
                            } else {
                                cmd.send_event(ConsumeItemEvent {
                                    player: entity,
                                    item: type_id,
                                    amount,
                                });
                            }
                        },
                        _ => {},
                    };
                }
            }

            let time = next.get_time_for(action.parameter.skill).unwrap_or(0);
            action.progress = next;
            action.timer = Timer::new(Duration::from_millis(time as u64), TimerMode::Once);

            if next == SkillProgressState::Execution {
                let attack = action
                    .parameter
                    .skill
                    .params
                    .iter()
                    .find(|param| matches!(param, SkillParam::Attack { .. }))
                    .unwrap();
                match attack {
                    SkillParam::Attack { .. } => {
                        let SkillTarget::Entity(target) = action.parameter.target else {
                            panic!();
                        };
                        let target_ = target_query.get(target).unwrap();
                        cmd.send_event(DamageReceiveEvent {
                            source: EntityReference(entity, *game_entity),
                            target: EntityReference(target, *target_),
                            attack: SkillDefinition {
                                skill: action.parameter.skill,
                                instance: attack_instance_counter.next(),
                            },
                            amount: 10,
                        });
                    },
                    _ => {},
                }
            }
        }
    }
}

pub(crate) fn movement(
    mut query: Query<(Entity, &mut Position, &Agent, &Moving, &MovementState)>,
    time: Res<Time>,
    mut cmd: Commands,
    navmesh: Res<Navmesh>,
) {
    let delta = time.delta_secs();
    for (entity, mut pos, agent, movement, speed_state) in query.iter_mut() {
        let speed = agent.get_speed_value(*speed_state.deref());
        let (next_location, heading, finished) = match movement.parameter {
            AgentMovementTarget::Location(location) => {
                get_next_step(delta, pos.location(), speed, location.to_location())
            },
            AgentMovementTarget::Direction(direction) => {
                let current_location_2d = pos.location().0;
                let direction_vec = Quaternion::from_angle_y(Deg(direction.0)) * Vector3::unit_x();
                let direction_vec = direction_vec.to_flat_vec2().normalize();
                let movement = direction_vec * (speed * delta);
                (GlobalLocation(current_location_2d + movement), direction, false)
            },
        };

        move_with_step(&navmesh, &mut pos, next_location, heading);

        if finished {
            cmd.entity(entity).remove::<Moving>().try_insert(Idle);
        }
    }
}

fn get_next_step(
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

fn move_with_step(navmesh: &Navmesh, pos: &mut Position, target: GlobalLocation, heading: Heading) {
    let target_location = target.to_local();
    let height = navmesh.height_for(target_location).unwrap_or(pos.position().0.y);

    let position = target.with_y(height);
    pos.update(position, heading);
}

pub(crate) fn turning(mut query: Query<(&mut Position, &PlayerInput), With<Idle>>) {
    for (mut pos, input) in query.iter_mut() {
        if let Some(ref rotate) = input.rotation {
            pos.rotate(Heading::from(rotate.heading));
        }
    }
}
