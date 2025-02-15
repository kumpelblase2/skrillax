use crate::agent::component::AgentGoalReachedEvent;
use crate::agent::state::{
    ActionParameter, AgentState, AgentStateQueue, Dead, Idle, MovementTarget, SkillParameter, SkillTarget, Transition,
    TransitionPriority,
};
use crate::comp::inventory::PlayerInventory;
use crate::comp::net::Client;
use crate::comp::pos::Position;
use crate::comp::GameEntity;
use crate::config::GameConfig;
use crate::ext::Navmesh;
use crate::game::attack::Attack;
use crate::world::WorldData;
use bevy::prelude::*;
use cgmath::num_traits::Pow;
use cgmath::{InnerSpace, MetricSpace};
use silkroad_data::skilldata::RefSkillData;
use silkroad_definitions::inventory::EquipmentSlot;
use silkroad_game_base::{AttackSkill, GlobalLocation, GlobalPosition, Heading, Vector3Ext};
use silkroad_protocol::combat::{DoActionResponseCode, PerformActionResponse};

pub struct AttackingGoal {
    pub(crate) target: Entity,
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
    pub(crate) distance_squared: f32,
}

#[derive(Default)]
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
        Self::Attacking(AttackingGoal { target, skill: None })
    }

    pub fn attacking_with(target: Entity, skill: &'static RefSkillData) -> Self {
        Self::Attacking(AttackingGoal {
            target,
            skill: Some(skill),
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
        })
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

#[derive(Component, Default)]
pub struct GoalTracker {
    goal: AgentGoal,
    notify_reached: bool,
}

impl GoalTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn switch_goal(&mut self, goal: AgentGoal) {
        self.goal = goal;
        self.notify_reached = false;
    }

    pub fn switch_goal_notified(&mut self, goal: AgentGoal) {
        self.goal = goal;
        self.notify_reached = true;
    }

    pub fn should_notify(&self) -> bool {
        self.notify_reached
    }

    pub fn reset(&mut self) {
        self.switch_goal(AgentGoal::None);
    }

    pub fn mark_notified(&mut self) {
        self.notify_reached = false;
    }

    pub fn has_goal(&self) -> bool {
        !self.goal.is_none()
    }
}

const FOLLOW_DISTANCE_SQUARED: f32 = 1000.0;

pub(crate) fn apply_goal(
    mut query: Query<
        (
            &GameEntity,
            &mut GoalTracker,
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
    for (game_entity, mut goal, mut state, position, idle, inventory) in query.iter_mut() {
        match &goal.goal {
            AgentGoal::Attacking(args) => {
                let Ok((target_pos, dead)) = target_query.get(args.target) else {
                    goal.reset();
                    continue;
                };

                if dead.is_some() {
                    goal.reset();
                    continue;
                }

                if idle.is_none() {
                    continue;
                }

                let skill = args.skill.unwrap_or(match inventory {
                    Some(inv) => Attack::find_attack_for_player(inv).unwrap(),
                    None => Attack::find_attack_for_monster(*game_entity).unwrap(),
                });
                let weapon = inventory.and_then(|inv| inv.get_equipment_item(EquipmentSlot::Weapon));
                let range = AttackSkill::get_range_for_attack(skill, weapon.map(|item| item.reference));
                let range_squared = range.pow(2);
                let range_to_target = position.distance_to(target_pos);

                if range_to_target <= range_squared {
                    let target_state = AgentState::PerformSkill(SkillParameter {
                        target: SkillTarget::Entity(args.target),
                        skill,
                    });
                    state.push(Transition::create(target_state, TransitionPriority::Default, true));
                } else {
                    let new_target_position = position
                        .location()
                        .point_in_line_with_range(target_pos.location(), range - 0.1);
                    let new_height = navmesh
                        .height_for(new_target_position)
                        .unwrap_or(target_pos.position().y);
                    let final_position = new_target_position.with_y(new_height);
                    state.push(Transition::new(AgentState::Moving(MovementTarget::Location(
                        final_position,
                    ))));
                }
            },
            AgentGoal::Moving(target) => {
                let mut next_state: Option<AgentState> = None;
                match target {
                    MovingGoal::Direction(dir) => {
                        next_state = Some(AgentState::Moving(MovementTarget::Direction(*dir)))
                    },
                    MovingGoal::Destination(target_pos) => {
                        if position.position().distance2(target_pos.0) < 1.0 {
                            goal.reset();
                        } else {
                            next_state = Some(AgentState::Moving(MovementTarget::Location(*target_pos)));
                        }
                    },
                }

                if let Some(next_state) = next_state {
                    state.push(Transition::create(next_state, TransitionPriority::Default, true));
                }
            },
            AgentGoal::PickingUp(args) => {
                let Ok((target_pos, _)) = target_query.get(args.target) else {
                    goal.reset();
                    continue;
                };

                let Some(character_data) = WorldData::characters().find_id(game_entity.ref_id) else {
                    goal.reset();
                    continue;
                };

                let Some(range) = character_data.pickup_range else {
                    goal.reset();
                    continue;
                };

                let range: f32 = range.get().into();

                if target_pos.distance_to(position) <= range.powf(2.0) {
                    if idle.is_some() {
                        let next_state = AgentState::PickingUp((*args).into());
                        state.push(Transition::create(next_state, TransitionPriority::Default, true));
                    }
                } else {
                    let my_location = position.location();
                    let target_movement_pos = my_location.point_in_line_with_range(target_pos.location(), range - 0.1);

                    let target_height = navmesh.height_for(target_movement_pos).unwrap_or(position.position().y);
                    state.push(Transition::new(AgentState::Moving(MovementTarget::Location(
                        target_movement_pos.with_y(target_height),
                    ))));
                }
            },
            AgentGoal::PerformingAction(args) => {
                if idle.is_none() {
                    let next_state = AgentState::PerformingAction(ActionParameter::new(args.action));
                    state.push(Transition::create(next_state, TransitionPriority::Default, true))
                }
            },
            AgentGoal::Following(args) => {
                let Ok((target_pos, _)) = target_query.get(args.target) else {
                    goal.reset();
                    continue;
                };

                let distance_to_target = position.distance_to(target_pos);
                if distance_to_target > settings.max_follow_distance.pow(2) {
                    goal.reset();
                } else if distance_to_target > FOLLOW_DISTANCE_SQUARED {
                    let dir_vector = position.position().to_flat_vec2() - target_pos.position().to_flat_vec2();
                    let target_vector = dir_vector.normalize() * FOLLOW_DISTANCE_SQUARED;
                    let target_location = GlobalLocation(position.position().to_flat_vec2() - target_vector);
                    let height = navmesh.height_for(target_location).unwrap_or(position.position().y);
                    state.push(Transition::new(AgentState::Moving(MovementTarget::Location(
                        target_location.with_y(height),
                    ))));
                }
            },
            _ => {
                if idle.is_none() {
                    state.push(Transition::new(AgentState::Idle));
                }
            },
        }
    }
}

pub(crate) fn handle_state_reached_notification(
    mut reader: EventReader<AgentGoalReachedEvent>,
    mut query: Query<(&Client, &mut GoalTracker)>,
) {
    for event in reader.read() {
        let Ok((client, mut goal)) = query.get_mut(event.entity) else {
            continue;
        };

        if goal.notify_reached {
            goal.notify_reached = false;
            match event.state {
                AgentState::PerformSkill(performing) => {
                    let mut used_skill = false;
                    if let AgentGoal::Attacking(attacking) = &goal.goal {
                        if let Some(skill) = attacking.skill {
                            if skill.ref_id == performing.skill.ref_id && performing.target.is_entity(attacking.target)
                            {
                                used_skill = true;
                            }
                        }
                    }

                    if used_skill {
                        if let Some(target) = performing.target.entity() {
                            goal.switch_goal(AgentGoal::attacking(target));
                        }
                    }

                    client.send(PerformActionResponse::Do(DoActionResponseCode::Success));
                },
                AgentState::Sitting => {},
                AgentState::PerformingAction(_) => {
                    client.send(PerformActionResponse::Do(DoActionResponseCode::Success));
                },
                AgentState::PickingUp(_) => {},
                _ => {},
            }
        }
    }
}
