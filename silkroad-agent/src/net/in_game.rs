use crate::comp::player::{Agent, Character, MovementState, MovementTarget, Player, SpawningState};
use crate::comp::pos::{Heading, LocalPosition, Position};
use crate::comp::sync::{MovementUpdate, Synchronize};
use crate::comp::{Client, GameEntity};
use crate::event::{ChatEvent, ClientDisconnectedEvent};
use crate::world::EntityLookup;
use crate::GameSettings;
use bevy_core::Time;
use bevy_ecs::prelude::*;
use cgmath::Vector3;
use silkroad_protocol::auth::{LogoutFinished, LogoutRequest, LogoutResponse, LogoutResult};
use silkroad_protocol::character::CharacterStatsMessage;
use silkroad_protocol::chat::{
    ChatMessageResponse, ChatMessageResult, ChatSource, ChatTarget, ChatUpdate, TextCharacterInitialization,
};
use silkroad_protocol::world::{
    CelestialUpdate, EntityUpdateState, PlayerMovementRequest, Rotation, TargetEntity, TargetEntityResponse,
    TargetEntityResult,
};
use silkroad_protocol::{ClientPacket, ServerPacket};
use std::ops::Add;
use std::time::Duration;
use tracing::debug;

pub(crate) fn in_game(
    mut disconnect_events: EventWriter<ClientDisconnectedEvent>,
    mut chat_messages: EventWriter<ChatEvent>,
    mut lookup: ResMut<EntityLookup>,
    settings: Res<GameSettings>,
    delta: Res<Time>,
    mut query: Query<(
        Entity,
        &mut Client,
        &GameEntity,
        &mut Player,
        &mut Agent,
        &Position,
        &mut Synchronize,
    )>,
) {
    for (entity, mut client, game_entity, mut player, mut agent, position, mut sync) in query.iter_mut() {
        while let Some(packet) = client.1.pop_front() {
            match packet {
                ClientPacket::FinishLoading(_) => {
                    debug!(id = ?client.0.id(), "Finished loading.");
                    player.character.state = SpawningState::Finished;
                    send_celestial_status(&client, game_entity.unique_id);
                    send_character_stats(&client, &player.character);
                    send_text_initialization(&client);
                    client.send(ServerPacket::EntityUpdateState(EntityUpdateState::new(
                        game_entity.unique_id,
                        0,
                        1,
                    )));
                    if let Some(notice) = &settings.join_notice {
                        client.send(ChatUpdate::new(ChatSource::Notice, notice.clone()));
                    }
                    lookup.add_player(player.character.name.clone(), entity, game_entity.unique_id);
                },
                ClientPacket::PlayerMovementRequest(PlayerMovementRequest { kind }) => match kind {
                    silkroad_protocol::world::MovementTarget::TargetLocation { region, x, y, z } => {
                        let local_position = position.location.to_local();
                        let target_pos = LocalPosition(region.into(), Vector3::new(x as f32, y as f32, z as f32));
                        debug!(id = ?client.0.id(), "Movement: {} -> {}", local_position, target_pos);
                        sync.movement = Some(MovementUpdate::StartMove(local_position, target_pos.clone()));
                        agent.movement_target = Some(MovementTarget::Location(target_pos.to_global()));
                        agent.movement_state = MovementState::Moving;
                    },
                    silkroad_protocol::world::MovementTarget::Direction { unknown, angle } => {
                        let direction = Heading::from(angle);
                        debug!(id = ?client.0.id(), "Movement: {} / {}({})", unknown, direction.0, angle);
                        let local_position = position.location.to_local();
                        sync.movement = Some(MovementUpdate::StartMoveTowards(local_position, direction.clone()));
                        agent.movement_target = Some(MovementTarget::Direction(direction));
                    },
                },
                ClientPacket::Rotation(Rotation { heading }) => {
                    let heading = Heading::from(heading);
                    if agent.movement_target.is_none() {
                        agent.movement_target = Some(MovementTarget::Turn(heading));
                    }
                },
                ClientPacket::ChatMessage(message) => {
                    debug!(id = ?client.0.id(), "Received chat message: {} @ {}", message.message, message.index);
                    match message.target {
                        ChatTarget::All | ChatTarget::AllGm => {
                            if message.message.starts_with(".") && player.character.gm {
                                chat_messages.send(ChatEvent::Command {
                                    sender: entity,
                                    message: message.message,
                                });
                            } else {
                                chat_messages.send(ChatEvent::RegionalChat {
                                    sender: entity,
                                    sender_unique_id: game_entity.unique_id,
                                    position: position.location.0,
                                    message: message.message,
                                });
                            }
                        },
                        ChatTarget::PrivateMessage => {
                            chat_messages.send(ChatEvent::PrivateChat {
                                sender: player.character.name.clone(),
                                target: message.recipient.unwrap(),
                                message: message.message,
                            });
                        },
                        _ => {},
                    }

                    // TODO: this currently just says "success" to everything, but does not deal with errors,
                    //  like the target not being online.
                    client.send(ChatMessageResponse::new(
                        ChatMessageResult::Success,
                        message.target,
                        message.index,
                    ));
                },
                ClientPacket::LogoutRequest(LogoutRequest { mode }) => {
                    player.logout = Some(
                        delta
                            .last_update()
                            .unwrap()
                            .add(Duration::from_secs(settings.logout_duration as u64)),
                    );
                    client.send(LogoutResponse::new(LogoutResult::success(
                        settings.logout_duration as u32,
                        mode,
                    )));
                },
                ClientPacket::TargetEntity(TargetEntity { unique_id }) => {
                    client.send(ServerPacket::TargetEntityResponse(TargetEntityResponse::new(
                        TargetEntityResult::success_monster(unique_id, 54),
                    )));
                },
                _ => {},
            }
        }

        if let Some(logout_time) = player.logout {
            if delta.last_update().unwrap() > logout_time {
                client.send(LogoutFinished);
                disconnect_events.send(ClientDisconnectedEvent(entity));
            }
        }
    }
}

fn send_celestial_status(client: &Client, my_id: u32) {
    client.send(CelestialUpdate::new(my_id, 0x75, 0x13, 0x1c));
}

fn send_character_stats(client: &Client, character: &Character) {
    client.send(CharacterStatsMessage::new(
        100,
        100,
        100,
        100,
        100,
        100,
        100,
        100,
        character.max_hp(),
        character.max_mp(),
        character.stats.strength(),
        character.stats.intelligence(),
    ));
}

fn send_text_initialization(client: &Client) {
    let mut characters = Vec::new();
    for i in 0x1d..0x8c {
        if i < 0x85 || i >= 0x89 {
            characters.push((i as u64) << 56);
        }
    }

    client.send(TextCharacterInitialization::new(characters));
}

fn match_chat_target_to_source(target: &ChatTarget, sender_id: u32, sender_name: &str) -> ChatSource {
    match target {
        ChatTarget::All => ChatSource::All { sender: sender_id },
        ChatTarget::AllGm => ChatSource::AllGm { sender: sender_id },
        ChatTarget::NPC => ChatSource::NPC { sender: sender_id },
        ChatTarget::PrivateMessage => ChatSource::PrivateMessage {
            sender: sender_name.to_string(),
        },
        ChatTarget::Party => ChatSource::Party {
            sender: sender_name.to_string(),
        },
        ChatTarget::Guild => ChatSource::Guild {
            sender: sender_name.to_string(),
        },
        ChatTarget::Global => ChatSource::Global {
            sender: sender_name.to_string(),
        },
        ChatTarget::Stall => ChatSource::Stall {
            sender: sender_name.to_string(),
        },
        ChatTarget::Union => ChatSource::Union {
            sender: sender_name.to_string(),
        },
        ChatTarget::Academy => ChatSource::Academy {
            sender: sender_name.to_string(),
        },
        ChatTarget::Notice => ChatSource::Notice,
    }
}
