use crate::agent::event::MovementFinished;
use crate::agent::states::{get_next_step, move_with_step, Idle, StateTransitionQueue};
use crate::agent::{Agent, MovementState};
use crate::comp::inventory::PlayerInventory;
use crate::comp::net::Client;
use crate::comp::pos::Position;
use crate::comp::sync::{ActionAnimation, Synchronize};
use crate::comp::{drop, EntityReference, GameEntity};
use crate::config::GameConfig;
use crate::event::{AttackDefinition, DamageReceiveEvent};
use crate::ext::Navmesh;
use crate::game::attack::AttackInstanceCounter;
use bevy_ecs::prelude::*;
use bevy_ecs::query::QueryEntityError;
use bevy_time::{Time, Timer, TimerMode};
use cgmath::num_traits::Pow;
use cgmath::InnerSpace;
use derive_more::Deref;
use silkroad_data::skilldata::{RefSkillData, SkillParam};
use silkroad_data::DataEntry;
use silkroad_game_base::{get_range_for_attack, GlobalLocation, GlobalPosition, ItemTypeData, Vector3Ext};
use silkroad_protocol::combat::{PerformActionError, PerformActionResponse};
use silkroad_protocol::inventory::{InventoryItemContentData, InventoryOperationError, InventoryOperationResult};
use silkroad_protocol::world::UnknownActionData;
use std::ops::Deref;
use std::time::Duration;
use tracing::{debug, error};

#[derive(Copy, Clone)]
pub(crate) enum ActionTarget {
    None,
    Own,
    Entity(Entity),
    Location(GlobalLocation),
}

#[derive(Component, Clone)]
#[component(storage = "SparseSet")]
pub(crate) struct Action {
    skill: &'static RefSkillData,
    target: ActionTarget,
    state: ActionProgressState,
    progress: Timer,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
enum ActionProgressState {
    Preparation,
    Casting,
    Execution,
    Teardown,
}

impl ActionProgressState {
    pub fn next(&self) -> Option<ActionProgressState> {
        match self {
            ActionProgressState::Preparation => Some(ActionProgressState::Casting),
            ActionProgressState::Casting => Some(ActionProgressState::Execution),
            ActionProgressState::Execution => Some(ActionProgressState::Teardown),
            ActionProgressState::Teardown => None,
        }
    }

    pub fn get_time_for(&self, skill: &RefSkillData) -> Option<i32> {
        let value = match self {
            ActionProgressState::Preparation => skill.timings.preparation_time as i32,
            ActionProgressState::Casting => skill.timings.cast_time as i32,
            ActionProgressState::Execution => skill.timings.duration,
            ActionProgressState::Teardown => skill.timings.next_delay as i32,
        };

        if value > 0 {
            Some(value)
        } else {
            None
        }
    }
}

#[derive(Copy, Clone)]
pub(crate) struct ActionDescription(pub &'static RefSkillData, pub ActionTarget);

impl From<ActionDescription> for Action {
    fn from(value: ActionDescription) -> Self {
        Action {
            skill: value.0,
            target: value.1,
            state: ActionProgressState::Preparation,
            progress: Timer::new(
                Duration::from_millis(value.0.timings.preparation_time as u64),
                TimerMode::Once,
            ),
        }
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub(crate) struct MoveToAction(pub Entity, pub GlobalPosition, pub ActionDescription);

#[derive(Component)]
#[component(storage = "SparseSet")]
pub(crate) struct MoveToPickup(pub Entity, pub GlobalPosition);

#[derive(Component, Deref)]
#[component(storage = "SparseSet")]
pub(crate) struct Pickup(pub Entity);

pub(crate) fn pickup(
    mut query: Query<(
        Entity,
        &GameEntity,
        &Position,
        &Client,
        &Pickup,
        &mut PlayerInventory,
        &mut Synchronize,
    )>,
    target_query: Query<&drop::Drop>,
    mut cmd: Commands,
) {
    for (entity, game_entity, pos, client, pickup, mut inventory, mut sync) in query.iter_mut() {
        let drop = match target_query.get(*pickup.deref()) {
            Ok(drop) => drop,
            Err(QueryEntityError::NoSuchEntity(_)) => {
                client.send(PerformActionResponse::Stop(PerformActionError::InvalidTarget));
                cmd.entity(entity).remove::<Pickup>();
                continue;
            },
            Err(e) => {
                error!("Could not load target pickup item: {:?}", e);
                cmd.entity(entity).remove::<Pickup>();
                continue;
            },
        };

        sync.actions.push(ActionAnimation::Pickup);
        cmd.entity(*pickup.deref()).despawn();

        match &drop.item.type_data {
            ItemTypeData::Gold { amount } => {
                inventory.gold += (*amount) as u64;
                client.send(UnknownActionData {
                    entity: game_entity.ref_id,
                    unknown: 0,
                });
                client.send(InventoryOperationResult::success_gain_gold(*amount));
                client.send(PerformActionResponse::Stop(PerformActionError::Completed));
            },
            _ => {
                if let Some(slot) = inventory.add_item(drop.item) {
                    client.send(UnknownActionData {
                        entity: game_entity.ref_id,
                        unknown: 0xb2,
                    });

                    client.send(InventoryOperationResult::success_gain_item(
                        slot,
                        drop.item.reference.ref_id(),
                        InventoryItemContentData::Expendable {
                            stack_size: drop.item.stack_size(),
                        },
                    ));
                } else {
                    client.send(InventoryOperationResult::Error(InventoryOperationError::InventoryFull));
                }
                client.send(PerformActionResponse::Stop(PerformActionError::Completed));
            },
        }
        cmd.entity(entity).remove::<Pickup>().insert(Idle);
    }
}

pub(crate) fn move_to_pickup(
    mut query: Query<(Entity, &MoveToPickup, &MovementState, &Agent, &mut Position)>,
    mut navmesh: ResMut<Navmesh>,
    time: Res<Time>,
    mut cmd: Commands,
    mut movement_finished: EventWriter<MovementFinished>,
) {
    let delta = time.delta_seconds_f64() as f32;
    for (entity, action, speed_state, agent, mut pos) in query.iter_mut() {
        let speed = agent.get_speed_value(*speed_state.deref());
        let (target, heading, finished) =
            get_next_step(delta, pos.location.to_location(), speed, action.1.to_location());
        move_with_step(&mut navmesh, &mut pos, target, heading);

        if finished {
            cmd.entity(entity).remove::<MoveToPickup>().insert(Pickup(action.0));
            movement_finished.send(MovementFinished(entity));
        }
    }
}

pub(crate) fn update_action_destination(
    mut query: Query<(Entity, &mut MoveToAction, &Position, &PlayerInventory)>,
    target_query: Query<&Position>,
    settings: Res<GameConfig>,
    mut navmesh: ResMut<Navmesh>,
    mut cmd: Commands,
    mut stopped: EventWriter<MovementFinished>,
) {
    for (itself, mut moving, own_position, inventory) in query.iter_mut() {
        match target_query.get(moving.0) {
            Ok(pos) => {
                if own_position.distance_to(pos) > settings.max_follow_distance.pow(2) {
                    cmd.entity(itself).remove::<MoveToAction>().insert(Idle);
                    stopped.send(MovementFinished(itself));
                } else {
                    let vector_pointing_to_player =
                        (own_position.location.to_flat_vec2() - pos.location.to_flat_vec2()).normalize();
                    let skill_range = get_range_for_attack(moving.2 .0, inventory.weapon().map(|item| item.reference));
                    let position_offset = vector_pointing_to_player * skill_range.into();
                    let target_location = GlobalLocation(pos.location.to_flat_vec2() + position_offset);
                    let height = navmesh.height_for(target_location).unwrap_or(moving.1.y);
                    moving.1 = target_location.with_y(height);
                }
            },
            Err(QueryEntityError::NoSuchEntity(_)) => {
                cmd.entity(itself).remove::<MoveToAction>().insert(Idle);
                stopped.send(MovementFinished(itself));
            },
            Err(e) => {
                debug!("Could not update entity position: {:?}", e);
            },
        }
    }
}

pub(crate) fn move_to_action(
    mut query: Query<(
        Entity,
        &MoveToAction,
        &MovementState,
        &Agent,
        &mut Position,
        &mut StateTransitionQueue,
    )>,
    mut navmesh: ResMut<Navmesh>,
    time: Res<Time>,
    mut cmd: Commands,
    mut movement_finished: EventWriter<MovementFinished>,
) {
    let delta = time.delta_seconds_f64() as f32;
    for (entity, action, speed_state, agent, mut pos, mut transition) in query.iter_mut() {
        let speed = agent.get_speed_value(*speed_state.deref());
        let (target, heading, finished) =
            get_next_step(delta, pos.location.to_location(), speed, action.1.to_location());
        move_with_step(&mut navmesh, &mut pos, target, heading);

        if finished {
            cmd.entity(entity).remove::<MoveToAction>();
            transition.request_transition(Action::from(action.2));
            movement_finished.send(MovementFinished(entity));
        }
    }
}

pub(crate) fn action(
    mut query: Query<(Entity, &GameEntity, &Client, &mut Action)>,
    target_query: Query<&GameEntity>,
    time: Res<Time>,
    mut attack_instance_counter: ResMut<AttackInstanceCounter>,
    mut cmd: Commands,
    mut damage_event: EventWriter<DamageReceiveEvent>,
) {
    let delta = time.delta();
    for (entity, game_entity, client, mut action) in query.iter_mut() {
        if action.progress.tick(delta).just_finished() {
            if let Some(next) = action.state.next() {
                let time = next.get_time_for(action.skill).unwrap_or(0);
                action.state = next;
                action.progress = Timer::new(Duration::from_millis(time as u64), TimerMode::Once);

                if next == ActionProgressState::Execution {
                    let attack = action
                        .skill
                        .params
                        .iter()
                        .find(|param| matches!(param, SkillParam::Attack { .. }))
                        .unwrap();
                    match attack {
                        SkillParam::Attack { .. } => {
                            let ActionTarget::Entity(target) = action.target else {
                                panic!();
                            };
                            let target_ = target_query.get(target).unwrap();
                            damage_event.send(DamageReceiveEvent {
                                source: EntityReference(entity, *game_entity),
                                target: EntityReference(target, *target_),
                                attack: AttackDefinition {
                                    skill: action.skill,
                                    instance: attack_instance_counter.next(),
                                },
                                amount: 10,
                            });
                        },
                        _ => {},
                    }
                }
            } else {
                cmd.entity(entity).remove::<Action>();
            }
        }
    }
}
