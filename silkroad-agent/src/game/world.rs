use crate::comp::monster::Monster;
use crate::comp::net::WorldInput;
use crate::comp::npc::NPC;
use crate::comp::player::{Agent, AgentAction, Item, Player};
use crate::comp::pos::{GlobalPosition, Position};
use crate::comp::{Client, GameEntity, Health};
use crate::event::ClientDisconnectedEvent;
use crate::world::{EntityLookup, CHARACTERS, SKILLS};
use crate::GameSettings;
use bevy_core::{Time, Timer};
use bevy_ecs::prelude::*;
use cgmath::num_traits::Pow;
use cgmath::{InnerSpace, MetricSpace, Vector3};
use silkroad_data::skilldata::RefSkillData;
use silkroad_data::type_id::{ObjectEquippable, ObjectItem, ObjectType, ObjectWeaponType};
use silkroad_data::DataMap;
use silkroad_protocol::auth::{LogoutFinished, LogoutResponse, LogoutResult};
use silkroad_protocol::combat::{ActionTarget, DoActionType, PerformAction, PerformActionResponse};
use silkroad_protocol::world::{
    TargetEntity, TargetEntityError, TargetEntityResponse, TargetEntityResult, UnTargetEntityResponse,
};
use silkroad_protocol::{ClientPacket, ServerPacket};
use std::mem::take;
use std::ops::Add;
use std::time::Duration;

const PUNCH_SKILL_ID: u32 = 1;
const MAX_TARGET_DISTANCE: f32 = 500. * 500.;

enum NextAction<'a> {
    MoveTo(Vector3<f32>),
    UseSkill(&'a RefSkillData),
}

pub(crate) fn handle_world_input(
    delta: Res<Time>,
    mut query: Query<(Entity, &Client, &Position, &mut WorldInput)>,
    mut target_lookup: ParamSet<(
        Query<(
            &GameEntity,
            &Position,
            Option<&Health>,
            Option<&Monster>,
            Option<&NPC>,
            Option<&Player>,
        )>,
        Query<(&mut Player, &mut Agent)>,
    )>,
    settings: Res<GameSettings>,
    lookup: Res<EntityLookup>,
) {
    // This kinda works, but is quite horrible workaround.
    // The main problem is that in the case of a logout or a successful target, we need to modify the player.
    // However, we also need to a normal reference to a player, which interferes with the exclusivity requirement
    // of the previous query.
    for (entity, client, player_pos, mut input) in query.iter_mut() {
        for packet in take(&mut input.inputs) {
            match packet {
                ClientPacket::LogoutRequest(logout) => {
                    let mut player_query = target_lookup.p1();
                    let (mut player, _) = player_query.get_mut(entity).unwrap();
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
                    let target_query = target_lookup.p0();

                    let (target_entity, target) = match lookup
                        .get_entity_for_id(unique_id)
                        .and_then(|entity| target_query.get(entity).ok().map(|data| (entity, data)))
                    {
                        Some(content) => content,
                        None => {
                            client.send(ServerPacket::TargetEntityResponse(TargetEntityResponse::new(
                                TargetEntityResult::failure(TargetEntityError::InvalidTarget),
                            )));
                            continue;
                        },
                    };

                    match target {
                        (_, pos, Some(health), Some(_mob), _, _) => {
                            let distance = pos.location.0.distance2(player_pos.location.0);
                            if distance < MAX_TARGET_DISTANCE {
                                client.send(ServerPacket::TargetEntityResponse(TargetEntityResponse::new(
                                    TargetEntityResult::success_monster(unique_id, health.current_health),
                                )));
                            } else {
                                // Is this an adequate response?
                                client.send(ServerPacket::TargetEntityResponse(TargetEntityResponse::new(
                                    TargetEntityResult::failure(TargetEntityError::InvalidTarget),
                                )));
                                continue;
                            }
                        },
                        (_entity, _, _, _, Some(npc), _) => {},
                        (_entity, _, _, _, _, Some(player)) => {},
                        _ => {
                            client.send(ServerPacket::TargetEntityResponse(TargetEntityResponse::new(
                                TargetEntityResult::failure(TargetEntityError::InvalidTarget),
                            )));
                            continue;
                        },
                    }

                    let mut agent_lookup = target_lookup.p1();
                    let (_, mut agent) = agent_lookup.get_mut(entity).unwrap();
                    agent.target = Some(target_entity);
                },
                ClientPacket::UnTargetEntity(_) => {
                    let mut agent_lookup = target_lookup.p1();
                    let (_, mut agent) = agent_lookup.get_mut(entity).unwrap();
                    agent.target = None;
                    client.send(ServerPacket::UnTargetEntityResponse(UnTargetEntityResponse::new(true)));
                },
                ClientPacket::PerformAction(action) => match action {
                    PerformAction::Do(DoActionType::Attack { target }) => {
                        let target_query = target_lookup.p0();

                        let target = match target {
                            ActionTarget::Entity(entity) => entity,
                            _ => {
                                client.send(ServerPacket::PerformActionResponse(PerformActionResponse::Do(2)));
                                continue;
                            },
                        };

                        let target = match lookup
                            .get_entity_for_id(target)
                            .and_then(|entity| target_query.get(entity).ok().map(|data| (entity, data)))
                        {
                            Some(entity) => entity,
                            None => {
                                client.send(ServerPacket::PerformActionResponse(PerformActionResponse::Do(2)));
                                continue;
                            },
                        };

                        let (player_entity, _, _, _, _, player) = target_query.get(entity).unwrap();
                        let player = player.unwrap();
                        let (attack_skill, range) = match get_default_attack_range(player_entity, player) {
                            Ok(inner) => inner,
                            Err(_) => {
                                client.send(ServerPacket::PerformActionResponse(PerformActionResponse::Do(2)));
                                continue;
                            },
                        };

                        let next = match target {
                            (target_ref, (entity, pos, Some(health), Some(mob), _, _)) => {
                                client.send(ServerPacket::PerformActionResponse(PerformActionResponse::Do(1)));
                                let distance = pos.location.0.distance2(player_pos.location.0);
                                if distance <= range.pow(2) {
                                    AgentAction::Attack {
                                        target: target_ref,
                                        range,
                                        reference: attack_skill,
                                        current_destination: player_pos.location,
                                        refresh_timer: Timer::new(Duration::from_secs(1), true),
                                    }
                                } else {
                                    let direction_vector = player_pos.location.0 - pos.location.0;
                                    let position_offset = direction_vector.normalize() * (range - 1.);
                                    let dest = pos.location.0 + position_offset;
                                    AgentAction::Attack {
                                        target: target_ref,
                                        range,
                                        reference: attack_skill,
                                        current_destination: GlobalPosition(dest),
                                        refresh_timer: Timer::new(Duration::from_secs(1), true),
                                    }
                                }
                            },
                            _ => continue,
                        };

                        let mut player_lookup = target_lookup.p1();
                        let (_, mut agent) = player_lookup.get_mut(entity).unwrap();
                        agent.next_action = Some(next);
                    },
                    _ => {},
                },
                _ => {},
            }
        }
    }
}

fn get_default_attack_range(
    player_entity: &GameEntity,
    player: &Player,
) -> Result<(&'static RefSkillData, f32), AttackSkillError> {
    let weapon = player.inventory.weapon();
    let attack_skill = get_attack_skill(&SKILLS.get().unwrap(), weapon)?;
    let character_data = CHARACTERS.get().unwrap().find_id(player_entity.ref_id).unwrap();
    let weapon_data = weapon.map(|weapon| weapon.reference);
    let range = vec![
        attack_skill.range,
        character_data.base_range,
        weapon_data
            .and_then(|weapon| weapon.range)
            .map(|r| r.get())
            .unwrap_or(0),
    ]
    .into_iter()
    .max()
    .unwrap_or(0) as f32;
    Ok((attack_skill, range))
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

pub(crate) enum AttackSkillError {
    NotAWeapon,
    SkillNotFound,
    UnknownWeapon,
}

fn get_attack_skill<'a>(
    skills: &'a DataMap<RefSkillData>,
    weapon: Option<&Item>,
) -> Result<&'a RefSkillData, AttackSkillError> {
    let skill = if let Some(weapon) = weapon {
        let item_type = match ObjectType::from_type_id(&weapon.reference.type_id).unwrap() {
            ObjectType::Item(item) => item,
            _ => return Err(AttackSkillError::NotAWeapon),
        };
        match item_type {
            ObjectItem::Equippable(equipment) => match equipment {
                ObjectEquippable::Weapon(weapon_type) => match weapon_type {
                    ObjectWeaponType::Sword | ObjectWeaponType::Blade => {
                        skills.find_id(2).ok_or(AttackSkillError::SkillNotFound)?
                    },
                    ObjectWeaponType::Spear | ObjectWeaponType::Glavie => {
                        skills.find_id(40).ok_or(AttackSkillError::SkillNotFound)?
                    },
                    ObjectWeaponType::Bow => skills.find_id(70).ok_or(AttackSkillError::SkillNotFound)?,
                    ObjectWeaponType::OneHandSword => skills.find_id(7127).ok_or(AttackSkillError::SkillNotFound)?,
                    ObjectWeaponType::TwoHandSword => skills.find_id(7128).ok_or(AttackSkillError::SkillNotFound)?,
                    ObjectWeaponType::Axe => skills.find_id(7129).ok_or(AttackSkillError::SkillNotFound)?,
                    ObjectWeaponType::WarlockStaff => skills.find_id(9069).ok_or(AttackSkillError::SkillNotFound)?,
                    ObjectWeaponType::Staff => skills.find_id(8454).ok_or(AttackSkillError::SkillNotFound)?,
                    ObjectWeaponType::Crossbow => skills.find_id(7909).ok_or(AttackSkillError::SkillNotFound)?,
                    ObjectWeaponType::Dagger => skills.find_id(7910).ok_or(AttackSkillError::SkillNotFound)?,
                    ObjectWeaponType::Harp => skills.find_id(9606).ok_or(AttackSkillError::SkillNotFound)?,
                    ObjectWeaponType::ClericRod => skills.find_id(9970).ok_or(AttackSkillError::SkillNotFound)?,
                    _ => return Err(AttackSkillError::UnknownWeapon),
                },
                _ => return Err(AttackSkillError::NotAWeapon),
            },
            _ => return Err(AttackSkillError::NotAWeapon),
        }
    } else {
        skills.find_id(PUNCH_SKILL_ID).ok_or(AttackSkillError::SkillNotFound)?
    };
    Ok(skill)
}
