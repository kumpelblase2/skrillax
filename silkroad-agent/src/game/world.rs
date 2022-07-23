use crate::comp::monster::Monster;
use crate::comp::net::WorldInput;
use crate::comp::npc::NPC;
use crate::comp::player::Player;
use crate::comp::{Client, GameEntity, Health};
use crate::event::ClientDisconnectedEvent;
use crate::world::EntityLookup;
use crate::GameSettings;
use bevy_core::Time;
use bevy_ecs::prelude::*;
use silkroad_data::characterdata::CharacterMap;
use silkroad_protocol::auth::{LogoutFinished, LogoutResponse, LogoutResult};
use silkroad_protocol::world::{
    TargetEntity, TargetEntityError, TargetEntityResponse, TargetEntityResult, UnTargetEntityResponse,
};
use silkroad_protocol::{ClientPacket, ServerPacket};
use std::mem::take;
use std::ops::Add;
use std::time::Duration;

pub(crate) fn handle_world_input(
    delta: Res<Time>,
    mut query: Query<(Entity, &Client, &mut WorldInput)>,
    mut target_lookup: ParamSet<(
        Query<(
            &GameEntity,
            Option<&Health>,
            Option<&Monster>,
            Option<&NPC>,
            Option<&Player>,
        )>,
        Query<&mut Player>,
    )>,
    settings: Res<GameSettings>,
    lookup: Res<EntityLookup>,
    _character_data: Res<CharacterMap>,
) {
    // This kinda works, but is quite horrible workaround.
    // The main problem is that in the case of a logout or a successful target, we need to modify the player.
    // However, we also need to a normal reference to a player, which interferes with the exclusivity requirement
    // of the previous query.
    for (entity, client, mut input) in query.iter_mut() {
        for packet in take(&mut input.inputs) {
            match packet {
                ClientPacket::LogoutRequest(logout) => {
                    let mut player_query = target_lookup.p1();
                    let mut player = player_query.get_mut(entity).unwrap();
                    player.logout = Some(
                        delta
                            .last_update()
                            .unwrap()
                            .add(Duration::from_secs(settings.logout_duration as u64)),
                    );
                    client.send(LogoutResponse::new(LogoutResult::success(
                        settings.logout_duration as u32,
                        logout.mode,
                    )));
                },
                ClientPacket::TargetEntity(TargetEntity { unique_id }) => {
                    let entity_for_target = match lookup.get_entity_for_id(unique_id) {
                        Some(entity) => entity,
                        None => {
                            client.send(ServerPacket::TargetEntityResponse(TargetEntityResponse::new(
                                TargetEntityResult::failure(TargetEntityError::InvalidTarget),
                            )));
                            continue;
                        },
                    };

                    let target_query = target_lookup.p0();
                    let target = match target_query.get(entity_for_target) {
                        Ok(res) => res,
                        _ => {
                            client.send(ServerPacket::TargetEntityResponse(TargetEntityResponse::new(
                                TargetEntityResult::failure(TargetEntityError::InvalidTarget),
                            )));
                            continue;
                        },
                    };

                    match target {
                        (_, Some(health), Some(_mob), _, _) => {
                            client.send(ServerPacket::TargetEntityResponse(TargetEntityResponse::new(
                                TargetEntityResult::success_monster(unique_id, health.current_health),
                            )));
                        },
                        (_entity, _, _, Some(npc), _) => {},
                        (_entity, _, _, _, Some(player)) => {},
                        _ => {
                            client.send(ServerPacket::TargetEntityResponse(TargetEntityResponse::new(
                                TargetEntityResult::failure(TargetEntityError::InvalidTarget),
                            )));
                            continue;
                        },
                    }

                    target_lookup.p1().get_mut(entity).unwrap().target = Some(entity_for_target);
                },
                ClientPacket::UnTargetEntity(_) => {
                    target_lookup.p1().get_mut(entity).unwrap().target = None;
                    client.send(ServerPacket::UnTargetEntityResponse(UnTargetEntityResponse::new(true)));
                },
                _ => {},
            }
        }
    }
}

pub(crate) fn finish_logout(
    query: Query<(Entity, &Client, &Player)>,
    delta: Res<Time>,
    mut disconnect_events: EventWriter<ClientDisconnectedEvent>,
) {
    for (entity, client, player) in query.iter() {
        if let Some(logout_time) = player.logout {
            if delta.last_update().unwrap() > logout_time {
                client.send(ServerPacket::LogoutFinished(LogoutFinished));
                disconnect_events.send(ClientDisconnectedEvent(entity));
            }
        }
    }
}
