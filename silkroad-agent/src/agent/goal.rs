use crate::agent::state::Dead;
use crate::agent::state::{
    ActionParameter, AgentState, AgentStateQueue, Idle, MovementTarget, SkillParameter, SkillTarget,
};
use crate::comp::inventory::PlayerInventory;
use crate::comp::pos::Position;
use crate::comp::GameEntity;
use crate::config::GameConfig;
use crate::ext::Navmesh;
use crate::game::attack::Attack;
use crate::world::WorldData;
use bevy_ecs::prelude::*;
use bevy_time::prelude::*;
use cgmath::num_traits::Pow;
use cgmath::{InnerSpace, MetricSpace};
use silkroad_data::skilldata::RefSkillData;
use silkroad_definitions::inventory::EquipmentSlot;
use silkroad_game_base::{AttackSkill, GlobalLocation, GlobalPosition, Heading, Vector3Ext};
use std::time::Duration;

pub struct AttackingGoal {
    pub(crate) target: Entity,
    pub(crate) move_recheck: Timer,
    pub(crate) skill: Option<&'static RefSkillData>,
}

#[derive(Clone, Copy)]
pub enum MovingGoal {
    Direction(Heading),
    Destination(GlobalPosition),
}

#[derive(Clone, Copy)]
pub struct PickingUpGoal {
    pub(crate) target: Entity,
}

#[derive(Clone, Copy)]
pub struct ActionGoal {
    pub(crate) action: u32,
}

#[derive(Clone)]
pub struct FollowingGoal {
    pub(crate) target: Entity,
    pub(crate) move_recheck: Timer,
    pub(crate) distance_squared: f32,
}

#[derive(Component, Default)]
pub enum AgentGoal {
    #[default]
    None,
    Attacking(AttackingGoal),
    Moving(MovingGoal),
    PickingUp(PickingUpGoal),
    PerformingAction(ActionGoal),
    Following(FollowingGoal),
}

impl AgentGoal {
    pub fn none() -> Self {
        Self::None
    }

    pub fn attacking(target: Entity) -> Self {
        Self::Attacking(AttackingGoal {
            target,
            skill: None,
            move_recheck: Timer::new(Duration::from_millis(500), TimerMode::Repeating),
        })
    }

    pub fn attacking_with(target: Entity, skill: &'static RefSkillData) -> Self {
        Self::Attacking(AttackingGoal {
            target,
            skill: Some(skill),
            move_recheck: Timer::new(Duration::from_millis(500), TimerMode::Repeating),
        })
    }

    pub fn moving_to(destination: GlobalPosition) -> Self {
        Self::Moving(MovingGoal::Destination(destination))
    }

    pub fn moving_in_direction(heading: Heading) -> Self {
        Self::Moving(MovingGoal::Direction(heading))
    }

    pub fn picking_up(target: Entity) -> Self {
        Self::PickingUp(PickingUpGoal { target })
    }

    pub fn perform_action(action: u32) -> Self {
        Self::PerformingAction(ActionGoal { action })
    }

    pub fn follow(target: Entity, distance: f32) -> Self {
        Self::Following(FollowingGoal {
            target,
            distance_squared: distance * distance,
            move_recheck: Timer::new(Duration::from_millis(500), TimerMode::Repeating),
        })
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

pub(crate) fn apply_goal(
    mut cmd: Commands,
    mut query: Query<
        (
            Entity,
            &GameEntity,
            &AgentGoal,
            &mut AgentStateQueue,
            &Position,
            Option<&Idle>,
            Option<&PlayerInventory>,
        ),
        Without<Dead>,
    >,
    target_query: Query<(&Position, Option<&Dead>)>,
    settings: Res<GameConfig>,
    navmesh: Res<Navmesh>,
) {
    for (us, entity, goal, mut state, position, idle, inventory) in query.iter_mut() {
        match goal {
            AgentGoal::Attacking(args) => {
                let Ok((target_pos, dead)) = target_query.get(args.target) else {
                    cmd.entity(us).insert(AgentGoal::None);
                    continue;
                };

                if dead.is_some() {
                    cmd.entity(us).insert(AgentGoal::None);
                    continue;
                }

                let skill = args.skill.unwrap_or(match inventory {
                    Some(inv) => Attack::find_attack_for_player(inv).unwrap(),
                    None => Attack::find_attack_for_monster(*entity).unwrap(),
                });
                let weapon = inventory.and_then(|inv| inv.get_equipment_item(EquipmentSlot::Weapon));
                let range = AttackSkill::get_range_for_attack(skill, weapon.map(|item| item.reference));
                let range_squared = range.pow(2);
                if position.distance_to(target_pos) <= range_squared {
                    state.push(AgentState::PerformSkill(SkillParameter {
                        target: SkillTarget::Entity(args.target),
                        skill,
                    }))
                } else {
                    let new_target_position = position
                        .position()
                        .to_location()
                        .point_in_line_with_range(target_pos.location(), range);
                    let new_height = navmesh
                        .height_for(new_target_position)
                        .unwrap_or(target_pos.position().y);
                    state.push(AgentState::Moving(MovementTarget::Location(
                        new_target_position.with_y(new_height),
                    )));
                }
            },
            AgentGoal::Moving(target) => match target {
                MovingGoal::Direction(dir) => state.push(AgentState::Moving(MovementTarget::Direction(*dir))),
                MovingGoal::Destination(target_pos) => {
                    if position.position().distance2(target_pos.0) < 1.0 {
                        cmd.entity(us).insert(AgentGoal::None);
                    } else {
                        state.push(AgentState::Moving(MovementTarget::Location(*target_pos)))
                    }
                },
            },
            AgentGoal::PickingUp(args) => {
                let Ok((target_pos, _)) = target_query.get(args.target) else {
                    cmd.entity(us).insert(AgentGoal::None);
                    continue;
                };

                let Some(character_data) = WorldData::characters().find_id(entity.ref_id) else {
                    cmd.entity(us).insert(AgentGoal::None);
                    continue;
                };

                let Some(range) = character_data.pickup_range else {
                    cmd.entity(us).insert(AgentGoal::None);
                    continue;
                };

                let range: f32 = range.get().into();

                if target_pos.distance_to(position) <= range.powf(2.0) {
                    state.push(AgentState::PickingUp((*args).into()));
                } else {
                    let my_location = position.location();
                    let target_movement_pos = my_location.point_in_line_with_range(target_pos.location(), range);

                    let target_height = navmesh.height_for(target_movement_pos).unwrap_or(position.position().y);
                    state.push(AgentState::Moving(MovementTarget::Location(
                        target_movement_pos.with_y(target_height),
                    )));
                }
            },
            AgentGoal::PerformingAction(args) => {
                state.push(AgentState::PerformingAction(ActionParameter::new(args.action)))
            },
            AgentGoal::Following(args) => {
                let Ok((target_pos, _)) = target_query.get(args.target) else {
                    cmd.entity(us).insert(AgentGoal::None);
                    continue;
                };

                if position.distance_to(target_pos) > settings.max_follow_distance.pow(2) {
                    cmd.entity(us).insert(AgentGoal::None);
                } else {
                    let dir_vector = position.position().to_flat_vec2() - target_pos.position().to_flat_vec2();
                    let target_vector = dir_vector.normalize() * 5.0; // TODO where to put this?
                    let target_location = GlobalLocation(position.position().to_flat_vec2() - target_vector);
                    let height = navmesh.height_for(target_location).unwrap_or(position.position().y);
                    state.push(AgentState::Moving(MovementTarget::Location(
                        target_location.with_y(height),
                    )));
                }
            },
            _ => {
                if idle.is_none() {
                    state.push(AgentState::Idle);
                }
            },
        }
    }
}
