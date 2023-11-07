use crate::agent::states::Idle;
use crate::comp::inventory::PlayerInventory;
use crate::comp::net::Client;
use crate::comp::{drop, EntityReference, GameEntity};
use crate::event::{AttackDefinition, DamageReceiveEvent};
use crate::ext::ActionIdCounter;
use bevy_ecs::prelude::*;
use bevy_ecs::query::QueryEntityError;
use bevy_time::{Time, Timer, TimerMode};
use silkroad_data::skilldata::{RefSkillData, SkillParam};
use silkroad_data::DataEntry;
use silkroad_game_base::{GlobalLocation, ItemTypeData};
use silkroad_protocol::combat::{DoActionResponseCode, PerformActionError, PerformActionResponse};
use silkroad_protocol::inventory::{InventoryItemContentData, InventoryOperationError, InventoryOperationResult};
use std::time::Duration;
use tracing::error;

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
                Duration::from_millis(value.0.timings.preparation_time.into()),
                TimerMode::Once,
            ),
        }
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub(crate) struct Pickup(pub Entity, pub Option<Timer>);

pub(crate) fn pickup(
    mut query: Query<(Entity, &Client, &mut Pickup, &mut PlayerInventory)>,
    time: Res<Time>,
    target_query: Query<&drop::Drop>,
    mut cmd: Commands,
) {
    let delta = time.delta();
    for (entity, client, mut pickup, mut inventory) in query.iter_mut() {
        if let Some(cooldown) = pickup.1.as_mut() {
            if cooldown.tick(delta).just_finished() {
                client.send(PerformActionResponse::Stop(PerformActionError::Completed));
                cmd.entity(entity).remove::<Pickup>().insert(Idle);
            }
        } else {
            let drop = match target_query.get(pickup.0) {
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

            cmd.entity(pickup.0).despawn();
            pickup.1 = Some(Timer::from_seconds(1.0, TimerMode::Once));

            match &drop.item.type_data {
                ItemTypeData::Gold { amount } => {
                    inventory.gain_gold(u64::from(*amount));
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
                        client.send(InventoryOperationResult::Error(InventoryOperationError::InventoryFull));
                    }
                    client.send(PerformActionResponse::Stop(PerformActionError::Completed));
                },
            }
        }
    }
}

pub(crate) fn action(
    mut query: Query<(Entity, &GameEntity, &mut Action)>,
    target_query: Query<&GameEntity>,
    time: Res<Time>,
    attack_instance_counter: Res<ActionIdCounter>,
    mut cmd: Commands,
    mut damage_event: EventWriter<DamageReceiveEvent>,
) {
    let delta = time.delta();
    for (entity, game_entity, mut action) in query.iter_mut() {
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
