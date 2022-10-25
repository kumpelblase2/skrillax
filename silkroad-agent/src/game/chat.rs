use crate::comp::net::{ChatInput, Client};
use crate::comp::player::Player;
use crate::comp::visibility::Visibility;
use crate::comp::GameEntity;
use crate::game::gm::handle_gm_commands;
use crate::world::EntityLookup;
use bevy_app::{App, Plugin};
use bevy_ecs::prelude::*;
use silkroad_protocol::chat::{
    ChatErrorCode, ChatMessageResponse, ChatMessageResult, ChatSource, ChatTarget, ChatUpdate,
};
use silkroad_protocol::ClientPacket;
use std::mem::take;
use tracing::debug;

pub(crate) struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(handle_chat).add_system(handle_gm_commands);
    }
}

fn handle_chat(
    mut query: Query<(&Client, &GameEntity, &mut ChatInput, &Visibility, &Player)>,
    lookup: Res<EntityLookup>,
    others: Query<(&Client, &Player)>,
) {
    for (client, game_entity, mut chat_input, visibility, player) in query.iter_mut() {
        for packet in take(&mut chat_input.inputs) {
            match packet {
                ClientPacket::ChatMessage(message) => {
                    debug!(id = ?client.0.id(), "Received chat message: {} @ {}", message.message, message.index);
                    match message.target {
                        ChatTarget::All => {
                            visibility.entities_in_radius.iter().for_each(|e| {
                                if let Ok((client, _)) = others.get(e.0) {
                                    client.send(ChatUpdate::new(
                                        ChatSource::all(game_entity.unique_id),
                                        message.message.clone(),
                                    ));
                                }
                            });
                            client.send(ChatMessageResponse::new(
                                ChatMessageResult::Success,
                                message.target,
                                message.index,
                            ));
                        },
                        ChatTarget::AllGm => {
                            if player.character.gm {
                                others
                                    .iter()
                                    .filter(|(_, player)| player.character.gm)
                                    .filter(|(_, other)| other.user.id != player.user.id)
                                    .for_each(|(client, _)| {
                                        client.send(ChatUpdate::new(
                                            ChatSource::allgm(game_entity.unique_id),
                                            message.message.clone(),
                                        ));
                                    });
                                client.send(ChatMessageResponse::new(
                                    ChatMessageResult::Success,
                                    message.target,
                                    message.index,
                                ));
                            } else {
                                client.send(ChatMessageResponse::new(
                                    ChatMessageResult::error(ChatErrorCode::InvalidTarget),
                                    message.target,
                                    message.index,
                                ));
                            }
                        },
                        ChatTarget::PrivateMessage => {
                            match message
                                .recipient
                                .and_then(|target| lookup.get_entity_for_name(&target))
                                .and_then(|entity| others.get(entity).ok())
                            {
                                Some((other, _)) => {
                                    other.send(ChatUpdate::new(
                                        ChatSource::privatemessage(player.character.name.clone()),
                                        message.message.clone(),
                                    ));
                                    client.send(ChatMessageResponse::new(
                                        ChatMessageResult::Success,
                                        message.target,
                                        message.index,
                                    ));
                                },
                                None => {
                                    client.send(ChatMessageResponse::new(
                                        ChatMessageResult::error(ChatErrorCode::InvalidTarget),
                                        message.target,
                                        message.index,
                                    ));
                                },
                            }
                        },
                        _ => {},
                    }
                },
                _ => {},
            }
        }
    }
}
