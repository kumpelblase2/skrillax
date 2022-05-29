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
pub enum SecurityCodeAction {
    Define,
    Enter,
    Unknown,
}

#[derive(Clone, PartialEq, PartialOrd)]
pub enum PasscodeRequiredCode {
    DefinePasscode,
    PasscodeRequired,
    PasscodeBlocked,
    PasscodeInvalid,
}

#[derive(Clone)]
pub enum PatchError {
    InvalidVersion,
    Update {
        server_ip: String,
        server_port: u16,
        current_version: u32,
        patch_files: Vec<PatchFile>,
        http_server: String,
    }
    ,
    Offline,
    InvalidClient,
    PatchDisabled,
}

#[derive(Clone)]
pub enum PatchResult {
    UpToDate {
        unknown: u8,
    }
    ,
    Problem {
        error: PatchError,
    }
    ,
}

#[derive(Clone, PartialEq, PartialOrd)]
pub enum PasscodeAccountStatus {
    Ok,
    EmailUnverified,
}

#[derive(Clone)]
pub enum BlockReason {
    AccountInspection,
    Punishment {
        reason: String,
        end: DateTime<Utc>,
    }
    ,
}

#[derive(Clone)]
pub enum SecurityError {
    InvalidCredentials {
        max_attempts: u32,
        current_attempts: u32,
    }
    ,
    Blocked {
        reason: BlockReason,
    }
    ,
    AlreadyConnected,
    Inspection,
    ServerFull,
    LoginQueue {
        total_in_queue: u16,
        expected_wait_time: u32,
    }
    ,
    PasswordExpired,
    IpLimit,
}

#[derive(Clone)]
pub enum LoginResult {
    Success {
        session_id: u32,
        agent_ip: String,
        agent_port: u16,
        unknown: u8,
    }
    ,
    Error {
        error: SecurityError,
    }
    ,
    Unknown,
}

#[derive(Clone)]
pub struct QueueUpdateStatus {
    pub total_in_queue: u16,
    pub expected_wait_time: u32,
    pub current_position: u16,
}

#[derive(Clone)]
pub struct PatchFile {
    pub file_id: u32,
    pub filename: String,
    pub file_path: String,
    pub size: u32,
    pub in_pk2: bool,
}

#[derive(Clone)]
pub struct GatewayNotice {
    pub subject: String,
    pub article: String,
    pub published: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PingServer {
    pub index: u8,
    pub domain: String,
    pub unknown_1: u8,
    pub unknown_2: u8,
}

#[derive(Clone)]
pub struct Shard {
    pub id: u16,
    pub name: String,
    pub status: u8,
    pub is_online: bool,
}

#[derive(Clone)]
pub struct Farm {
    pub id: u8,
    pub name: String,
}

#[derive(Clone)]
pub struct PatchRequest {
    pub content: u8,
    pub module: String,
    pub version: u32,
}

impl TryFrom<Bytes> for PatchRequest {
    type Error = ProtocolError;

    fn try_from(data: Bytes) -> Result<Self, Self::Error> {
        let mut data_reader = data.reader();
        let content = data_reader.read_u8()?;
        let module_string_len = data_reader.read_u16::<byteorder::LittleEndian>()?;
        let mut module_bytes = Vec::with_capacity(module_string_len as usize);
        for _ in 0..module_string_len {
            	module_bytes.push(data_reader.read_u8()?);
        }
        let module = String::from_utf8(module_bytes)?;
        let version = data_reader.read_u32::<byteorder::LittleEndian>()?;
        Ok(PatchRequest { content, module, version,  })
    }
}

impl Into<ClientPacket> for PatchRequest {
    fn into(self) -> ClientPacket {
        ClientPacket::PatchRequest(self)
    }
}

#[derive(Clone)]
pub struct PatchResponse {
    pub result: PatchResult,
}

impl From<PatchResponse> for Bytes {
    fn from(op: PatchResponse) -> Bytes {
        let mut data_writer = BytesMut::new();
        match &op.result {
            PatchResult::UpToDate { unknown,  } => {
                data_writer.put_u8(1);
                data_writer.put_u8(*unknown);
            }
            PatchResult::Problem { error,  } => {
                data_writer.put_u8(2);
                match &error {
                    PatchError::InvalidVersion => data_writer.put_u8(1),
                    PatchError::Update { server_ip, server_port, current_version, patch_files, http_server,  } => {
                        data_writer.put_u8(2);
                        data_writer.put_u16_le(server_ip.len() as u16);
                        data_writer.put_slice(server_ip.as_bytes());
                        data_writer.put_u16_le(*server_port);
                        data_writer.put_u32_le(*current_version);
                        for element in patch_files.iter() {
                            data_writer.put_u8(1);
                            data_writer.put_u32_le(element.file_id);
                            data_writer.put_u16_le(element.filename.len() as u16);
                            data_writer.put_slice(element.filename.as_bytes());
                            data_writer.put_u16_le(element.file_path.len() as u16);
                            data_writer.put_slice(element.file_path.as_bytes());
                            data_writer.put_u32_le(element.size);
                            data_writer.put_u8(element.in_pk2 as u8);
                        }
                        data_writer.put_u8(0);
                        data_writer.put_u16_le(http_server.len() as u16);
                        data_writer.put_slice(http_server.as_bytes());
                    }
                    PatchError::Offline => data_writer.put_u8(3),
                    PatchError::InvalidClient => data_writer.put_u8(4),
                    PatchError::PatchDisabled => data_writer.put_u8(5),
                }
            }
        }
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for PatchResponse {
    fn into(self) -> ServerPacket {
        ServerPacket::PatchResponse(self)
    }
}

impl PatchResponse {
    pub fn new(result: PatchResult) -> Self {
        PatchResponse { result,  }
    }
}

#[derive(Clone)]
pub struct LoginRequest {
    pub unknown_1: u8,
    pub username: String,
    pub password: String,
    pub shard_id: u16,
    pub unknown_2: u8,
}

impl TryFrom<Bytes> for LoginRequest {
    type Error = ProtocolError;

    fn try_from(data: Bytes) -> Result<Self, Self::Error> {
        let mut data_reader = data.reader();
        let unknown_1 = data_reader.read_u8()?;
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
        let shard_id = data_reader.read_u16::<byteorder::LittleEndian>()?;
        let unknown_2 = data_reader.read_u8()?;
        Ok(LoginRequest { unknown_1, username, password, shard_id, unknown_2,  })
    }
}

impl Into<ClientPacket> for LoginRequest {
    fn into(self) -> ClientPacket {
        ClientPacket::LoginRequest(self)
    }
}

#[derive(Clone)]
pub struct LoginResponse {
    pub result: LoginResult,
}

impl From<LoginResponse> for Bytes {
    fn from(op: LoginResponse) -> Bytes {
        let mut data_writer = BytesMut::new();
        match &op.result {
            LoginResult::Success { session_id, agent_ip, agent_port, unknown,  } => {
                data_writer.put_u8(1);
                data_writer.put_u32_le(*session_id);
                data_writer.put_u16_le(agent_ip.len() as u16);
                data_writer.put_slice(agent_ip.as_bytes());
                data_writer.put_u16_le(*agent_port);
                data_writer.put_u8(*unknown);
            }
            LoginResult::Error { error,  } => {
                data_writer.put_u8(2);
                match &error {
                    SecurityError::InvalidCredentials { max_attempts, current_attempts,  } => {
                        data_writer.put_u8(1);
                        data_writer.put_u32_le(*max_attempts);
                        data_writer.put_u32_le(*current_attempts);
                    }
                    SecurityError::Blocked { reason,  } => {
                        data_writer.put_u8(2);
                        match &reason {
                            BlockReason::AccountInspection => data_writer.put_u8(2),
                            BlockReason::Punishment { reason, end,  } => {
                                data_writer.put_u8(1);
                                data_writer.put_u16_le(reason.len() as u16);
                                data_writer.put_slice(reason.as_bytes());
                                data_writer.put_u16_le(end.year() as u16);
                                data_writer.put_u16_le(end.month() as u16);
                                data_writer.put_u16_le(end.day() as u16);
                                data_writer.put_u16_le(end.hour() as u16);
                                data_writer.put_u16_le(end.minute() as u16);
                                data_writer.put_u16_le(end.second() as u16);
                                data_writer.put_u32_le(end.timestamp_millis() as u32);
                            }
                        }
                    }
                    SecurityError::AlreadyConnected => data_writer.put_u8(3),
                    SecurityError::Inspection => data_writer.put_u8(4),
                    SecurityError::ServerFull => data_writer.put_u8(6),
                    SecurityError::LoginQueue { total_in_queue, expected_wait_time,  } => {
                        data_writer.put_u8(0x1A);
                        data_writer.put_u16_le(*total_in_queue);
                        data_writer.put_u32_le(*expected_wait_time);
                    }
                    SecurityError::PasswordExpired => data_writer.put_u8(0x2A),
                    SecurityError::IpLimit => data_writer.put_u8(8),
                }
            }
            LoginResult::Unknown => data_writer.put_u8(3),
        }
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for LoginResponse {
    fn into(self) -> ServerPacket {
        ServerPacket::LoginResponse(self)
    }
}

impl LoginResponse {
    pub fn new(result: LoginResult) -> Self {
        LoginResponse { result,  }
    }
}

#[derive(Clone)]
pub struct SecurityCodeInput {
    pub action: SecurityCodeAction,
    pub inner_size: u16,
    pub data: Bytes,
}

impl TryFrom<Bytes> for SecurityCodeInput {
    type Error = ProtocolError;

    fn try_from(data: Bytes) -> Result<Self, Self::Error> {
        let mut data_reader = data.reader();
        let action = match data_reader.read_u8()? {
            0x01 => SecurityCodeAction::Define,
            0x04 => SecurityCodeAction::Enter,
            0xFF => SecurityCodeAction::Unknown,
            unknown => return Err(ProtocolError::UnknownVariation(unknown, "SecurityCodeAction"))
        };
        let inner_size = data_reader.read_u16::<byteorder::LittleEndian>()?;
        let mut data_raw = BytesMut::with_capacity(8);
        for _ in 0..8 {
            	data_raw.put_u8(data_reader.read_u8()?);
        }
        let data = data_raw.freeze();
        Ok(SecurityCodeInput { action, inner_size, data,  })
    }
}

impl Into<ClientPacket> for SecurityCodeInput {
    fn into(self) -> ClientPacket {
        ClientPacket::SecurityCodeInput(self)
    }
}

#[derive(Clone)]
pub struct SecurityCodeResponse {
    pub account_status: PasscodeAccountStatus,
    pub result: u8,
    pub invalid_attempts: u8,
}

impl From<SecurityCodeResponse> for Bytes {
    fn from(op: SecurityCodeResponse) -> Bytes {
        let mut data_writer = BytesMut::new();
        match &op.account_status {
            PasscodeAccountStatus::Ok => data_writer.put_u8(4),
            PasscodeAccountStatus::EmailUnverified => data_writer.put_u8(2),
        }
        data_writer.put_u8(op.result);
        data_writer.put_u8(op.invalid_attempts);
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for SecurityCodeResponse {
    fn into(self) -> ServerPacket {
        ServerPacket::SecurityCodeResponse(self)
    }
}

impl SecurityCodeResponse {
    pub fn new(account_status: PasscodeAccountStatus, result: u8, invalid_attempts: u8) -> Self {
        SecurityCodeResponse { account_status, result, invalid_attempts,  }
    }
}

#[derive(Clone)]
pub struct GatewayNoticeRequest {
    pub unknown: u8,
}

impl TryFrom<Bytes> for GatewayNoticeRequest {
    type Error = ProtocolError;

    fn try_from(data: Bytes) -> Result<Self, Self::Error> {
        let mut data_reader = data.reader();
        let unknown = data_reader.read_u8()?;
        Ok(GatewayNoticeRequest { unknown,  })
    }
}

impl Into<ClientPacket> for GatewayNoticeRequest {
    fn into(self) -> ClientPacket {
        ClientPacket::GatewayNoticeRequest(self)
    }
}

#[derive(Clone)]
pub struct GatewayNoticeResponse {
    pub notices: Vec<GatewayNotice>,
}

impl From<GatewayNoticeResponse> for Bytes {
    fn from(op: GatewayNoticeResponse) -> Bytes {
        let mut data_writer = BytesMut::new();
        data_writer.put_u8(op.notices.len() as u8);
        for element in op.notices.iter() {
            data_writer.put_u16_le(element.subject.len() as u16);
            data_writer.put_slice(element.subject.as_bytes());
            data_writer.put_u16_le(element.article.len() as u16);
            data_writer.put_slice(element.article.as_bytes());
            data_writer.put_u16_le(element.published.year() as u16);
            data_writer.put_u16_le(element.published.month() as u16);
            data_writer.put_u16_le(element.published.day() as u16);
            data_writer.put_u16_le(element.published.hour() as u16);
            data_writer.put_u16_le(element.published.minute() as u16);
            data_writer.put_u16_le(element.published.second() as u16);
            data_writer.put_u32_le(element.published.timestamp_millis() as u32);
        }
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for GatewayNoticeResponse {
    fn into(self) -> ServerPacket {
        ServerPacket::GatewayNoticeResponse(self)
    }
}

impl GatewayNoticeResponse {
    pub fn new(notices: Vec<GatewayNotice>) -> Self {
        GatewayNoticeResponse { notices,  }
    }
}

#[derive(Clone)]
pub struct PingServerRequest;

impl TryFrom<Bytes> for PingServerRequest {
    type Error = ProtocolError;

    fn try_from(data: Bytes) -> Result<Self, Self::Error> {
        let mut data_reader = data.reader();
        Ok(PingServerRequest {  })
    }
}

impl Into<ClientPacket> for PingServerRequest {
    fn into(self) -> ClientPacket {
        ClientPacket::PingServerRequest(self)
    }
}

#[derive(Clone)]
pub struct PingServerResponse {
    pub servers: Vec<PingServer>,
}

impl From<PingServerResponse> for Bytes {
    fn from(op: PingServerResponse) -> Bytes {
        let mut data_writer = BytesMut::new();
        data_writer.put_u8(op.servers.len() as u8);
        for element in op.servers.iter() {
            data_writer.put_u8(element.index);
            data_writer.put_u16_le(element.domain.len() as u16);
            data_writer.put_slice(element.domain.as_bytes());
            data_writer.put_u8(element.unknown_1);
            data_writer.put_u8(element.unknown_2);
        }
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for PingServerResponse {
    fn into(self) -> ServerPacket {
        ServerPacket::PingServerResponse(self)
    }
}

impl PingServerResponse {
    pub fn new(servers: Vec<PingServer>) -> Self {
        PingServerResponse { servers,  }
    }
}

#[derive(Clone)]
pub struct ShardListRequest;

impl TryFrom<Bytes> for ShardListRequest {
    type Error = ProtocolError;

    fn try_from(data: Bytes) -> Result<Self, Self::Error> {
        let mut data_reader = data.reader();
        Ok(ShardListRequest {  })
    }
}

impl Into<ClientPacket> for ShardListRequest {
    fn into(self) -> ClientPacket {
        ClientPacket::ShardListRequest(self)
    }
}

#[derive(Clone)]
pub struct ShardListResponse {
    pub farms: Vec<Farm>,
    pub shards: Vec<Shard>,
}

impl From<ShardListResponse> for Bytes {
    fn from(op: ShardListResponse) -> Bytes {
        let mut data_writer = BytesMut::new();
        for element in op.farms.iter() {
            data_writer.put_u8(1);
            data_writer.put_u8(element.id);
            data_writer.put_u16_le(element.name.len() as u16);
            data_writer.put_slice(element.name.as_bytes());
        }
        data_writer.put_u8(0);
        for element in op.shards.iter() {
            data_writer.put_u8(1);
            data_writer.put_u16_le(element.id);
            data_writer.put_u16_le(element.name.len() as u16);
            data_writer.put_slice(element.name.as_bytes());
            data_writer.put_u8(element.status);
            data_writer.put_u8(element.is_online as u8);
        }
        data_writer.put_u8(0);
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for ShardListResponse {
    fn into(self) -> ServerPacket {
        ServerPacket::ShardListResponse(self)
    }
}

impl ShardListResponse {
    pub fn new(farms: Vec<Farm>, shards: Vec<Shard>) -> Self {
        ShardListResponse { farms, shards,  }
    }
}

#[derive(Clone)]
pub struct PasscodeRequiredResponse {
    pub result: PasscodeRequiredCode,
}

impl From<PasscodeRequiredResponse> for Bytes {
    fn from(op: PasscodeRequiredResponse) -> Bytes {
        let mut data_writer = BytesMut::new();
        match &op.result {
            PasscodeRequiredCode::DefinePasscode => data_writer.put_u8(0),
            PasscodeRequiredCode::PasscodeRequired => data_writer.put_u8(1),
            PasscodeRequiredCode::PasscodeBlocked => data_writer.put_u8(2),
            PasscodeRequiredCode::PasscodeInvalid => data_writer.put_u8(3),
        }
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for PasscodeRequiredResponse {
    fn into(self) -> ServerPacket {
        ServerPacket::PasscodeRequiredResponse(self)
    }
}

impl PasscodeRequiredResponse {
    pub fn new(result: PasscodeRequiredCode) -> Self {
        PasscodeRequiredResponse { result,  }
    }
}

#[derive(Clone)]
pub struct PasscodeResponse {
    pub unknown_1: u8,
    pub status: u8,
    pub invalid_attempts: u8,
}

impl From<PasscodeResponse> for Bytes {
    fn from(op: PasscodeResponse) -> Bytes {
        let mut data_writer = BytesMut::new();
        data_writer.put_u8(op.unknown_1);
        data_writer.put_u8(op.status);
        data_writer.put_u8(op.invalid_attempts);
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for PasscodeResponse {
    fn into(self) -> ServerPacket {
        ServerPacket::PasscodeResponse(self)
    }
}

impl PasscodeResponse {
    pub fn new(unknown_1: u8, status: u8, invalid_attempts: u8) -> Self {
        PasscodeResponse { unknown_1, status, invalid_attempts,  }
    }
}

#[derive(Clone)]
pub struct QueueUpdate {
    pub still_in_queue: bool,
    pub status: QueueUpdateStatus,
}

impl From<QueueUpdate> for Bytes {
    fn from(op: QueueUpdate) -> Bytes {
        let mut data_writer = BytesMut::new();
        data_writer.put_u8(op.still_in_queue as u8);
        data_writer.put_u16_le(op.status.total_in_queue);
        data_writer.put_u32_le(op.status.expected_wait_time);
        data_writer.put_u16_le(op.status.current_position);
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for QueueUpdate {
    fn into(self) -> ServerPacket {
        ServerPacket::QueueUpdate(self)
    }
}

impl QueueUpdate {
    pub fn new(still_in_queue: bool, status: QueueUpdateStatus) -> Self {
        QueueUpdate { still_in_queue, status,  }
    }
}