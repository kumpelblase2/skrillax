use crate::agent::goal::{MovingGoal, PickingUpGoal};
use bevy_ecs::prelude::*;
use bevy_time::prelude::*;
use cgmath::MetricSpace;
use silkroad_data::skilldata::RefSkillData;
use silkroad_game_base::{GlobalLocation, GlobalPosition, Heading};
use std::mem;

#[derive(Copy, Clone, PartialEq)]
pub enum SkillTarget {
    Entity(Entity),
    None,
    Own,
    Location(GlobalLocation),
}

#[derive(Copy, Clone)]
pub struct SkillParameter {
    pub(crate) target: SkillTarget,
    pub(crate) skill: &'static RefSkillData,
}

impl PartialEq for SkillParameter {
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target && self.skill.ref_id == other.skill.ref_id
    }
}

impl SkillParameter {
    pub fn new(target: SkillTarget, skill: &'static RefSkillData) -> Self {
        Self { target, skill }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum MovementTarget {
    Location(GlobalPosition),
    Direction(Heading),
}

impl MovementTarget {
    fn is_similar(&self, other: &MovementTarget) -> bool {
        match (self, other) {
            (MovementTarget::Location(loc), MovementTarget::Location(loc2)) => loc.distance2(loc2.0) < 2.0,
            (MovementTarget::Direction(loc), MovementTarget::Direction(loc2)) => loc.difference(loc2) <= 1.0,
            _ => false,
        }
    }
}

impl From<MovingGoal> for MovementTarget {
    fn from(value: MovingGoal) -> Self {
        match value {
            MovingGoal::Direction(dir) => MovementTarget::Direction(dir),
            MovingGoal::Destination(loc) => MovementTarget::Location(loc),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct ActionParameter {
    pub(crate) action: u32,
}

impl ActionParameter {
    pub fn new(action: u32) -> Self {
        Self { action }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PickupParameter {
    pub(crate) target: Entity,
}

impl From<PickingUpGoal> for PickupParameter {
    fn from(value: PickingUpGoal) -> Self {
        PickupParameter { target: value.target }
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct PickingUp {
    pub(crate) parameter: PickupParameter,
    pub(crate) cooldown: Option<Timer>,
}

impl PickingUp {
    pub fn new(parameter: PickupParameter) -> Self {
        Self {
            parameter,
            cooldown: None,
        }
    }
}

impl AsState for PickingUp {
    fn as_state(&self) -> AgentState {
        AgentState::PickingUp(self.parameter)
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum AgentState {
    Idle,
    Moving(MovementTarget),
    PerformSkill(SkillParameter),
    Sitting,
    PerformingAction(ActionParameter),
    PickingUp(PickupParameter),
    Dead,
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Idle;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub enum SkillProgressState {
    #[default]
    Preparation,
    Casting,
    Execution,
    Teardown,
}

impl SkillProgressState {
    pub fn next(&self) -> Option<SkillProgressState> {
        match self {
            SkillProgressState::Preparation => Some(SkillProgressState::Casting),
            SkillProgressState::Casting => Some(SkillProgressState::Execution),
            SkillProgressState::Execution => Some(SkillProgressState::Teardown),
            SkillProgressState::Teardown => None,
        }
    }

    pub fn get_time_for(&self, skill: &RefSkillData) -> Option<i32> {
        let value = match self {
            SkillProgressState::Preparation => skill.timings.preparation_time as i32,
            SkillProgressState::Casting => skill.timings.cast_time as i32,
            SkillProgressState::Execution => skill.timings.duration,
            SkillProgressState::Teardown => skill.timings.next_delay as i32,
        };

        if value > 0 {
            Some(value)
        } else {
            None
        }
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct PerformingSkill {
    pub(crate) parameter: SkillParameter,
    pub(crate) progress: SkillProgressState,
    pub(crate) timer: Timer,
}

impl PerformingSkill {
    pub fn new(parameter: SkillParameter) -> Self {
        Self {
            parameter,
            progress: SkillProgressState::default(),
            timer: Timer::default(),
        }
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Moving {
    pub(crate) parameter: MovementTarget,
}

impl Moving {
    pub fn new(parameter: MovementTarget) -> Self {
        Self { parameter }
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Sitting;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct PerformingAction {
    pub(crate) parameter: ActionParameter,
}

impl PerformingAction {
    pub fn new(parameter: ActionParameter) -> Self {
        Self { parameter }
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Dead;

#[derive(Event)]
pub struct StateTransitionEvent {
    pub entity: Entity,
    pub from: AgentState,
    pub to: AgentState,
}

trait AsState {
    fn as_state(&self) -> AgentState;
}

impl AsState for Idle {
    fn as_state(&self) -> AgentState {
        AgentState::Idle
    }
}

impl AsState for PerformingSkill {
    fn as_state(&self) -> AgentState {
        AgentState::PerformSkill(self.parameter)
    }
}

impl AsState for Moving {
    fn as_state(&self) -> AgentState {
        AgentState::Moving(self.parameter)
    }
}

impl AsState for Sitting {
    fn as_state(&self) -> AgentState {
        AgentState::Sitting
    }
}

impl AsState for PerformingAction {
    fn as_state(&self) -> AgentState {
        AgentState::PerformingAction(self.parameter)
    }
}

impl AsState for Dead {
    fn as_state(&self) -> AgentState {
        AgentState::Dead
    }
}

#[derive(PartialEq)]
struct Transition {
    priority: TransitionPriority,
    target: AgentState,
}

#[derive(Component, Default)]
pub struct AgentStateQueue {
    next_states: Vec<Transition>,
}

#[derive(PartialOrd, PartialEq, Ord, Eq, Copy, Clone)]
pub enum TransitionPriority {
    Default,
    Important,
    Forced,
}

impl AgentStateQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, state: AgentState) {
        self.force_push(state, TransitionPriority::Default);
    }

    pub fn force_push(&mut self, state: AgentState, priority: TransitionPriority) {
        self.next_states.push(Transition {
            priority,
            target: state,
        });
    }
}

impl AgentState {
    fn importance(&self) -> u8 {
        match self {
            AgentState::Idle => 0,
            AgentState::PerformSkill(_) => 3,
            AgentState::Moving(_) => 2,
            AgentState::Sitting => 3,
            AgentState::PerformingAction(_) => 3,
            AgentState::PickingUp(_) => 3,
            AgentState::Dead => 4,
        }
    }

    fn apply_to<'a, 'b>(&self, commands: &'a mut EntityCommands<'b>) -> &'a mut EntityCommands<'b> {
        match self {
            AgentState::Idle => commands.insert(Idle),
            AgentState::PerformSkill(args) => commands.insert(PerformingSkill::new(*args)),
            AgentState::Moving(args) => commands.insert(Moving::new(*args)),
            AgentState::Sitting => commands.insert(Sitting),
            AgentState::PerformingAction(args) => commands.insert(PerformingAction::new(*args)),
            AgentState::PickingUp(args) => commands.insert(PickingUp::new(*args)),
            AgentState::Dead => commands.insert(Dead),
        }
    }

    fn is_similar_to(&self, other: &AgentState) -> bool {
        match (self, other) {
            (AgentState::Idle, AgentState::Idle) => true,
            (AgentState::PerformSkill(param), AgentState::PerformSkill(param2)) => param.eq(param2),
            (AgentState::Moving(param), AgentState::Moving(param2)) => param.is_similar(param2),
            (AgentState::Sitting, AgentState::Sitting) => true,
            (AgentState::PerformingAction(param), AgentState::PerformingAction(param2)) => param.eq(param2),
            (AgentState::PickingUp(param), AgentState::PickingUp(param2)) => param.eq(param2),
            (AgentState::Dead, AgentState::Dead) => true,
            _ => false,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            AgentState::Idle => "Idle",
            AgentState::PerformSkill(_) => "PerformingSkill",
            AgentState::Moving(_) => "Moving",
            AgentState::Sitting => "Sitting",
            AgentState::PerformingAction(_) => "PerformingAction",
            AgentState::PickingUp(_) => "PickingUp",
            AgentState::Dead => "Dead",
        }
    }
}

pub(crate) fn run_transitions(
    commands: ParallelCommands,
    mut query: Query<(
        Entity,
        &mut AgentStateQueue,
        Option<&Dead>,
        Option<&Idle>,
        Option<&PerformingSkill>,
        Option<&Moving>,
        Option<&Sitting>,
        Option<&PerformingAction>,
        Option<&PickingUp>,
    )>,
) {
    query.par_iter_mut().for_each(
        |(entity, mut state_queue, dead, idle, skill, moving, sitting, action, pickup)| {
            commands.command_scope(|mut commands| {
                let current_state = match (dead, idle, skill, moving, sitting, action, pickup) {
                    (Some(dead), _, _, _, _, _, _) => dead.as_state(),
                    (_, Some(idle), _, _, _, _, _) => idle.as_state(),
                    (_, _, Some(skill), _, _, _, _) => skill.as_state(),
                    (_, _, _, Some(moving), _, _, _) => moving.as_state(),
                    (_, _, _, _, Some(sitting), _, _) => sitting.as_state(),
                    (_, _, _, _, _, Some(action), _) => action.as_state(),
                    (_, _, _, _, _, _, Some(pickup)) => pickup.as_state(),
                    _ => {
                        commands.entity(entity).insert(Idle);
                        AgentState::Idle
                    },
                };

                let current_importance = current_state.importance();
                let mut incoming_transitions = mem::take(&mut state_queue.next_states);
                incoming_transitions.sort_by(|trans_a, trans_b| {
                    trans_a
                        .priority
                        .cmp(&trans_b.priority)
                        .reverse()
                        .then_with(|| trans_a.target.importance().cmp(&trans_b.target.importance()))
                });
                for next_state in incoming_transitions.into_iter() {
                    if next_state.priority == TransitionPriority::Forced
                        || (next_state.target.importance() > current_importance
                            && next_state.priority == TransitionPriority::Important)
                        || next_state.target.importance() >= current_importance
                    {
                        if !next_state.target.is_similar_to(&current_state) {
                            let mut entity_commands = commands.entity(entity);
                            entity_commands
                                .remove::<Dead>()
                                .remove::<PerformingSkill>()
                                .remove::<Moving>()
                                .remove::<Sitting>()
                                .remove::<PerformingAction>()
                                .remove::<PickingUp>()
                                .remove::<Idle>();

                            next_state.target.apply_to(&mut entity_commands);

                            commands.send_event(StateTransitionEvent {
                                entity,
                                from: current_state,
                                to: next_state.target,
                            });
                        }
                        break;
                    }
                }
            });
        },
    );
}
