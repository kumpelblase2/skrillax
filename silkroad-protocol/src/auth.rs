// This is generated code. Do not modify manually.
#![allow(
    unused_imports,
    unused_variables,
    unused_mut,
    clippy::too_many_arguments,
    clippy::new_without_default
)]

use crate::error::ProtocolError;
use crate::size::Size;
use crate::ClientPacket;
use crate::ServerPacket;
use byteorder::ReadBytesExt;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use chrono::{DateTime, Datelike, Timelike, Utc};

#[derive(Clone, PartialEq, PartialOrd, Copy)]
pub enum LogoutMode {
    Logout,
    Restart,
}

impl Size for LogoutMode {
    fn calculate_size(&self) -> usize {
        std::mem::size_of::<u8>()
    }
}

#[derive(Clone)]
pub enum LogoutResult {
    Success { seconds_to_logout: u32, mode: LogoutMode },
    Error { error: u16 },
}

impl LogoutResult {
    pub fn success(seconds_to_logout: u32, mode: LogoutMode) -> Self {
        LogoutResult::Success {
            seconds_to_logout,
            mode,
        }
    }

    pub fn error(error: u16) -> Self {
        LogoutResult::Error { error }
    }
}

impl Size for LogoutResult {
    fn calculate_size(&self) -> usize {
        std::mem::size_of::<u8>()
            + match &self {
                LogoutResult::Success {
                    seconds_to_logout,
                    mode,
                } => seconds_to_logout.calculate_size() + mode.calculate_size(),
                LogoutResult::Error { error } => error.calculate_size(),
            }
    }
}

#[derive(Clone, PartialEq, PartialOrd, Copy)]
pub enum AuthResultError {
    InvalidData,
    NotInService,
    ServerFull,
    IpLimit,
}

impl Size for AuthResultError {
    fn calculate_size(&self) -> usize {
        std::mem::size_of::<u8>()
    }
}

#[derive(Clone)]
pub enum AuthResult {
    Success { unknown_1: u8, unknown_2: u8 },
    Error { code: AuthResultError },
}

impl AuthResult {
    pub fn success() -> Self {
        AuthResult::Success {
            unknown_1: 3,
            unknown_2: 0,
        }
    }

    pub fn error(code: AuthResultError) -> Self {
        AuthResult::Error { code }
    }
}

impl Size for AuthResult {
    fn calculate_size(&self) -> usize {
        std::mem::size_of::<u8>()
            + match &self {
                AuthResult::Success { unknown_1, unknown_2 } => unknown_1.calculate_size() + unknown_2.calculate_size(),
                AuthResult::Error { code } => code.calculate_size(),
            }
    }
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
        Ok(AuthRequest {
            token,
            username,
            password,
            unknown,
            mac_bytes,
        })
    }
}

impl From<AuthRequest> for ClientPacket {
    fn from(other: AuthRequest) -> Self {
        ClientPacket::AuthRequest(other)
    }
}

#[derive(Clone)]
pub struct AuthResponse {
    pub result: AuthResult,
}

impl From<AuthResponse> for Bytes {
    fn from(op: AuthResponse) -> Bytes {
        let mut data_writer = BytesMut::with_capacity(op.calculate_size());
        match &op.result {
            AuthResult::Success { unknown_1, unknown_2 } => {
                data_writer.put_u8(1);
                data_writer.put_u8(*unknown_1);
                data_writer.put_u8(*unknown_2);
            },
            AuthResult::Error { code } => {
                data_writer.put_u8(2);
                match &code {
                    AuthResultError::InvalidData => data_writer.put_u8(2),
                    AuthResultError::NotInService => data_writer.put_u8(3),
                    AuthResultError::ServerFull => data_writer.put_u8(4),
                    AuthResultError::IpLimit => data_writer.put_u8(5),
                }
            },
        }
        data_writer.freeze()
    }
}

impl From<AuthResponse> for ServerPacket {
    fn from(other: AuthResponse) -> Self {
        ServerPacket::AuthResponse(other)
    }
}

impl AuthResponse {
    pub fn new(result: AuthResult) -> Self {
        AuthResponse { result }
    }
}

impl Size for AuthResponse {
    fn calculate_size(&self) -> usize {
        self.result.calculate_size()
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
            unknown => return Err(ProtocolError::UnknownVariation(unknown, "LogoutMode")),
        };
        Ok(LogoutRequest { mode })
    }
}

impl From<LogoutRequest> for ClientPacket {
    fn from(other: LogoutRequest) -> Self {
        ClientPacket::LogoutRequest(other)
    }
}

#[derive(Clone)]
pub struct LogoutResponse {
    pub result: LogoutResult,
}

impl From<LogoutResponse> for Bytes {
    fn from(op: LogoutResponse) -> Bytes {
        let mut data_writer = BytesMut::with_capacity(op.calculate_size());
        match &op.result {
            LogoutResult::Success {
                seconds_to_logout,
                mode,
            } => {
                data_writer.put_u8(1);
                data_writer.put_u32_le(*seconds_to_logout);
                match &mode {
                    LogoutMode::Logout => data_writer.put_u8(1),
                    LogoutMode::Restart => data_writer.put_u8(2),
                }
            },
            LogoutResult::Error { error } => {
                data_writer.put_u8(2);
                data_writer.put_u16_le(*error);
            },
        }
        data_writer.freeze()
    }
}

impl From<LogoutResponse> for ServerPacket {
    fn from(other: LogoutResponse) -> Self {
        ServerPacket::LogoutResponse(other)
    }
}

impl LogoutResponse {
    pub fn new(result: LogoutResult) -> Self {
        LogoutResponse { result }
    }
}

impl Size for LogoutResponse {
    fn calculate_size(&self) -> usize {
        self.result.calculate_size()
    }
}

#[derive(Clone)]
pub struct LogoutFinished;

impl From<LogoutFinished> for Bytes {
    fn from(op: LogoutFinished) -> Bytes {
        let mut data_writer = BytesMut::with_capacity(op.calculate_size());
        data_writer.freeze()
    }
}

impl From<LogoutFinished> for ServerPacket {
    fn from(other: LogoutFinished) -> Self {
        ServerPacket::LogoutFinished(other)
    }
}

impl LogoutFinished {
    pub fn new() -> Self {
        LogoutFinished {}
    }
}

impl Size for LogoutFinished {
    fn calculate_size(&self) -> usize {
        0
    }
}

#[derive(Clone)]
pub struct Disconnect {
    pub unknown: u8,
}

impl From<Disconnect> for Bytes {
    fn from(op: Disconnect) -> Bytes {
        let mut data_writer = BytesMut::with_capacity(op.calculate_size());
        data_writer.put_u8(op.unknown);
        data_writer.freeze()
    }
}

impl From<Disconnect> for ServerPacket {
    fn from(other: Disconnect) -> Self {
        ServerPacket::Disconnect(other)
    }
}

impl Disconnect {
    pub fn new() -> Self {
        Disconnect { unknown: 0xFF }
    }
}

impl Size for Disconnect {
    fn calculate_size(&self) -> usize {
        self.unknown.calculate_size()
    }
}
