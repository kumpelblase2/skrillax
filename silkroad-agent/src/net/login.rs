use crate::comp::net::CharselectInput;
use crate::comp::{CharacterSelect, Client, Login, Playing};
use crate::event::ClientDisconnectedEvent;
use crate::population::LoginQueue;
use crate::population::ReservationError;
use bevy_ecs::prelude::*;
use silkroad_network::stream::{SendResult, Stream};
use silkroad_protocol::auth::{AuthResponse, AuthResult, AuthResultError};
use silkroad_protocol::general::IdentityInformation;
use silkroad_protocol::ClientPacket;
use tracing::{debug, warn};

pub(crate) fn login(
    mut buffer: Commands,
    login_queue: Res<LoginQueue>,
    mut events: EventWriter<ClientDisconnectedEvent>,
    mut query: Query<(Entity, &Client), With<Login>>,
) {
    for (entity, client) in query.iter_mut() {
        if handle_packets(entity, client, &login_queue, &mut buffer).is_err() {
            events.send(ClientDisconnectedEvent(entity));
        }
    }
}

fn handle_packets(entity: Entity, client: &Client, login_queue: &Res<LoginQueue>, cmd: &mut Commands) -> SendResult {
    while let Ok(Some(packet)) = client.0.received() {
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
                        .insert(CharselectInput::default())
                        .insert(CharacterSelect::default());
                    send_login_result(&client.0, AuthResult::success())?;
                    break;
                },
                Err(err) => match err {
                    ReservationError::NoSuchToken | ReservationError::AlreadyHasReservation => {
                        send_login_result(&client.0, AuthResult::error(AuthResultError::InvalidData))?;
                    },
                    ReservationError::NoSpotsAvailable | ReservationError::AllTokensTaken => {
                        send_login_result(&client.0, AuthResult::error(AuthResultError::ServerFull))?;
                    },
                },
            },
            _ => {
                warn!(id = ?client.0.id(), "Client sent packet that didn't fit in login phase.")
            },
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
