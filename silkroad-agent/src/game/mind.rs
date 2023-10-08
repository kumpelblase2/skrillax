use crate::agent::states::{Dead, Idle, MovementGoal, Moving, Pickup, StateTransitionQueue};
use crate::comp::inventory::PlayerInventory;
use crate::comp::net::Client;
use crate::comp::pos::Position;
use crate::comp::{EntityReference, GameEntity};
use crate::ext::Navmesh;
use crate::game::attack::{Attack, AttackProcess};
use crate::world::WorldData;
use bevy_app::{App, Plugin, PostUpdate, PreUpdate};
use bevy_ecs::prelude::*;
use bevy_ecs_macros::Component;
use bevy_time::common_conditions::on_timer;
use silkroad_data::skilldata::RefSkillData;
use silkroad_definitions::inventory::EquipmentSlot;
use silkroad_game_base::AttackSkillError;
use silkroad_protocol::combat::{DoActionResponseCode, PerformActionError, PerformActionResponse};
use std::time::Duration;
use tracing::warn;

#[derive(Component, Default)]
pub struct Mind {
    current_goal: Option<Goal>,
}

impl Mind {
    pub fn attack(&mut self, target: EntityReference) {
        self.current_goal = Some(Goal::Attack(target))
    }

    pub fn attack_with(&mut self, target: EntityReference, skill: &'static RefSkillData) {
        self.current_goal = Some(Goal::ExecuteSkill(target, skill))
    }

    pub fn cancel(&mut self) {
        self.current_goal = None;
    }

    pub fn pickup(&mut self, target: EntityReference) {
        self.current_goal = Some(Goal::PickUp(target));
    }
}

#[derive(Copy, Clone)]
pub enum Goal {
    Attack(EntityReference),
    ExecuteSkill(EntityReference, &'static RefSkillData),
    PickUp(EntityReference),
}

fn enqueue_action(
    mut query: Query<
        (
            &GameEntity,
            Option<&Client>,
            &mut Mind,
            &Position,
            &mut StateTransitionQueue,
            Option<&PlayerInventory>,
        ),
        With<Idle>,
    >,
    target_query: Query<&Position, Without<Dead>>,
    navmesh: Res<Navmesh>,
) {
    for (entity, client, mut mind, position, mut state, inventory) in query.iter_mut() {
        if let Some(goal) = mind.current_goal.as_ref() {
            if matches!(goal, Goal::PickUp(_)) {
                let Goal::PickUp(target) = goal else {
                    // This should never happen.
                    continue;
                };

                let Ok(target_pos) = target_query.get(target.0) else {
                    mind.cancel();
                    state.request_transition(Idle);
                    continue;
                };
                let Some(character_data) = WorldData::characters().find_id(entity.ref_id) else {
                    mind.cancel();
                    state.request_transition(Idle);
                    continue;
                };

                let Some(range) = character_data.pickup_range else {
                    mind.cancel();
                    state.request_transition(Idle);
                    continue;
                };

                let range: f32 = range.get().into();

                if target_pos.distance_to(position) <= range.powf(2.0) {
                    state.request_transition(Pickup(target.0, None));
                } else {
                    let my_location = position.location.to_location();
                    let target_movement_pos =
                        my_location.point_in_line_with_range(target_pos.location.to_location(), range);

                    let target_height = navmesh.height_for(target_movement_pos).unwrap_or(position.location.y);
                    state.request_transition(Moving(MovementGoal::Location(
                        target_movement_pos.with_y(target_height),
                    )));
                }
            } else {
                let (target, skill) = match goal {
                    Goal::Attack(target) => (
                        target,
                        match inventory {
                            Some(inv) => Attack::find_attack_for_player(inv).unwrap(),
                            None => Attack::find_attack_for_monster(*entity).unwrap(),
                        },
                    ),
                    Goal::ExecuteSkill(target, skill) => (target, *skill),
                    _ => continue,
                };

                let target_pos = match target_query.get(target.0) {
                    Ok(target_position) => target_position,
                    Err(_) => {
                        // Target most likely died.
                        mind.cancel();
                        continue;
                    },
                };

                let mut process = AttackProcess::new(
                    &mut state,
                    position,
                    skill,
                    inventory.and_then(|inv| inv.get_equipment_item(EquipmentSlot::Weapon)),
                    target,
                    target_pos,
                    &navmesh,
                );

                match process.try_attack() {
                    Ok(_) => {
                        if let Some(client) = client {
                            client.send(PerformActionResponse::Do(DoActionResponseCode::Success));
                        }
                    },
                    Err(err) => {
                        mind.cancel();
                        if let Some(client) = client {
                            match err {
                                AttackSkillError::NotAWeapon | AttackSkillError::UnknownWeapon => {
                                    client.send(PerformActionResponse::Stop(PerformActionError::InvalidWeapon));
                                },
                                AttackSkillError::SkillNotFound => {
                                    client.send(PerformActionResponse::Stop(PerformActionError::NotLearned));
                                },
                            }
                        } else {
                            warn!("Couldn't execute attack for monster");
                        }
                    },
                }
            }
        }
    }
}

pub(crate) fn update_action(
    mut query: Query<
        (
            &GameEntity,
            Option<&Client>,
            &mut Mind,
            &Position,
            &mut StateTransitionQueue,
            Option<&PlayerInventory>,
            &Moving,
        ),
        Without<Idle>,
    >,
    target_query: Query<&Position, Without<Dead>>,
    navmesh: Res<Navmesh>,
) {
    for (entity, client, mut mind, position, mut state, inventory, moving) in query.iter_mut() {
        let Some(goal) = mind.current_goal.as_ref() else {
            continue;
        };

        if let Goal::PickUp(target) = goal {
            if target_query.get(target.0).is_err() {
                mind.cancel();
                state.request_transition(Idle);
                continue;
            }

            if !matches!(moving.0, MovementGoal::Location(_)) {
                mind.cancel();
                continue;
            }
        } else {
            let (target, skill) = match goal {
                Goal::Attack(target) => (
                    target,
                    match inventory {
                        Some(inv) => Attack::find_attack_for_player(inv).unwrap(),
                        None => Attack::find_attack_for_monster(*entity).unwrap(),
                    },
                ),
                Goal::ExecuteSkill(target, skill) => (target, *skill),
                _ => continue,
            };

            if !matches!(moving.0, MovementGoal::Location(_)) {
                // If we aren't moving to a specific location, we aren't moving to a target.
                // Thus, we probably cancelled the attack.
                mind.cancel();
                continue;
            }

            let Ok(entity_location) = target_query.get(target.0) else {
                // Target probably died. We might need to send some "invalid target" response here?
                mind.cancel();
                state.request_transition(Idle);
                continue;
            };

            let mut process = AttackProcess::new(
                &mut state,
                position,
                skill,
                inventory.and_then(|inv| inv.get_equipment_item(EquipmentSlot::Weapon)),
                target,
                entity_location,
                &navmesh,
            );

            match process.try_attack() {
                Ok(_) => {
                    if let Some(client) = client {
                        client.send(PerformActionResponse::Do(DoActionResponseCode::Success));
                    }
                },
                Err(err) => {
                    mind.cancel();
                    if let Some(client) = client {
                        match err {
                            AttackSkillError::NotAWeapon | AttackSkillError::UnknownWeapon => {
                                client.send(PerformActionResponse::Stop(PerformActionError::InvalidWeapon));
                            },
                            AttackSkillError::SkillNotFound => {
                                client.send(PerformActionResponse::Stop(PerformActionError::NotLearned));
                            },
                        }
                    } else {
                        warn!("Couldn't execute attack for monster");
                    }
                },
            }
        }
    }
}

pub(crate) struct MindPlugin;

impl Plugin for MindPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, enqueue_action)
            .add_systems(PreUpdate, update_action.run_if(on_timer(Duration::from_millis(100))));
    }
}
