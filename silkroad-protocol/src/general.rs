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
pub struct ServerInfoSeed {
    pub unknown_1: u8,
    pub unknown_2: u8,
    pub unknown_3: u8,
    pub seed_value: u16,
    pub unknown_4: u32,
    pub unknown_5: u8,
}

impl ServerInfoSeed {
    pub fn new(seed_value: u16) -> Self {
        ServerInfoSeed {
            unknown_1: 1,
            unknown_2: 0,
            unknown_3: 1,
            seed_value,
            unknown_4: 5,
            unknown_5: 2,
        }
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub struct ServerStateSeed {
    pub unknown_1: u8,
    pub unknown_2: u8,
    pub unknown_3: u8,
    pub unknown_4: u8,
    pub unknown_5: u8,
}

impl ServerStateSeed {
    pub fn new() -> Self {
        ServerStateSeed {
            unknown_1: 3,
            unknown_2: 0,
            unknown_3: 2,
            unknown_4: 0,
            unknown_5: 2,
        }
    }
}

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
