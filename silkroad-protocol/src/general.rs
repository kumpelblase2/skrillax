use silkroad_serde::*;
use silkroad_serde_derive::*;

#[derive(Clone, Serialize, ByteSize)]
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

#[derive(Clone, Serialize, ByteSize, Deserialize)]
pub struct IdentityInformation {
    pub module_name: String,
    pub locality: u8,
}

impl IdentityInformation {
    pub fn new(module_name: String, locality: u8) -> Self {
        IdentityInformation { module_name, locality }
    }
}

#[derive(Clone, Deserialize, ByteSize)]
pub struct KeepAlive;

#[derive(Clone, Serialize, ByteSize)]
pub struct SecuritySetup {
    pub stage: HandshakeStage,
}

impl SecuritySetup {
    pub fn new(stage: HandshakeStage) -> Self {
        SecuritySetup { stage }
    }
}

#[derive(Clone, Deserialize, ByteSize)]
pub struct HandshakeChallenge {
    pub b: u32,
    pub key: u64,
}

#[derive(Clone, Deserialize, ByteSize)]
pub struct HandshakeAccepted;
