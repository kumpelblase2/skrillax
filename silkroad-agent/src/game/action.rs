use crate::comp::drop::Drop;
use crate::comp::net::Client;
use crate::comp::{EntityReference, GameEntity};
use crate::game::mind::Mind;
use crate::input::PlayerInput;
use crate::world::{EntityLookup, WorldData};
use bevy_ecs::prelude::*;
use silkroad_protocol::combat::{ActionTarget, DoActionType, PerformAction, PerformActionError, PerformActionResponse};
use tracing::warn;

pub(crate) fn handle_action(
    mut query: Query<(&Client, &PlayerInput, &mut Mind)>,
    lookup: Res<EntityLookup>,
    target_query: Query<&GameEntity>,
    pickup_query: Query<&GameEntity, With<Drop>>,
) {
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

                        let Ok(found_target) = target_query.get(target) else {
                            client.send(PerformActionResponse::Stop(PerformActionError::InvalidTarget));
                            continue;
                        };

                        mind.attack(EntityReference(target, *found_target))
                    },
                    _ => continue,
                },
                DoActionType::PickupItem { target } => match target {
                    ActionTarget::Entity(unique_id) => {
                        let Some(target) = lookup.get_entity_for_id(*unique_id) else {
                            client.send(PerformActionResponse::Stop(PerformActionError::InvalidTarget));
                            continue;
                        };

                        let Ok(game_entity) = pickup_query.get(target) else {
                            client.send(PerformActionResponse::Stop(PerformActionError::InvalidTarget));
                            continue;
                        };

                        mind.pickup(EntityReference(target, *game_entity));
                    },
                    _ => continue,
                },
                DoActionType::UseSkill { ref_id, target } => match target {
                    ActionTarget::Entity(unique_id) => {
                        let Some(target) = lookup.get_entity_for_id(*unique_id) else {
                            client.send(PerformActionResponse::Stop(PerformActionError::InvalidTarget));
                            continue;
                        };

                        let Ok(found_target) = target_query.get(target) else {
                            client.send(PerformActionResponse::Stop(PerformActionError::InvalidTarget));
                            continue;
                        };

                        let Some(skill) = WorldData::skills().find_id(*ref_id) else {
                            client.send(PerformActionResponse::Stop(PerformActionError::NotLearned));
                            continue;
                        };

                        mind.attack_with(EntityReference(target, *found_target), skill)
                    },
                    _ => {
                        warn!("Tried to use a skill on unsupported target.")
                    },
                },
                DoActionType::CancelBuff { .. } => {},
            },
            PerformAction::Stop => mind.cancel(),
        }
    }
}
