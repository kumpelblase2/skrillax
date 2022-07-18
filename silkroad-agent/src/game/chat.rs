use crate::comp::drop::ItemDrop;
use crate::comp::pos::Position;
use crate::comp::visibility::Visibility;
use crate::comp::{Client, GameEntity};
use crate::event::{ChatEvent, PlayerLevelUp};
use crate::world::EntityLookup;
use bevy_app::{App, Plugin};
use bevy_core::Timer;
use bevy_ecs::event::EventReader;
use bevy_ecs::prelude::*;
use cgmath::num_traits::Pow;
use cgmath::MetricSpace;
use id_pool::IdPool;
use silkroad_protocol::chat::{ChatSource, ChatUpdate};
use silkroad_protocol::ServerPacket;

pub(crate) struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ChatEvent>().add_system(chat_update);
    }
}

pub(crate) fn chat_update(
    mut chats: EventReader<ChatEvent>,
    mut levelup: EventWriter<PlayerLevelUp>,
    mut cmd: Commands,
    mut id_pool: ResMut<IdPool>,
    mut lookup: ResMut<EntityLookup>,
    query: Query<(Entity, &Client, &Position, &Visibility)>,
) {
    for chat in chats.iter() {
        match chat {
            ChatEvent::RegionalChat {
                sender,
                sender_unique_id,
                position,
                message,
            } => {
                for (_, client, _, _) in
                    query
                        .iter()
                        .filter(|(target, _, _, _)| target != sender)
                        .filter(|(_, _, pos, visibility)| {
                            let pos: &Position = pos;
                            let visibility: &Visibility = visibility;
                            position.distance2(pos.location.0) <= visibility.visibility_radius.pow(2)
                        })
                {
                    client.send(ServerPacket::ChatUpdate(ChatUpdate::new(
                        ChatSource::all(*sender_unique_id),
                        message.clone(),
                    )));
                }
            },

            ChatEvent::PrivateChat {
                sender,
                target,
                message,
            } => {
                if let Some(target) = lookup.get_entity_for_name(target) {
                    if let Ok((_, client, _, _)) = query.get(*target) {
                        client.send(ServerPacket::ChatUpdate(ChatUpdate::new(
                            ChatSource::privatemessage(sender.clone()),
                            message.clone(),
                        )));
                    }
                } else {
                    // TODO
                }
            },

            ChatEvent::Command { sender, message } => {
                let command_str = &message[1..];
                let elements = command_str.split(' ').collect::<Vec<&str>>();
                match elements[0] {
                    "levelup" => {
                        let target: u8 = elements[1].parse().unwrap();
                        levelup.send(PlayerLevelUp(*sender, target));
                    },
                    "drop" => {
                        let amount: u32 = elements[1].parse().unwrap();
                        let (_, _, pos, _) = query.get(*sender).unwrap();
                        let id = id_pool.request_id().unwrap();
                        let drop = cmd
                            .spawn()
                            .insert(ItemDrop {
                                despawn_timer: Timer::from_seconds(10.0, false),
                                owner: None,
                                amount,
                            })
                            .insert(pos.clone())
                            .insert(GameEntity {
                                unique_id: id,
                                ref_id: 1,
                            })
                            .id();

                        lookup.add_entity(id, drop);
                    },
                    _ => {},
                }
            },
        }
    }
}
