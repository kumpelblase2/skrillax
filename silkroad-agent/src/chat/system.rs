use crate::cmd::{CommandExecutionExt, Sender};
use crate::comp::damage::Invincible;
use crate::comp::monster::SpawnedBy;
use crate::comp::net::Client;
use crate::comp::player::Player;
use crate::comp::pos::Position;
use crate::comp::visibility::{Invisible, Visibility};
use crate::comp::GameEntity;
use crate::event::SpawnMonster;
use crate::game::drop::SpawnDrop;
use crate::input::PlayerInputEvent;
use crate::world::{EntityLookup, WorldData};
use bevy::prelude::*;
use silkroad_definitions::type_id::{ObjectConsumable, ObjectConsumableCurrency, ObjectItem, ObjectType};
use silkroad_game_base::{Item, ItemTypeData};
use silkroad_protocol::chat::{
    ChatErrorCode, ChatMessage, ChatMessageResponse, ChatMessageResult, ChatSource, ChatTarget, ChatUpdate,
};
use silkroad_protocol::gm::{GmCommand, GmResponse};
use tracing::debug;

fn can_send_message(message: &ChatMessage, player: &Player) -> bool {
    match message.target {
        ChatTarget::AllGm => player.character.gm,
        ChatTarget::NPC | ChatTarget::Notice | ChatTarget::Global => false,
        _ => true,
    }
}

pub(crate) fn handle_chat(
    mut query: Query<(Entity, &Client, &GameEntity, &Visibility, &Player)>,
    lookup: Res<EntityLookup>,
    others: Query<(&Client, &Player)>,
    mut cmds: Commands,
    mut reader: MessageReader<PlayerInputEvent<ChatMessage>>,
) {
    for event in reader.read() {
        let Ok((entity, client, game_entity, visibility, player)) = query.get_mut(event.player) else {
            continue;
        };

        let message = &event.input;

        debug!(identifier = ?client.id(), "Received chat message: {} @ {}", message.message, message.index);
        if !can_send_message(message, player) {
            client.send(ChatMessageResponse::new(
                ChatMessageResult::error(ChatErrorCode::InvalidTarget),
                message.target,
                message.index,
            ));
            continue;
        }

        match message.target {
            ChatTarget::All => {
                visibility
                    .entities_in_radius
                    .iter()
                    .filter_map(|entity| others.get(entity.0).ok())
                    .for_each(|(client, _)| {
                        client.send(ChatUpdate::new(
                            ChatSource::all(game_entity.unique_id),
                            message.message.clone(),
                        ));
                    });
                client.send(ChatMessageResponse::new(
                    ChatMessageResult::Success,
                    message.target,
                    message.index,
                ));
            },
            ChatTarget::AllGm => {
                if message.message.starts_with('.') {
                    let message_without_dot = message.message.trim_start_matches('.');
                    cmds.enqueue_chat_command(Sender::Player(entity), message_without_dot.to_string());
                    continue;
                }

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
            },
            ChatTarget::PrivateMessage => {
                match message
                    .recipient
                    .as_ref()
                    .and_then(|target| lookup.get_entity_for_name(target))
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
    }
}

pub(crate) fn handle_gm_commands(
    mut query: Query<(Entity, &Client, &Position)>,
    mut commands: Commands,
    mut item_spawn: MessageWriter<SpawnDrop>,
    mut monster_spawn: MessageWriter<SpawnMonster>,
    mut reader: MessageReader<PlayerInputEvent<GmCommand>>,
) {
    for event in reader.read() {
        let Ok((entity, client, position)) = query.get_mut(event.player) else {
            continue;
        };

        match &event.input {
            GmCommand::SpawnMonster { ref_id, amount, .. } => {
                // TODO: for some reason `rarity` is always 1
                for _ in 0..(*amount) {
                    monster_spawn.write(SpawnMonster {
                        ref_id: *ref_id,
                        location: position.location(),
                        spawner: Some(SpawnedBy::Player(entity)),
                        with_ai: true,
                    });
                }
                client.send(GmResponse::success_message(format!(
                    "Spawned {} of {}",
                    *amount, ref_id
                )));
            },
            GmCommand::MakeItem { ref_id, upgrade } => {
                let item = WorldData::items().find_id(*ref_id).unwrap();
                let object_type = ObjectType::from_type_id(&item.common.type_id).unwrap();
                let item_type = if matches!(object_type, ObjectType::Item(ObjectItem::Equippable(_))) {
                    ItemTypeData::Equipment {
                        upgrade_level: *upgrade,
                    }
                } else if matches!(
                    object_type,
                    ObjectType::Item(ObjectItem::Consumable(ObjectConsumable::Currency(
                        ObjectConsumableCurrency::Gold
                    )))
                ) {
                    ItemTypeData::Gold { amount: 1 }
                } else {
                    ItemTypeData::Consumable { amount: 1 }
                };
                item_spawn.write(SpawnDrop::new(
                    Item {
                        reference: item,
                        variance: None,
                        type_data: item_type,
                    },
                    position.location(),
                    None,
                ));
                client.send(GmResponse::success_message(format!("Dropped 1 of {}", item.common.id)));
            },
            GmCommand::Invincible => {
                commands.entity(entity).try_insert(Invincible::from_command());
                client.send(GmResponse::success_message("Enabled invincibility".to_string()));
            },
            GmCommand::Invisible => {
                commands.entity(entity).try_insert(Invisible::from_command());
                client.send(GmResponse::success_message("Enabled invisibility".to_string()));
            },
            _ => {},
        }
    }
}
