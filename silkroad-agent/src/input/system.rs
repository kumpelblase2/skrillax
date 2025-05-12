use crate::comp::net::{Client, LastAction};
use crate::config::GameConfig;
use crate::event::ClientDisconnectedEvent;
use crate::input::events::PlayerInputEvent;
use bevy::prelude::*;
use silkroad_protocol::auth::{AuthRequest, LogoutRequest};
use silkroad_protocol::character::{CharacterJoinRequest, CharacterListRequest, FinishLoading};
use silkroad_protocol::chat::ChatMessage;
use silkroad_protocol::combat::PerformAction;
use silkroad_protocol::gm::GmCommand;
use silkroad_protocol::inventory::{ConsignmentList, InventoryOperation, OpenItemMall};
use silkroad_protocol::movement::{PlayerMovementRequest, Rotation};
use silkroad_protocol::skill::{HotbarUpdate, LearnSkill, LevelUpMastery};
use silkroad_protocol::world::{
    GameGuideResponse, IncreaseInt, IncreaseStr, TargetEntity, UnTargetEntity, UpdateGameGuide,
};
use silkroad_protocol::{IdentityInformation, KeepAlive};
use skrillax_packet::Packet;
use skrillax_stream::stream::DynamicPacket;
use std::time::Instant;

macro_rules! handle_packets {
    ($packet:expr, $cmd:expr, $entity:expr, [$( $ty:ty ),*]) => {{
        match $packet.opcode() {
            $(
                <$ty>::ID => {
                    $cmd.write_message(crate::input::events::PlayerInputEvent::new(
                        $entity,
                        $packet.into_packet::<$ty>()?
                    ));
                }
            )*
            unhandled => {
                println!("Unhandled packet opcode: {unhandled}");
            }
        }
    }};
}

fn dispatch_events(packet: DynamicPacket, cmd: &mut Commands, entity: Entity) -> Result<(), DynamicPacket> {
    if packet.opcode() == KeepAlive::ID {
        return Ok(());
    }

    handle_packets!(
        packet,
        cmd,
        entity,
        [
            ChatMessage,
            IncreaseInt,
            IncreaseStr,
            FinishLoading,
            GmCommand,
            TargetEntity,
            PerformAction,
            UnTargetEntity,
            UpdateGameGuide,
            CharacterListRequest,
            CharacterJoinRequest,
            AuthRequest,
            LogoutRequest,
            GameGuideResponse,
            HotbarUpdate,
            LearnSkill,
            LevelUpMastery,
            InventoryOperation,
            OpenItemMall,
            ConsignmentList,
            PlayerMovementRequest,
            IdentityInformation,
            Rotation
        ]
    );
    Ok(())
}

pub(crate) fn handle_packet_inputs(
    mut query: Query<(Entity, &Client, &mut LastAction)>,
    mut cmd: Commands,
    settings: Res<GameConfig>,
    time: Res<Time<Real>>,
    mut disconnect_events: MessageWriter<ClientDisconnectedEvent>,
) {
    for (entity, client, mut last_action) in query.iter_mut() {
        let mut had_action = false;
        loop {
            match client.next() {
                Ok(Some(packet)) => {
                    had_action = true;

                    if let Err(invalid) = dispatch_events(packet, &mut cmd, entity) {
                        println!("Packet had opcode different from type: {:?}", invalid.opcode());
                    }
                },
                Ok(None) => {
                    break;
                },
                Err(_) => {
                    disconnect_events.write(ClientDisconnectedEvent(entity));
                    break;
                },
            }
        }

        let last_tick_time = time.last_update().unwrap_or_else(Instant::now);
        if had_action {
            last_action.0 = last_tick_time;
        }

        if last_tick_time.duration_since(last_action.0).as_secs() > settings.client_timeout.into() {
            disconnect_events.write(ClientDisconnectedEvent(entity));
        }
    }
}

fn send_identity_information(client: &Client) {
    client.send(IdentityInformation::new("AgentServer".to_string(), 0))
}

pub(crate) fn handle_identity_information(
    mut reader: MessageReader<PlayerInputEvent<IdentityInformation>>,
    query: Query<&Client>,
) {
    reader.read().for_each(|event| {
        let client = query.get(event.player).unwrap();
        send_identity_information(client);
    });
}
