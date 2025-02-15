use crate::agent::goal::{AgentGoal, GoalTracker};
use crate::comp::net::Client;
use crate::input::PlayerInput;
use crate::world::{EntityLookup, WorldData};
use bevy::prelude::*;
use silkroad_protocol::combat::{ActionTarget, DoActionType, PerformAction, PerformActionError, PerformActionResponse};
use tracing::warn;

pub(crate) fn handle_action(mut query: Query<(&Client, &PlayerInput, &mut GoalTracker)>, lookup: Res<EntityLookup>) {
    for (client, input, mut mind) in query.iter_mut() {
        let Some(ref action) = input.action else {
            continue;
        };

        match action {
            PerformAction::Do(action) => match action {
                DoActionType::Attack { target } => match target {
                    ActionTarget::Entity(unique_id) => {
                        let Some(target) = lookup.get_entity_for_id(*unique_id) else {
                            client.send(PerformActionResponse::Stop(PerformActionError::InvalidTarget));
                            continue;
                        };

                        mind.switch_goal_notified(AgentGoal::attacking(target));
                    },
                    _ => continue,
                },
                DoActionType::PickupItem { target } => match target {
                    ActionTarget::Entity(unique_id) => {
                        let Some(target) = lookup.get_entity_for_id(*unique_id) else {
                            client.send(PerformActionResponse::Stop(PerformActionError::InvalidTarget));
                            continue;
                        };

                        mind.switch_goal_notified(AgentGoal::picking_up(target));
                    },
                    _ => continue,
                },
                DoActionType::UseSkill { ref_id, target } => match target {
                    ActionTarget::Entity(unique_id) => {
                        let Some(target) = lookup.get_entity_for_id(*unique_id) else {
                            client.send(PerformActionResponse::Stop(PerformActionError::InvalidTarget));
                            continue;
                        };

                        let Some(skill) = WorldData::skills().find_id(*ref_id) else {
                            client.send(PerformActionResponse::Stop(PerformActionError::NotLearned));
                            continue;
                        };

                        mind.switch_goal_notified(AgentGoal::attacking_with(target, skill));
                    },
                    _ => {
                        warn!("Tried to use a skill on unsupported target.")
                    },
                },
                DoActionType::CancelBuff { .. } => {},
            },
            PerformAction::Stop => {
                mind.reset();
            },
        }
    }
}
