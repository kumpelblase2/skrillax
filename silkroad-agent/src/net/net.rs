use crate::comp::net::{
    CharselectInput, ChatInput, Client, GmInput, InventoryInput, LastAction, MovementInput, WorldInput,
};
use crate::comp::player::Player;
use crate::comp::Login;
use crate::config::GameConfig;
use crate::db::character::CharacterData;
use crate::event::{ClientConnectedEvent, ClientDisconnectedEvent, LoadingFinishedEvent};
use crate::ext::{DbPool, ServerResource};
use crate::tasks::TaskCreator;
use bevy_ecs::prelude::*;
use bevy_time::Time;
use silkroad_network::stream::StreamError;
use silkroad_protocol::character::{GameGuideResponse, UpdateGameGuide};
use silkroad_protocol::inventory::{ConsignmentResponse, OpenItemMallResponse, OpenItemMallResult};
use silkroad_protocol::ClientPacket;
use tracing::{debug, warn};

pub(crate) fn accept(
    mut events: EventWriter<ClientConnectedEvent>,
    network: Res<ServerResource>,
    time: Res<Time>,
    mut cmd: Commands,
) {
    while let Some(client) = network.connected() {
        debug!(id = ?client.id(), "Accepted client");

        let entity = cmd
            .spawn((Client(client), LastAction(time.last_update().unwrap()), Login))
            .id();

        events.send(ClientConnectedEvent(entity));
    }
}

pub(crate) fn receive(
    mut events: EventWriter<ClientDisconnectedEvent>,
    mut loading_events: EventWriter<LoadingFinishedEvent>,
    settings: Res<GameConfig>,
    time: Res<Time>,
    mut query: Query<
        (
            Entity,
            &Client,
            &mut LastAction,
            Option<&mut CharselectInput>,
            Option<&mut MovementInput>,
            Option<&mut InventoryInput>,
            Option<&mut ChatInput>,
            Option<&mut WorldInput>,
            Option<&mut GmInput>,
        ),
        Without<Login>,
    >,
) {
    'query: for (
        entity,
        client,
        mut last_action,
        mut charselect_opt,
        mut movement_opt,
        mut inventory_opt,
        mut chat_opt,
        mut world_opt,
        mut gm_opt,
    ) in query.iter_mut()
    {
        let mut has_activity = false;
        loop {
            match client.0.received() {
                Ok(Some(packet)) => {
                    has_activity = true;
                    // Already handle keep-alives to not clog other systems
                    match packet {
                        packet @ ClientPacket::ChatMessage(_) => {
                            if let Some(input) = chat_opt.as_mut() {
                                input.inputs.push(packet);
                            }
                        },
                        packet @ ClientPacket::CharacterListRequest(_)
                        | packet @ ClientPacket::CharacterJoinRequest(_) => {
                            if let Some(input) = charselect_opt.as_mut() {
                                input.inputs.push(packet);
                            }
                        },
                        packet @ ClientPacket::Rotation(_) | packet @ ClientPacket::PlayerMovementRequest(_) => {
                            if let Some(input) = movement_opt.as_mut() {
                                input.inputs.push(packet);
                            }
                        },
                        ClientPacket::FinishLoading(_) => {
                            loading_events.send(LoadingFinishedEvent(entity));
                        },
                        packet @ ClientPacket::LogoutRequest(_)
                        | packet @ ClientPacket::TargetEntity(_)
                        | packet @ ClientPacket::UnTargetEntity(_)
                        | packet @ ClientPacket::PerformAction(_) => {
                            if let Some(input) = world_opt.as_mut() {
                                input.inputs.push(packet);
                            }
                        },
                        ClientPacket::GmCommand(command) => {
                            if let Some(input) = gm_opt.as_mut() {
                                input.inputs.push(command);
                            }
                        },
                        ClientPacket::OpenItemMall(_) => {
                            client.send(OpenItemMallResponse(OpenItemMallResult::Success {
                                jid: 123,
                                token: "123".to_string(),
                            }));
                        },
                        packet @ ClientPacket::InventoryOperation(_) => {
                            if let Some(input) = inventory_opt.as_mut() {
                                input.inputs.push(packet);
                            }
                        },
                        ClientPacket::ConsignmentList(_) => {
                            client.send(ConsignmentResponse::success_empty());
                        },
                        ClientPacket::AddFriend(_) => {},
                        ClientPacket::CreateFriendGroup(_) => {},
                        ClientPacket::DeleteFriend(_) => {},
                        ClientPacket::UpdateGameGuide(UpdateGameGuide(val)) => {
                            client.send(GameGuideResponse::Success(val));
                        },
                        _ => {},
                    }
                },
                Ok(None) => break,
                Err(StreamError::StreamClosed) => {
                    events.send(ClientDisconnectedEvent(entity));
                    continue 'query;
                },
                Err(e) => {
                    warn!(id = ?client.0.id(), "Error when receiving. {:?}", e);
                },
            }
        }

        let last_tick_time = time.last_update().unwrap();
        if has_activity {
            last_action.0 = last_tick_time;
        }

        if last_tick_time.duration_since(last_action.0).as_secs() > settings.client_timeout as u64 {
            events.send(ClientDisconnectedEvent(entity));
        }
    }
}

pub(crate) fn disconnected(
    mut events: EventReader<ClientDisconnectedEvent>,
    mut cmd: Commands,
    task_creator: Res<TaskCreator>,
    pool: Res<DbPool>,
    query: Query<&Player>,
) {
    for event in events.iter() {
        let entity = event.0;
        debug!("Handling client disconnect.");
        if let Ok(player) = query.get(event.0) {
            let id = player.character.id;
            task_creator.spawn(CharacterData::update_last_played_of(id, pool.clone()));
        }
        cmd.entity(entity).despawn();
    }
}

pub(crate) fn connected(mut events: EventReader<ClientConnectedEvent>) {
    for _ in events.iter() {
        // ..
    }
}
