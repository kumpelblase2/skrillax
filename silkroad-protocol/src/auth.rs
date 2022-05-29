#![allow(
    unused_imports,
    unused_variables,
    unused_mut,
    clippy::too_many_arguments,
    clippy::new_without_default
)]
use bytes::{Buf, Bytes, BytesMut, BufMut};
use chrono::{DateTime, Datelike, Timelike, Utc};
use byteorder::ReadBytesExt;
use crate::ClientPacket;
use crate::ServerPacket;
use crate::error::ProtocolError;

#[derive(Clone, PartialEq, PartialOrd)]
pub enum LogoutMode {
    Logout,
    Restart,
}

#[derive(Clone)]
pub enum LogoutResult {
    Success {
        seconds_to_logout: u32,
        mode: LogoutMode,
    }
    ,
    Error {
        error: u16,
    }
    ,
}

#[derive(Clone, PartialEq, PartialOrd)]
pub enum AuthResultError {
    InvalidData,
    NotInService,
    ServerFull,
    IpLimit,
}

#[derive(Clone)]
pub enum AuthResult {
    Success {
        unknown_1: u8,
        unknown_2: u8,
    }
    ,
    Error {
        code: AuthResultError,
    }
    ,
}

#[derive(Clone)]
pub struct AuthRequest {
    pub token: u32,
    pub username: String,
    pub password: String,
    pub unknown: u8,
    pub mac_bytes: Bytes,
}

impl TryFrom<Bytes> for AuthRequest {
    type Error = ProtocolError;

    fn try_from(data: Bytes) -> Result<Self, Self::Error> {
        let mut data_reader = data.reader();
        let token = data_reader.read_u32::<byteorder::LittleEndian>()?;
        let username_string_len = data_reader.read_u16::<byteorder::LittleEndian>()?;
        let mut username_bytes = Vec::with_capacity(username_string_len as usize);
        for _ in 0..username_string_len {
            	username_bytes.push(data_reader.read_u8()?);
        }
        let username = String::from_utf8(username_bytes)?;
        let password_string_len = data_reader.read_u16::<byteorder::LittleEndian>()?;
        let mut password_bytes = Vec::with_capacity(password_string_len as usize);
        for _ in 0..password_string_len {
            	password_bytes.push(data_reader.read_u8()?);
        }
        let password = String::from_utf8(password_bytes)?;
        let unknown = data_reader.read_u8()?;
        let mut mac_bytes_raw = BytesMut::with_capacity(6);
        for _ in 0..6 {
            	mac_bytes_raw.put_u8(data_reader.read_u8()?);
        }
        let mac_bytes = mac_bytes_raw.freeze();
        Ok(AuthRequest { token, username, password, unknown, mac_bytes,  })
    }
}

impl Into<ClientPacket> for AuthRequest {
    fn into(self) -> ClientPacket {
        ClientPacket::AuthRequest(self)
    }
}

#[derive(Clone)]
pub struct AuthResponse {
    pub result: AuthResult,
}

impl From<AuthResponse> for Bytes {
    fn from(op: AuthResponse) -> Bytes {
        let mut data_writer = BytesMut::new();
        match &op.result {
            AuthResult::Success { unknown_1, unknown_2,  } => {
                data_writer.put_u8(1);
                data_writer.put_u8(*unknown_1);
                data_writer.put_u8(*unknown_2);
            }
            AuthResult::Error { code,  } => {
                data_writer.put_u8(2);
                match &code {
                    AuthResultError::InvalidData => data_writer.put_u8(2),
                    AuthResultError::NotInService => data_writer.put_u8(3),
                    AuthResultError::ServerFull => data_writer.put_u8(4),
                    AuthResultError::IpLimit => data_writer.put_u8(5),
                }
            }
        }
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for AuthResponse {
    fn into(self) -> ServerPacket {
        ServerPacket::AuthResponse(self)
    }
}

impl AuthResponse {
    pub fn new(result: AuthResult) -> Self {
        AuthResponse { result,  }
    }
}

#[derive(Clone)]
pub struct LogoutRequest {
    pub mode: LogoutMode,
}

impl TryFrom<Bytes> for LogoutRequest {
    type Error = ProtocolError;

    fn try_from(data: Bytes) -> Result<Self, Self::Error> {
        let mut data_reader = data.reader();
        let mode = match data_reader.read_u8()? {
            1 => LogoutMode::Logout,
            2 => LogoutMode::Restart,
            unknown => return Err(ProtocolError::UnknownVariation(unknown, "LogoutMode"))
        };
        Ok(LogoutRequest { mode,  })
    }
}

impl Into<ClientPacket> for LogoutRequest {
    fn into(self) -> ClientPacket {
        ClientPacket::LogoutRequest(self)
    }
}

#[derive(Clone)]
pub struct LogoutResponse {
    pub result: LogoutResult,
}

impl From<LogoutResponse> for Bytes {
    fn from(op: LogoutResponse) -> Bytes {
        let mut data_writer = BytesMut::new();
        match &op.result {
            LogoutResult::Success { seconds_to_logout, mode,  } => {
                data_writer.put_u8(1);
                data_writer.put_u32_le(*seconds_to_logout);
                match &mode {
                    LogoutMode::Logout => data_writer.put_u8(1),
                    LogoutMode::Restart => data_writer.put_u8(2),
                }
            }
            LogoutResult::Error { error,  } => {
                data_writer.put_u8(2);
                data_writer.put_u16_le(*error);
            }
        }
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for LogoutResponse {
    fn into(self) -> ServerPacket {
        ServerPacket::LogoutResponse(self)
    }
}

impl LogoutResponse {
    pub fn new(result: LogoutResult) -> Self {
        LogoutResponse { result,  }
    }
}

#[derive(Clone)]
pub struct LogoutFinished;

impl From<LogoutFinished> for Bytes {
    fn from(op: LogoutFinished) -> Bytes {
        let mut data_writer = BytesMut::new();
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for LogoutFinished {
    fn into(self) -> ServerPacket {
        ServerPacket::LogoutFinished(self)
    }
}

impl LogoutFinished {
    pub fn new() -> Self {
        LogoutFinished {  }
    }
}

#[derive(Clone)]
pub struct Disconnect {
    pub unknown: u8,
}

impl From<Disconnect> for Bytes {
    fn from(op: Disconnect) -> Bytes {
        let mut data_writer = BytesMut::new();
        data_writer.put_u8(op.unknown);
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for Disconnect {
    fn into(self) -> ServerPacket {
        ServerPacket::Disconnect(self)
    }
}

impl Disconnect {
    pub fn new() -> Self {
        Disconnect { unknown: 0xFF,  }
    }
}