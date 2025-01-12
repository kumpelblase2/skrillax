use crate::comp::monster::Monster;
use crate::comp::net::Client;
use crate::comp::npc::NPC;
use crate::comp::player::Player;
use crate::comp::pos::Position;
use crate::comp::Health;
use crate::input::PlayerInput;
use crate::world::EntityLookup;
use bevy_ecs::prelude::*;
use cgmath::MetricSpace;
use derive_more::Deref;
use silkroad_protocol::world::{TargetEntityError, TargetEntityResponse, TargetEntityResult, UnTargetEntityResponse};

const MAX_TARGET_DISTANCE: f32 = 500. * 500.;

#[derive(Component, Deref)]
#[component(storage = "SparseSet")]
pub(crate) struct Target(Entity);

impl Target {
    pub fn entity(&self) -> Entity {
        self.0
    }
}

pub(crate) fn player_update_target(
    query: Query<(Entity, &Client, &PlayerInput, &Position, Option<&Target>)>,
    mut cmd: Commands,
    lookup: Res<EntityLookup>,
    target_lookup: Query<(
        &Position,
        Option<&Health>,
        Option<&Monster>,
        Option<&NPC>,
        Option<&Player>,
    )>,
) {
    for (entity, client, input, pos, current_target) in query.iter() {
        'target: {
            if let Some(ref target) = input.target {
                if let Some(target_entity) = lookup.get_entity_for_id(target.unique_id) {
                    if let Ok((target_pos, health, monster, npc, player)) = target_lookup.get(target_entity) {
                        let distance = target_pos.position().distance2(pos.position().0);
                        if distance >= MAX_TARGET_DISTANCE {
                            // Is this an adequate response?
                            client.send(TargetEntityResponse::new(TargetEntityResult::failure(
                                TargetEntityError::InvalidTarget,
                            )));
                            break 'target; // TODO
                        }

                        match (health, monster, npc, player) {
                            (Some(health), Some(_), _, _) => {
                                client.send(TargetEntityResponse::new(TargetEntityResult::success_monster(
                                    target.unique_id,
                                    health.current_health,
                                )));
                            },
                            (_, _, Some(_), _) => {
                                client.send(TargetEntityResponse::new(TargetEntityResult::success_npc(
                                    target.unique_id,
                                )));
                            },
                            (Some(health), _, _, Some(player)) => {},
                            _ => {
                                client.send(TargetEntityResponse::new(TargetEntityResult::failure(
                                    TargetEntityError::InvalidTarget,
                                )));
                                break 'target;
                            },
                        }
                        cmd.entity(entity).try_insert(Target(target_entity));
                    } else {
                        client.send(TargetEntityResponse::new(TargetEntityResult::failure(
                            TargetEntityError::InvalidTarget,
                        )));
                    };
                } else {
                    client.send(TargetEntityResponse::new(TargetEntityResult::failure(
                        TargetEntityError::InvalidTarget,
                    )));
                }
            }
        }

        if let Some(ref untarget) = input.untarget {
            let Some(target) = current_target else {
                client.send(UnTargetEntityResponse::new(true));
                continue;
            };
            let Some(found) = lookup.get_entity_for_id(untarget.unique_id) else {
                client.send(UnTargetEntityResponse::new(false));
                continue;
            };

            if found == target.0 {
                cmd.entity(entity).remove::<Target>();
                client.send(UnTargetEntityResponse::new(true));
            } else {
                client.send(UnTargetEntityResponse::new(false));
            }
        }
    }
}

pub(crate) fn deselect_despawned(
    mut query: Query<(Entity, Option<&Client>, &mut Target)>,
    target_query: Query<()>,
    mut cmd: Commands,
) {
    for (entity, client, target) in query.iter_mut() {
        if target_query.get(target.0).is_err() {
            cmd.entity(entity).remove::<Target>();
            if let Some(client) = client {
                client.send(UnTargetEntityResponse::new(true));
            }
        }
    }
}
