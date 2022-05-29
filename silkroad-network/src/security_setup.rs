use crate::stream::{StreamError, StreamReader, StreamWriter};
use silkroad_protocol::general::{HandshakeStage, SecuritySetup};
use silkroad_protocol::{ClientPacket, ServerPacket};
use silkroad_security::security::SilkroadSecurity;
use std::sync::{Arc, RwLock};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HandshakeError {
    #[error("Received a packet that not meant for the handshake")]
    NonHandshakePacketReceived,
    #[error("A packet was received before the handshake was accepted")]
    HandshakeNotAccepted,
    #[error("Stream error occurred while performing the handshake")]
    StreamError(#[from] StreamError),
}

pub(crate) struct SecurityHandshake;

impl SecurityHandshake {
    pub(crate) async fn do_handshake(
        writer: &mut StreamWriter,
        reader: &mut StreamReader,
        security: Option<Arc<RwLock<SilkroadSecurity>>>,
    ) -> Result<(), HandshakeError> {
        let security = security.expect("Security should be passed otherwise we shouldn't get run");
        let init = {
            let mut security = security
                .write()
                .expect("We should still hold the lock on security completely.");
            security
                .initialize()
                .expect("We should not be initialized yet.")
        };

        let handshake = HandshakeStage::Initialize {
            blowfish_seed: init.seed,
            seed_count: init.count_seed,
            seed_crc: init.crc_seed,
            handshake_seed: init.handshake_seed,
            a: init.additional_seeds[0],
            b: init.additional_seeds[1],
            c: init.additional_seeds[2],
        };
        writer
            .send(ServerPacket::SecuritySetup(SecuritySetup::new(handshake)))
            .await?;

        let response = reader.next().await?;
        let challenge = match response {
            ClientPacket::HandshakeChallenge(challenge) => {
                let mut security = security
                    .write()
                    .expect("Should still hold lock on security");
                security
                    .start_challenge(challenge.b, challenge.key)
                    .expect("We initialized security just before, cannot still be uninitialized")
            }
            _ => return Err(HandshakeError::NonHandshakePacketReceived),
        };

        writer
            .send(ServerPacket::SecuritySetup(SecuritySetup::new(
                HandshakeStage::Finalize { challenge },
            )))
            .await?;

        let response = reader.next().await?;
        match response {
            ClientPacket::HandshakeAccepted(_) => {
                let mut security = security
                    .write()
                    .expect("Should still hold lock on security");
                security.accept_challenge().unwrap();
            }
            _ => return Err(HandshakeError::HandshakeNotAccepted),
        }
        Ok(())
    }
}
