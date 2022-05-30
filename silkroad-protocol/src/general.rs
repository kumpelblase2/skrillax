#![allow(
    unused_imports,
    unused_variables,
    unused_mut,
    clippy::too_many_arguments,
    clippy::new_without_default
)]

use crate::error::ProtocolError;
use crate::ClientPacket;
use crate::ServerPacket;
use byteorder::ReadBytesExt;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use chrono::{DateTime, Datelike, Timelike, Utc};

#[derive(Clone)]
pub enum HandshakeStage {
    Initialize {
        blowfish_seed: u64,
        seed_count: u32,
        seed_crc: u32,
        handshake_seed: u64,
        a: u32,
        b: u32,
        c: u32,
    },
    Finalize {
        challenge: u64,
    },
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

#[derive(Clone)]
pub struct IdentityInformation {
    pub module_name: String,
    pub locality: u8,
}

impl TryFrom<Bytes> for IdentityInformation {
    type Error = ProtocolError;

    fn try_from(data: Bytes) -> Result<Self, Self::Error> {
        let mut data_reader = data.reader();
        let module_name_string_len = data_reader.read_u16::<byteorder::LittleEndian>()?;
        let mut module_name_bytes = Vec::with_capacity(module_name_string_len as usize);
        for _ in 0..module_name_string_len {
            module_name_bytes.push(data_reader.read_u8()?);
        }
        let module_name = String::from_utf8(module_name_bytes)?;
        let locality = data_reader.read_u8()?;
        Ok(IdentityInformation { module_name, locality })
    }
}

impl From<IdentityInformation> for ClientPacket {
    fn from(other: IdentityInformation) -> Self {
        ClientPacket::IdentityInformation(other)
    }
}

impl From<IdentityInformation> for Bytes {
    fn from(op: IdentityInformation) -> Bytes {
        let mut data_writer = BytesMut::new();
        data_writer.put_u16_le(op.module_name.len() as u16);
        data_writer.put_slice(op.module_name.as_bytes());
        data_writer.put_u8(op.locality);
        data_writer.freeze()
    }
}

impl From<IdentityInformation> for ServerPacket {
    fn from(other: IdentityInformation) -> Self {
        ServerPacket::IdentityInformation(other)
    }
}

impl IdentityInformation {
    pub fn new(module_name: String, locality: u8) -> Self {
        IdentityInformation { module_name, locality }
    }
}

#[derive(Clone)]
pub struct KeepAlive;

impl TryFrom<Bytes> for KeepAlive {
    type Error = ProtocolError;

    fn try_from(data: Bytes) -> Result<Self, Self::Error> {
        let mut data_reader = data.reader();
        Ok(KeepAlive {})
    }
}

impl From<KeepAlive> for ClientPacket {
    fn from(other: KeepAlive) -> Self {
        ClientPacket::KeepAlive(other)
    }
}

#[derive(Clone)]
pub struct ServerInfoSeed {
    pub unknown_1: u8,
    pub unknown_2: u8,
    pub unknown_3: u8,
    pub seed_value: u16,
    pub unknown_4: u32,
    pub unknown_5: u8,
}

impl From<ServerInfoSeed> for Bytes {
    fn from(op: ServerInfoSeed) -> Bytes {
        let mut data_writer = BytesMut::new();
        data_writer.put_u8(op.unknown_1);
        data_writer.put_u8(op.unknown_2);
        data_writer.put_u8(op.unknown_3);
        data_writer.put_u16_le(op.seed_value);
        data_writer.put_u32_le(op.unknown_4);
        data_writer.put_u8(op.unknown_5);
        data_writer.freeze()
    }
}

impl From<ServerInfoSeed> for ServerPacket {
    fn from(other: ServerInfoSeed) -> Self {
        ServerPacket::ServerInfoSeed(other)
    }
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

#[derive(Clone)]
pub struct ServerStateSeed {
    pub unknown_1: u8,
    pub unknown_2: u8,
    pub unknown_3: u8,
    pub unknown_4: u8,
    pub unknown_5: u8,
}

impl From<ServerStateSeed> for Bytes {
    fn from(op: ServerStateSeed) -> Bytes {
        let mut data_writer = BytesMut::new();
        data_writer.put_u8(op.unknown_1);
        data_writer.put_u8(op.unknown_2);
        data_writer.put_u8(op.unknown_3);
        data_writer.put_u8(op.unknown_4);
        data_writer.put_u8(op.unknown_5);
        data_writer.freeze()
    }
}

impl From<ServerStateSeed> for ServerPacket {
    fn from(other: ServerStateSeed) -> Self {
        ServerPacket::ServerStateSeed(other)
    }
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

#[derive(Clone)]
pub struct SecuritySetup {
    pub stage: HandshakeStage,
}

impl From<SecuritySetup> for Bytes {
    fn from(op: SecuritySetup) -> Bytes {
        let mut data_writer = BytesMut::new();
        match &op.stage {
            HandshakeStage::Initialize {
                blowfish_seed,
                seed_count,
                seed_crc,
                handshake_seed,
                a,
                b,
                c,
            } => {
                data_writer.put_u8(0xE);
                data_writer.put_u64_le(*blowfish_seed);
                data_writer.put_u32_le(*seed_count);
                data_writer.put_u32_le(*seed_crc);
                data_writer.put_u64_le(*handshake_seed);
                data_writer.put_u32_le(*a);
                data_writer.put_u32_le(*b);
                data_writer.put_u32_le(*c);
            },
            HandshakeStage::Finalize { challenge } => {
                data_writer.put_u8(0x10);
                data_writer.put_u64_le(*challenge);
            },
        }
        data_writer.freeze()
    }
}

impl From<SecuritySetup> for ServerPacket {
    fn from(other: SecuritySetup) -> Self {
        ServerPacket::SecuritySetup(other)
    }
}

impl SecuritySetup {
    pub fn new(stage: HandshakeStage) -> Self {
        SecuritySetup { stage }
    }
}

#[derive(Clone)]
pub struct HandshakeChallenge {
    pub b: u32,
    pub key: u64,
}

impl TryFrom<Bytes> for HandshakeChallenge {
    type Error = ProtocolError;

    fn try_from(data: Bytes) -> Result<Self, Self::Error> {
        let mut data_reader = data.reader();
        let b = data_reader.read_u32::<byteorder::LittleEndian>()?;
        let key = data_reader.read_u64::<byteorder::LittleEndian>()?;
        Ok(HandshakeChallenge { b, key })
    }
}

impl From<HandshakeChallenge> for ClientPacket {
    fn from(other: HandshakeChallenge) -> Self {
        ClientPacket::HandshakeChallenge(other)
    }
}

#[derive(Clone)]
pub struct HandshakeAccepted;

impl TryFrom<Bytes> for HandshakeAccepted {
    type Error = ProtocolError;

    fn try_from(data: Bytes) -> Result<Self, Self::Error> {
        let mut data_reader = data.reader();
        Ok(HandshakeAccepted {})
    }
}

impl From<HandshakeAccepted> for ClientPacket {
    fn from(other: HandshakeAccepted) -> Self {
        ClientPacket::HandshakeAccepted(other)
    }
}
