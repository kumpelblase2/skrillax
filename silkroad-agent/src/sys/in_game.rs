use crate::comp::monster::Monster;
use crate::comp::player::{Agent, Character, MovementState, MovementTarget, Player, SpawningState};
use crate::comp::pos::{Heading, LocalPosition, Position};
use crate::comp::visibility::Visibility;
use crate::comp::{Client, GameEntity, Health};
use crate::id_allocator::IdAllocator;
use crate::GameSettings;
use bevy_core::{Time, Timer};
use bevy_ecs::prelude::*;
use cgmath::{Deg, Quaternion, Rotation3, Vector3};
use silkroad_protocol::auth::{LogoutFinished, LogoutRequest, LogoutResponse, LogoutResult};
use silkroad_protocol::character::CharacterStatsMessage;
use silkroad_protocol::chat::{
    ChatMessageResponse, ChatMessageResult, ChatSource, ChatUpdate, TextCharacterInitialization,
};
use silkroad_protocol::world::{
    CelestialUpdate, EntityRarity, EntityUpdateState, MovementSource, PlayerMovementRequest, PlayerMovementResponse,
    Rotation, TargetEntity, TargetEntityResponse, TargetEntityResult,
};
use silkroad_protocol::{ClientPacket, ServerPacket};
use std::ops::Add;
use std::time::Duration;
use tracing::debug;

pub(crate) fn in_game(
    settings: Res<GameSettings>,
    delta: Res<Time>,
    mut allocator: ResMut<IdAllocator>,
    mut cmd: Commands,
    mut query: Query<(Entity, &mut Client, &GameEntity, &mut Player, &mut Agent, &Position)>,
) {
    for (entity, mut client, game_entity, mut player, mut agent, position) in query.iter_mut() {
        let mut agent: &mut Agent = &mut agent;
        let game_entity: &GameEntity = game_entity;

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

                    cmd.spawn()
                        .insert(Agent::new(32.))
                        .insert(position.clone())
                        .insert(Monster {
                            rarity: EntityRarity::Normal,
                            target: None,
                        })
                        .insert(GameEntity {
                            ref_id: 0x078d,
                            unique_id: allocator.allocate(),
                        })
                        .insert(Health::new(100))
                        .insert(Visibility::with_radius(10.));
                },
                ClientPacket::PlayerMovementRequest(PlayerMovementRequest { kind }) => match kind {
                    silkroad_protocol::world::MovementTarget::TargetLocation { region, x, y, z } => {
                        let local_position = position.location.to_local();
                        debug!(id = ?client.0.id(), "Movement: {}|{}|{} @ {} -> {}|{}|{} @ {}", local_position.1.x, local_position.1.y, local_position.1.z, local_position.0, x, y, z, region);
                        let response = ServerPacket::PlayerMovementResponse(PlayerMovementResponse::new(
                            game_entity.unique_id,
                            region,
                            x,
                            y,
                            z,
                            Some(MovementSource::new(
                                local_position.0.id(),
                                (local_position.1.x * 10.) as u16,
                                local_position.1.y,
                                (local_position.1.z * 10.) as u16,
                            )),
                        ));
                        client.send(response);
                        let target_pos = LocalPosition(region.into(), Vector3::new(x as f32, y as f32, z as f32));
                        agent.movement_target = Some(MovementTarget::Location(target_pos.to_global()));
                        agent.movement_state = MovementState::Moving;
                    },
                    silkroad_protocol::world::MovementTarget::Direction { unknown, angle } => {
                        debug!(id = ?client.0.id(), "Movement: {} / {} degrees", unknown, angle);
                        agent.movement_target =
                            Some(MovementTarget::Direction(Quaternion::from_angle_y(Deg(angle as f32))));
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
                        TargetEntityResult::success(unique_id),
                    )));
                },
                _ => {},
            }
        }

        if let Some(logout_time) = player.logout {
            if delta.last_update().unwrap() > logout_time {
                client.send(LogoutFinished);
                cmd.entity(entity).despawn();
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
