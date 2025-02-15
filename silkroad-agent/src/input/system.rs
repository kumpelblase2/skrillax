use crate::comp::net::{Client, LastAction};
use crate::config::GameConfig;
use crate::event::{ClientDisconnectedEvent, LoadingFinishedEvent};
use crate::input::{LoginInput, PlayerInput};
use crate::mall::event::MallOpenRequestEvent;
use crate::protocol::AgentClientProtocol;
use bevy::prelude::*;
use silkroad_game_base::StatType;
use silkroad_protocol::auth::AuthProtocol;
use silkroad_protocol::character::CharselectClientProtocol;
use silkroad_protocol::combat::CombatClientProtocol;
use silkroad_protocol::general::{BaseProtocol, IdentityInformation};
use silkroad_protocol::gm::GmClientProtocol;
use silkroad_protocol::inventory::{ConsignmentResponse, InventoryClientProtocol};
use silkroad_protocol::movement::MovementClientProtocol;
use silkroad_protocol::skill::SkillClientProtocol;
use silkroad_protocol::world::{GameGuideResponse, StatClientProtocol, WorldClientProtocol};
use std::time::Instant;

pub(crate) fn reset(mut player_input: Query<&mut PlayerInput>, mut login_input: Query<&mut LoginInput>) {
    for mut input in player_input.iter_mut() {
        input.reset();
    }

    for mut input in login_input.iter_mut() {
        input.reset();
    }
}

pub(crate) fn receive_game_inputs(
    mut query: Query<(Entity, &Client, &mut PlayerInput, &mut LastAction)>,
    time: Res<Time<Real>>,
    settings: Res<GameConfig>,
    mut loading_events: EventWriter<LoadingFinishedEvent>,
    mut disconnect_events: EventWriter<ClientDisconnectedEvent>,
    mut mall_events: EventWriter<MallOpenRequestEvent>,
) {
    for (entity, client, mut input, mut last_action) in query.iter_mut() {
        let mut had_action = false;
        loop {
            match client.next() {
                Ok(Some(packet)) => {
                    had_action = true;
                    match *packet {
                        AgentClientProtocol::ChatClientProtocol(chat) => {
                            input.chat.push(chat);
                        },
                        AgentClientProtocol::MovementClientProtocol(MovementClientProtocol::PlayerMovementRequest(
                            req,
                        )) => {
                            input.movement = Some(req.kind);
                        },
                        AgentClientProtocol::MovementClientProtocol(MovementClientProtocol::Rotation(rotate)) => {
                            input.rotation = Some(rotate);
                        },
                        AgentClientProtocol::FriendListClientProtocol(friend_list) => {},
                        AgentClientProtocol::SkillClientProtocol(SkillClientProtocol::LearnSkill(skill)) => {
                            input.skill_add = Some(skill);
                        },
                        AgentClientProtocol::SkillClientProtocol(SkillClientProtocol::LevelUpMastery(mastery)) => {
                            input.mastery = Some(mastery);
                        },
                        AgentClientProtocol::SkillClientProtocol(SkillClientProtocol::HotbarUpdate(hotbar)) => {
                            input.hotbar = Some(hotbar.content);
                        },
                        AgentClientProtocol::StatClientProtocol(stat) => match stat {
                            StatClientProtocol::IncreaseStr(_) => input.increase_stats.push(StatType::STR),
                            StatClientProtocol::IncreaseInt(_) => input.increase_stats.push(StatType::INT),
                        },
                        AgentClientProtocol::CombatClientProtocol(CombatClientProtocol::PerformAction(action)) => {
                            input.action = Some(action);
                        },
                        AgentClientProtocol::WorldClientProtocol(world) => match world {
                            WorldClientProtocol::TargetEntity(target) => {
                                input.target = Some(target);
                            },
                            WorldClientProtocol::UnTargetEntity(untarget) => {
                                input.untarget = Some(untarget);
                            },
                            WorldClientProtocol::UpdateGameGuide(guide) => {
                                client.send(GameGuideResponse::Success(guide.0));
                            },
                        },
                        AgentClientProtocol::CharselectClientProtocol(CharselectClientProtocol::FinishLoading(_)) => {
                            loading_events.send(LoadingFinishedEvent(entity));
                        },
                        AgentClientProtocol::InventoryClientProtocol(inventory) => match inventory {
                            InventoryClientProtocol::OpenItemMall(_) => {
                                mall_events.send(MallOpenRequestEvent(entity));
                            },
                            InventoryClientProtocol::InventoryOperation(inventory) => {
                                input.inventory = Some(inventory);
                            },
                            InventoryClientProtocol::ConsignmentList(_) => {
                                client.send(ConsignmentResponse::success_empty());
                            },
                        },
                        AgentClientProtocol::AuthProtocol(AuthProtocol::LogoutRequest(logout)) => {
                            input.logout = Some(logout);
                        },
                        AgentClientProtocol::GmClientProtocol(GmClientProtocol::GmCommand(command)) => {
                            input.gm = Some(command);
                        },
                        _ => {},
                    }
                },
                Ok(None) => {
                    break;
                },
                Err(_) => {
                    disconnect_events.send(ClientDisconnectedEvent(entity));
                    break;
                },
            }
        }

        let last_tick_time = time.last_update().unwrap_or_else(Instant::now);
        if had_action {
            last_action.0 = last_tick_time;
        }

        if last_tick_time.duration_since(last_action.0).as_secs() > settings.client_timeout.into() {
            disconnect_events.send(ClientDisconnectedEvent(entity));
        }
    }
}

pub(crate) fn receive_login_inputs(
    mut query: Query<(Entity, &Client, &mut LoginInput, &mut LastAction)>,
    time: Res<Time<Real>>,
    settings: Res<GameConfig>,
    mut disconnect_events: EventWriter<ClientDisconnectedEvent>,
) {
    for (entity, client, mut input, mut last_action) in query.iter_mut() {
        let mut had_action = false;
        loop {
            match client.next() {
                Ok(Some(packet)) => {
                    had_action = true;
                    match *packet {
                        AgentClientProtocol::CharselectClientProtocol(charselect) => match charselect {
                            CharselectClientProtocol::CharacterListRequest(request) => {
                                input.list.push(request.action);
                            },
                            CharselectClientProtocol::CharacterJoinRequest(join) => {
                                input.join = Some(join);
                            },
                            _ => {},
                        },
                        AgentClientProtocol::AuthProtocol(AuthProtocol::AuthRequest(auth)) => {
                            input.auth = Some(auth);
                        },
                        AgentClientProtocol::BaseProtocol(BaseProtocol::IdentityInformation(_id)) => {
                            send_identity_information(client)
                        },
                        _ => {},
                    }
                },
                Ok(None) => {
                    break;
                },
                Err(_) => {
                    disconnect_events.send(ClientDisconnectedEvent(entity));
                    break;
                },
            }
        }

        let last_tick_time = time.last_update().unwrap_or_else(Instant::now);
        if had_action {
            last_action.0 = last_tick_time;
        }

        if last_tick_time.duration_since(last_action.0).as_secs() > settings.client_timeout.into() {
            disconnect_events.send(ClientDisconnectedEvent(entity));
        }
    }
}

fn send_identity_information(client: &Client) {
    client.send(IdentityInformation::new("AgentServer".to_string(), 0))
}
