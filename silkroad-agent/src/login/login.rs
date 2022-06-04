use crate::comp::{CharacterSelect, Client, Login, Playing};
use crate::event::ClientDisconnectedEvent;
use crate::population::queue::LoginQueue;
use crate::population::queue::ReservationError;
use bevy_ecs::prelude::*;
use silkroad_network::stream::{SendResult, Stream};
use silkroad_protocol::auth::{AuthResponse, AuthResult, AuthResultError};
use silkroad_protocol::general::IdentityInformation;
use silkroad_protocol::ClientPacket;
use tracing::debug;

pub(crate) fn login(
    mut buffer: Commands,
    login_queue: Res<LoginQueue>,
    mut events: EventWriter<ClientDisconnectedEvent>,
    mut query: Query<(Entity, &mut Client), With<Login>>,
) {
    for (entity, mut client) in query.iter_mut() {
        match handle_packets(entity, &mut client, &login_queue, &mut buffer) {
            Err(_) => {
                events.send(ClientDisconnectedEvent(entity));
            },
            _ => {},
        }
    }
}

fn handle_packets(
    entity: Entity,
    client: &mut Client,
    login_queue: &Res<LoginQueue>,
    cmd: &mut Commands,
) -> SendResult {
    while let Some(packet) = client.1.pop_front() {
        match packet {
            ClientPacket::IdentityInformation(_id) => {
                send_identity_information(&client.0)?;
            },
            ClientPacket::AuthRequest(request) => match login_queue.hand_in_reservation(request.token) {
                Ok((token, user)) => {
                    debug!(id = ?client.0.id(), token = request.token, "Accepted token");
                    cmd.entity(entity)
                        .remove::<Login>()
                        .insert(Playing(user, token))
                        .insert(CharacterSelect::default());
                    send_login_result(&client.0, AuthResult::success())?;
                    break;
                },
                Err(ReservationError::NoSuchToken) => {
                    send_login_result(&client.0, AuthResult::error(AuthResultError::InvalidData))?;
                },
                _ => {
                    send_login_result(&client.0, AuthResult::error(AuthResultError::ServerFull))?;
                },
            },
            _ => {},
        }
    }
    Ok(())
}

fn send_identity_information(client: &Stream) -> SendResult {
    client.send(IdentityInformation::new("AgentServer".to_string(), 0))
}

fn send_login_result(client: &Stream, result: AuthResult) -> SendResult {
    client.send(AuthResponse::new(result))
}
