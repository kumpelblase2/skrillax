use skrillax_packet::Packet;
use skrillax_protocol::define_protocol;
use skrillax_serde::*;

#[derive(Clone, Serialize, ByteSize, Deserialize, Packet, Debug)]
#[packet(opcode = 0x2001)]
pub struct IdentityInformation {
    pub module_name: String,
    pub locality: u8,
}

impl IdentityInformation {
    pub fn new(module_name: String, locality: u8) -> Self {
        IdentityInformation { module_name, locality }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, ByteSize, Packet, Debug)]
#[packet(opcode = 0x2002)]
pub struct KeepAlive;

#[derive(Clone, Copy, Serialize, ByteSize, Deserialize, Debug)]
pub enum HandshakeStage {
    #[silkroad(value = 0xE)]
    Initialize {
        blowfish_seed: u64,
        seed_count: u32,
        seed_crc: u32,
        handshake_seed: u64,
        a: u32,
        b: u32,
        c: u32,
    },
    #[silkroad(value = 0x10)]
    Finalize { challenge: u64 },
}

impl HandshakeStage {
    pub fn initialize(
        blowfish_seed: u64,
        seed_count: u32,
        seed_crc: u32,
        handshake_seed: u64,
        a: u32,
        b: u32,
        c: u32,
    ) -> Self {
        HandshakeStage::Initialize {
            blowfish_seed,
            seed_count,
            seed_crc,
            handshake_seed,
            a,
            b,
            c,
        }
    }

    pub fn finalize(challenge: u64) -> Self {
        HandshakeStage::Finalize { challenge }
    }
}

#[derive(Clone, Copy, Serialize, ByteSize, Deserialize, Packet, Debug)]
#[packet(opcode = 0x5000)]
pub struct SecuritySetup {
    pub stage: HandshakeStage,
}

impl SecuritySetup {
    pub fn new(stage: HandshakeStage) -> Self {
        SecuritySetup { stage }
    }
}

#[derive(Clone, Copy, Deserialize, ByteSize, Serialize, Packet, Debug)]
#[packet(opcode = 0x5000)]
pub struct HandshakeChallenge {
    pub b: u32,
    pub key: u64,
}

#[derive(Clone, Copy, Deserialize, ByteSize, Serialize, Packet, Debug)]
#[packet(opcode = 0x9000)]
pub struct HandshakeAccepted;

define_protocol! { BaseProtocol =>
    IdentityInformation,
    KeepAlive
}

define_protocol! { HandshakeProtocol =>
    SecuritySetup,
    HandshakeChallenge,
    HandshakeAccepted
}
