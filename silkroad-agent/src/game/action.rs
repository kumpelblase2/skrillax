use crate::agent::goal::{AgentGoal, GoalTracker};
use crate::comp::net::Client;
use crate::input::PlayerInputEvent;
use crate::world::{EntityLookup, WorldData};
use bevy::prelude::*;
use silkroad_protocol::combat::{ActionTarget, DoActionType, PerformAction, PerformActionError, PerformActionResponse};
use tracing::warn;

pub(crate) fn handle_action(
    mut query: Query<(&Client, &mut GoalTracker)>,
    lookup: Res<EntityLookup>,
    mut reader: MessageReader<PlayerInputEvent<PerformAction>>,
) {
    for event in reader.read() {
        let (client, mut mind) = query.get_mut(event.player).unwrap();
        match &event.input {
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
