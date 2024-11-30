use chrono::{DateTime, Utc};
use skrillax_packet::Packet;
use skrillax_serde::*;

#[derive(Clone, Eq, PartialEq, Copy, Serialize, ByteSize, Deserialize, Debug)]
pub enum SecurityCodeAction {
    #[silkroad(value = 1)]
    Define,
    #[silkroad(value = 4)]
    Enter,
    #[silkroad(value = 0xFF)]
    Unknown,
}

#[derive(Clone, Eq, PartialEq, Copy, Deserialize, Serialize, ByteSize, Debug)]
pub enum PasscodeRequiredCode {
    #[silkroad(value = 0)]
    DefinePasscode,
    #[silkroad(value = 1)]
    PasscodeRequired,
    #[silkroad(value = 2)]
    PasscodeBlocked,
    #[silkroad(value = 3)]
    PasscodeInvalid,
}

#[derive(Clone, Deserialize, Serialize, ByteSize, Debug)]
pub enum PatchError {
    #[silkroad(value = 1)]
    InvalidVersion,
    #[silkroad(value = 2)]
    Update {
        server_ip: String,
        server_port: u16,
        current_version: u32,
        #[silkroad(list_type = "has-more")]
        patch_files: Vec<PatchFile>,
        http_server: String,
    },
    #[silkroad(value = 3)]
    Offline,
    #[silkroad(value = 4)]
    InvalidClient,
    #[silkroad(value = 5)]
    PatchDisabled,
}

impl PatchError {
    pub fn update(
        server_ip: String,
        server_port: u16,
        current_version: u32,
        patch_files: Vec<PatchFile>,
        http_server: String,
    ) -> Self {
        PatchError::Update {
            server_ip,
            server_port,
            current_version,
            patch_files,
            http_server,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, ByteSize, Debug)]
pub enum PatchResult {
    #[silkroad(value = 1)]
    UpToDate { unknown: u8 },
    #[silkroad(value = 2)]
    Problem { error: PatchError },
}

impl PatchResult {
    pub fn uptodate() -> Self {
        PatchResult::UpToDate { unknown: 0 }
    }

    pub fn problem(error: PatchError) -> Self {
        PatchResult::Problem { error }
    }
}

#[derive(Clone, Eq, PartialEq, Copy, Deserialize, Serialize, ByteSize, Debug)]
pub enum PasscodeAccountStatus {
    #[silkroad(value = 4)]
    Ok,
    #[silkroad(value = 2)]
    EmailUnverified,
}

type NormalDateTime = DateTime<Utc>;

#[derive(Clone, Deserialize, Serialize, ByteSize, Debug)]
pub enum BlockReason {
    #[silkroad(value = 2)]
    AccountInspection,
    #[silkroad(value = 1)]
    Punishment { reason: String, end: NormalDateTime },
}

impl BlockReason {
    pub fn punishment(reason: String, end: NormalDateTime) -> Self {
        BlockReason::Punishment { reason, end }
    }
}

#[derive(Clone, Deserialize, Serialize, ByteSize, Debug)]
pub enum SecurityError {
    #[silkroad(value = 1)]
    InvalidCredentials { max_attempts: u32, current_attempts: u32 },
    #[silkroad(value = 2)]
    Blocked { reason: BlockReason },
    #[silkroad(value = 3)]
    AlreadyConnected,
    #[silkroad(value = 4)]
    Inspection,
    #[silkroad(value = 6)]
    ServerFull,
    #[silkroad(value = 0x1A)]
    LoginQueue {
        total_in_queue: u16,
        expected_wait_time: u32,
    },
    #[silkroad(value = 0x2A)]
    PasswordExpired,
    #[silkroad(value = 8)]
    IpLimit,
    #[silkroad(value = 0x1C)]
    QueueLimitReached,
}

impl SecurityError {
    pub fn invalidcredentials(max_attempts: u32, current_attempts: u32) -> Self {
        SecurityError::InvalidCredentials {
            max_attempts,
            current_attempts,
        }
    }

    pub fn blocked(reason: BlockReason) -> Self {
        SecurityError::Blocked { reason }
    }

    pub fn loginqueue(total_in_queue: u16, expected_wait_time: u32) -> Self {
        SecurityError::LoginQueue {
            total_in_queue,
            expected_wait_time,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, ByteSize, Debug)]
pub enum LoginResult {
    #[silkroad(value = 1)]
    Success {
        session_id: u32,
        agent_ip: String,
        agent_port: u16,
        unknown: u8,
    },
    #[silkroad(value = 2)]
    LoginError { error: SecurityError },
    #[silkroad(value = 3)]
    Unknown,
}

impl LoginResult {
    pub fn success(session_id: u32, agent_ip: String, agent_port: u16) -> Self {
        LoginResult::Success {
            session_id,
            agent_ip,
            agent_port,
            unknown: 1,
        }
    }

    pub fn error(error: SecurityError) -> Self {
        LoginResult::LoginError { error }
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub struct QueueUpdateStatus {
    pub total_in_queue: u16,
    pub expected_wait_time: u32,
    pub current_position: u16,
}

impl QueueUpdateStatus {
    pub fn new(total_in_queue: u16, expected_wait_time: u32, current_position: u16) -> Self {
        QueueUpdateStatus {
            total_in_queue,
            expected_wait_time,
            current_position,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, ByteSize, Debug)]
pub struct PatchFile {
    pub file_id: u32,
    pub filename: String,
    pub file_path: String,
    pub size: u32,
    pub in_pk2: bool,
}

impl PatchFile {
    pub fn new(file_id: u32, filename: String, file_path: String, size: u32, in_pk2: bool) -> Self {
        PatchFile {
            file_id,
            filename,
            file_path,
            size,
            in_pk2,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, ByteSize, Debug)]
pub struct GatewayNotice {
    // #[cfg_attr(feature = "v657", silkroad(size = 2))]
    #[silkroad(size = 2)]
    pub subject: String,
    // #[cfg_attr(feature = "v657", silkroad(size = 2))]
    #[silkroad(size = 2)]
    pub article: String,
    pub published: NormalDateTime,
}

impl GatewayNotice {
    pub fn new(subject: String, article: String, published: DateTime<Utc>) -> Self {
        GatewayNotice {
            subject,
            article,
            published,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, ByteSize, Debug)]
pub struct PingServer {
    pub domain: String,
    pub unknown: u16,
    pub index: u8,
}

impl PingServer {
    pub fn new(index: u8, domain: String) -> Self {
        PingServer {
            index,
            domain,
            unknown: 0x32bd,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, ByteSize, Debug)]
pub struct Shard {
    pub id: u16,
    #[silkroad(size = 2)]
    pub name: String,
    pub status: u8,
    pub is_online: bool,
}

impl Shard {
    pub fn new(id: u16, name: String, status: u8, is_online: bool) -> Self {
        Shard {
            id,
            name,
            status,
            is_online,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, ByteSize, Debug)]
pub struct Farm {
    pub id: u8,
    pub name: String,
}

impl Farm {
    pub fn new(id: u8, name: String) -> Self {
        Farm { id, name }
    }
}

#[derive(Clone, Serialize, Deserialize, ByteSize, Packet, Debug)]
#[packet(opcode = 0x6100)]
pub struct PatchRequest {
    pub content: u8,
    pub module: String,
    pub version: u32,
}

#[derive(Clone, Serialize, Deserialize, ByteSize, Packet, Debug)]
#[packet(opcode = 0xA100, massive = true)]
pub struct PatchResponse {
    pub result: PatchResult,
}

impl PatchResponse {
    pub fn new(result: PatchResult) -> Self {
        PatchResponse { result }
    }

    pub fn up_to_date() -> Self {
        PatchResponse {
            result: PatchResult::UpToDate { unknown: 0 },
        }
    }

    pub fn error(error: PatchError) -> Self {
        PatchResponse {
            result: PatchResult::Problem { error },
        }
    }
}

#[derive(Clone, Serialize, Deserialize, ByteSize, Packet, Debug)]
#[packet(opcode = 0x610A)]
pub struct LoginRequest {
    pub unknown_1: u8,
    pub username: String,
    pub password: String,
    pub shard_id: u16,
    pub unknown_2: u8,
}

#[derive(Clone, Deserialize, Serialize, ByteSize, Packet, Debug)]
#[packet(opcode = 0xA10A, encrypted = true)]
pub struct LoginResponse {
    pub result: LoginResult,
}

impl LoginResponse {
    pub fn new(result: LoginResult) -> Self {
        LoginResponse { result }
    }

    pub fn error(error: SecurityError) -> Self {
        LoginResponse {
            result: LoginResult::error(error),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, ByteSize, Packet, Debug)]
#[packet(opcode = 0x6117)]
pub struct SecurityCodeInput {
    pub action: SecurityCodeAction,
    pub inner_size: u16,
    pub data: [u8; 8],
}

#[derive(Clone, Deserialize, Serialize, ByteSize, Packet, Debug)]
#[packet(opcode = 0xA117)]
pub struct SecurityCodeResponse {
    pub account_status: PasscodeAccountStatus,
    pub result: u8,
    pub invalid_attempts: u8,
}

impl SecurityCodeResponse {
    pub fn new(account_status: PasscodeAccountStatus, result: u8, invalid_attempts: u8) -> Self {
        SecurityCodeResponse {
            account_status,
            result,
            invalid_attempts,
        }
    }

    pub fn success() -> Self {
        SecurityCodeResponse {
            account_status: PasscodeAccountStatus::Ok,
            result: 1,
            invalid_attempts: 3,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, ByteSize, Packet, Debug)]
#[packet(opcode = 0x6104)]
pub struct GatewayNoticeRequest {
    pub unknown: u8,
}

#[derive(Clone, Serialize, Deserialize, ByteSize, Packet, Debug)]
#[packet(opcode = 0xA104, massive = true)]
pub struct GatewayNoticeResponse {
    #[silkroad(list_type = "length")]
    pub notices: Vec<GatewayNotice>,
}

impl GatewayNoticeResponse {
    pub fn new(notices: Vec<GatewayNotice>) -> Self {
        GatewayNoticeResponse { notices }
    }
}

#[derive(Clone, Serialize, Deserialize, ByteSize, Packet, Debug)]
#[packet(opcode = 0x6107)]
pub struct PingServerRequest;

#[derive(Clone, Deserialize, Serialize, ByteSize, Packet, Debug)]
#[packet(opcode = 0xA107)]
pub struct PingServerResponse {
    #[silkroad(list_type = "length")]
    pub servers: Vec<PingServer>,
}

impl PingServerResponse {
    pub fn new(servers: Vec<PingServer>) -> Self {
        PingServerResponse { servers }
    }
}

#[derive(Clone, Serialize, Deserialize, ByteSize, Packet, Debug)]
#[packet(opcode = 0x6101)]
pub struct ShardListRequest;

#[derive(Clone, Deserialize, Serialize, ByteSize, Packet, Debug)]
#[packet(opcode = 0xA101)]
pub struct ShardListResponse {
    #[silkroad(list_type = "has-more")]
    pub farms: Vec<Farm>,
    #[silkroad(list_type = "has-more")]
    pub shards: Vec<Shard>,
}

impl ShardListResponse {
    pub fn new(farms: Vec<Farm>, shards: Vec<Shard>) -> Self {
        ShardListResponse { farms, shards }
    }
}

#[derive(Clone, Deserialize, Serialize, ByteSize, Packet, Debug)]
#[packet(opcode = 0x2116)]
pub struct PasscodeRequiredResponse {
    pub result: PasscodeRequiredCode,
}

impl PasscodeRequiredResponse {
    pub fn new(result: PasscodeRequiredCode) -> Self {
        PasscodeRequiredResponse { result }
    }

    pub fn define_passcode() -> Self {
        PasscodeRequiredResponse {
            result: PasscodeRequiredCode::DefinePasscode,
        }
    }

    pub fn passcode_required() -> Self {
        PasscodeRequiredResponse {
            result: PasscodeRequiredCode::PasscodeRequired,
        }
    }

    pub fn passcode_invalid() -> Self {
        PasscodeRequiredResponse {
            result: PasscodeRequiredCode::PasscodeInvalid,
        }
    }

    pub fn passcode_blocked() -> Self {
        PasscodeRequiredResponse {
            result: PasscodeRequiredCode::PasscodeBlocked,
        }
    }
}

// This should be some kind of enum, because on error the last two bytes are (Error=0x2, WrongAttempts)
// but on success it's (Ok=0x1, Unknown=0x3)
#[derive(Clone, Deserialize, Serialize, ByteSize, Packet, Debug)]
#[packet(opcode = 0xA117)]
pub struct PasscodeResponse {
    pub unknown_1: u8,
    pub status: u8,
    pub invalid_attempts: u8,
}

impl PasscodeResponse {
    pub fn new(status: u8, invalid_attempts: u8) -> Self {
        PasscodeResponse {
            unknown_1: 4,
            status,
            invalid_attempts,
        }
    }
}

#[derive(Clone, Serialize, ByteSize, Packet)]
#[packet(opcode = 0x210E)]
pub struct QueueUpdate {
    pub still_in_queue: bool,
    pub status: QueueUpdateStatus,
}

impl QueueUpdate {
    pub fn new(still_in_queue: bool, status: QueueUpdateStatus) -> Self {
        QueueUpdate { still_in_queue, status }
    }
}

#[derive(Packet, Serialize, Deserialize, ByteSize, Clone, Debug, Copy)]
#[packet(opcode = 0x3612)]
pub struct UnknownLargePacket {
    data: [u8; 494],
}

#[derive(Packet, Serialize, Deserialize, ByteSize, Clone, Debug)]
#[packet(opcode = 0x3013)]
pub struct TempCharacterData {
    data: [u8; 1523],
}

impl TempCharacterData {
    pub fn new() -> Self {
        Self {
            data: [
                0xd8, 0xe2, 0x0a, 0xe7, 0x81, 0x07, 0x00, 0x00, 0x22, 0x1e, 0x1e, 0xbf, 0x8c, 0x06, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x4c, 0x01, 0x00, 0x00, 0xc2, 0x26, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00, 0xbb, 0x3e, 0x00,
                0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0xe5, 0x05, 0x00, 0x00, 0xce, 0x07, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x18, 0x0b, 0x00, 0x00, 0x03, 0x8c, 0x07,
                0x01, 0x37, 0x2e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x15, 0x06, 0x00, 0x00, 0x04, 0x02, 0x14, 0x71, 0x02,
                0x00, 0x00, 0x00, 0x00, 0x2f, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x98, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0x04, 0x00, 0x01,
                0x00, 0x00, 0x00, 0x00, 0x80, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x2f, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0x04, 0x00, 0x02, 0x00, 0x00, 0x00,
                0x00, 0x5e, 0x06, 0x00, 0x00, 0x00, 0xe1, 0x89, 0x71, 0x1a, 0x00, 0x00, 0x00, 0x00, 0x32, 0x00, 0x00,
                0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0b, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00,
                0x00, 0x80, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
                0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0x04, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0xc9, 0x06, 0x00, 0x00,
                0x00, 0x12, 0x91, 0x04, 0x10, 0x00, 0x00, 0x00, 0x00, 0x35, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00,
                0x03, 0x00, 0x04, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0xa4, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0x04,
                0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0xec, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x2f, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0x04, 0x00, 0x06, 0x00,
                0x00, 0x00, 0x00, 0xe1, 0x00, 0x00, 0x00, 0x03, 0x61, 0xc6, 0x87, 0x44, 0x05, 0x00, 0x00, 0x00, 0x2a,
                0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0x04, 0x00, 0x07, 0x00, 0x00, 0x00, 0x00,
                0x3e, 0x00, 0x00, 0x00, 0x7e, 0x00, 0x09, 0x00, 0x00, 0x00, 0x00, 0x32, 0x07, 0x00, 0x00, 0x00, 0x20,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03,
                0x00, 0x04, 0x00, 0x0a, 0x00, 0x00, 0x00, 0x00, 0x57, 0x07, 0x00, 0x00, 0x00, 0xcc, 0x01, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0x04, 0x00,
                0x0b, 0x00, 0x00, 0x00, 0x00, 0x10, 0x07, 0x00, 0x00, 0x00, 0x2a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0x04, 0x00, 0x0c, 0x00, 0x00,
                0x00, 0x00, 0x0f, 0x07, 0x00, 0x00, 0x00, 0x70, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0b, 0x00, 0x00, 0x00, 0x03, 0x00,
                0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0x04, 0x00, 0x0d, 0x00, 0x00, 0x00, 0x00, 0x11, 0x5f,
                0x00, 0x00, 0x02, 0x00, 0x0e, 0x00, 0x00, 0x00, 0x00, 0x15, 0x24, 0x00, 0x00, 0x2b, 0x00, 0x10, 0x00,
                0x00, 0x00, 0x00, 0xbe, 0x00, 0x00, 0x00, 0x00, 0x07, 0x02, 0x01, 0x0e, 0x01, 0x00, 0x00, 0x00, 0x3d,
                0x00, 0x00, 0x00, 0x01, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03,
                0x00, 0x04, 0x00, 0x11, 0x00, 0x00, 0x00, 0x00, 0x38, 0x00, 0x00, 0x00, 0x1a, 0x00, 0x12, 0x00, 0x00,
                0x00, 0x00, 0x07, 0x5b, 0x00, 0x00, 0x03, 0x66, 0x5c, 0x00, 0x00, 0x00, 0x00, 0x0f, 0x00, 0x15, 0x00,
                0x00, 0x00, 0x00, 0x0d, 0x00, 0x00, 0x00, 0x18, 0x00, 0x16, 0x00, 0x00, 0x00, 0x00, 0x14, 0x5f, 0x00,
                0x00, 0x05, 0x00, 0x17, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x03, 0x00, 0x18, 0x00, 0x00,
                0x00, 0x00, 0xa3, 0x0f, 0x00, 0x00, 0x01, 0x00, 0x1a, 0x00, 0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00,
                0x02, 0x00, 0x1b, 0x00, 0x00, 0x00, 0x00, 0x88, 0x28, 0x00, 0x00, 0x14, 0x00, 0x1d, 0x00, 0x00, 0x00,
                0x00, 0x60, 0x12, 0x00, 0x00, 0x00, 0x20, 0x31, 0xa4, 0x35, 0x00, 0x00, 0x00, 0x00, 0x35, 0x00, 0x00,
                0x00, 0x02, 0x28, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00,
                0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0x04, 0x00, 0x1e, 0x00, 0x00, 0x00, 0x00, 0xa2, 0x35, 0x00,
                0x00, 0x03, 0x01, 0x9c, 0x85, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0e, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00,
                0x02, 0x00, 0x03, 0x00, 0x04, 0x00, 0x1f, 0x00, 0x00, 0x00, 0x00, 0xec, 0x05, 0x00, 0x00, 0x02, 0x44,
                0xb1, 0x90, 0x08, 0x00, 0x00, 0x00, 0x00, 0x33, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x50, 0x00, 0x00, 0x00, 0x1e, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00,
                0x04, 0x00, 0x21, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x0e, 0x00, 0x22, 0x00, 0x00, 0x00,
                0x00, 0x05, 0x00, 0x00, 0x00, 0x1c, 0x00, 0x23, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x32,
                0x00, 0x24, 0x00, 0x00, 0x00, 0x00, 0x9a, 0x00, 0x00, 0x00, 0x00, 0x89, 0x04, 0x37, 0x46, 0x02, 0x00,
                0x00, 0x00, 0x38, 0x00, 0x00, 0x00, 0x02, 0x1a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00,
                0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0x04, 0x00, 0x25, 0x00, 0x00,
                0x00, 0x00, 0x13, 0x5f, 0x00, 0x00, 0x02, 0x00, 0x26, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00,
                0x32, 0x00, 0x27, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x32, 0x00, 0x28, 0x00, 0x00, 0x00,
                0x00, 0x05, 0x00, 0x00, 0x00, 0x32, 0x00, 0x29, 0x00, 0x00, 0x00, 0x00, 0x14, 0x00, 0x00, 0x00, 0x01,
                0x00, 0x2b, 0x00, 0x00, 0x00, 0x00, 0x3e, 0x00, 0x00, 0x00, 0xfa, 0x00, 0x2c, 0x00, 0x00, 0x00, 0x00,
                0x3e, 0x00, 0x00, 0x00, 0xfa, 0x00, 0x2d, 0x00, 0x00, 0x00, 0x00, 0x3d, 0x00, 0x00, 0x00, 0x2d, 0x00,
                0x2e, 0x00, 0x00, 0x00, 0x00, 0x94, 0x0f, 0x00, 0x00, 0x02, 0x00, 0x2f, 0x00, 0x00, 0x00, 0x00, 0x95,
                0x0f, 0x00, 0x00, 0x03, 0x00, 0x31, 0x00, 0x00, 0x00, 0x00, 0x5f, 0x08, 0x00, 0x00, 0x0f, 0x00, 0x32,
                0x00, 0x00, 0x00, 0x00, 0x60, 0x08, 0x00, 0x00, 0x10, 0x00, 0x33, 0x00, 0x00, 0x00, 0x00, 0x0c, 0x00,
                0x00, 0x00, 0x2e, 0x00, 0x34, 0x00, 0x00, 0x00, 0x00, 0x0c, 0x00, 0x00, 0x00, 0x32, 0x00, 0x35, 0x00,
                0x00, 0x00, 0x00, 0x42, 0x1d, 0x00, 0x00, 0x01, 0x00, 0x36, 0x00, 0x00, 0x00, 0x00, 0x43, 0x1d, 0x00,
                0x00, 0x01, 0x00, 0x05, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x5b, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0xf7, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0xf6, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
                0x00, 0xfc, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0xf8, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00,
                0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0x04, 0x00, 0x00, 0x0b, 0x00, 0x00, 0x01, 0x01, 0x01, 0x00, 0x00,
                0x00, 0x01, 0x02, 0x01, 0x00, 0x00, 0x00, 0x01, 0x03, 0x01, 0x00, 0x00, 0x00, 0x01, 0x14, 0x01, 0x00,
                0x00, 0x00, 0x01, 0x15, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x02, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00,
                0xdc, 0x00, 0x00, 0x00, 0x04, 0x3f, 0x04, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x58, 0x08, 0x01, 0x01,
                0x00, 0x1b, 0x00, 0x53, 0x4e, 0x5f, 0x43, 0x4f, 0x4e, 0x5f, 0x51, 0x4e, 0x4f, 0x5f, 0x4c, 0x56, 0x5f,
                0x57, 0x45, 0x41, 0x50, 0x4f, 0x4e, 0x5f, 0x43, 0x48, 0x5f, 0x34, 0x5f, 0x31, 0x01, 0x00, 0x00, 0x00,
                0x00, 0x03, 0xfd, 0x96, 0x00, 0x00, 0xf6, 0x97, 0x00, 0x00, 0xf5, 0x97, 0x00, 0x00, 0x47, 0x04, 0x00,
                0x00, 0x00, 0x01, 0x00, 0x00, 0x58, 0x08, 0x01, 0x01, 0x00, 0x1b, 0x00, 0x53, 0x4e, 0x5f, 0x43, 0x4f,
                0x4e, 0x5f, 0x51, 0x4e, 0x4f, 0x5f, 0x4c, 0x56, 0x5f, 0x53, 0x48, 0x49, 0x45, 0x4c, 0x44, 0x5f, 0x43,
                0x48, 0x5f, 0x34, 0x5f, 0x31, 0x01, 0x00, 0x00, 0x00, 0x00, 0x03, 0xfd, 0x96, 0x00, 0x00, 0xf6, 0x97,
                0x00, 0x00, 0xf5, 0x97, 0x00, 0x00, 0x4f, 0x04, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x58, 0x08, 0x01,
                0x01, 0x00, 0x1c, 0x00, 0x53, 0x4e, 0x5f, 0x43, 0x4f, 0x4e, 0x5f, 0x51, 0x4e, 0x4f, 0x5f, 0x4c, 0x56,
                0x5f, 0x44, 0x45, 0x46, 0x45, 0x4e, 0x53, 0x45, 0x5f, 0x43, 0x48, 0x5f, 0x34, 0x5f, 0x31, 0x01, 0x00,
                0x00, 0x00, 0x00, 0x03, 0xfd, 0x96, 0x00, 0x00, 0xf6, 0x97, 0x00, 0x00, 0xf5, 0x97, 0x00, 0x00, 0x57,
                0x04, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x58, 0x08, 0x01, 0x01, 0x00, 0x1e, 0x00, 0x53, 0x4e, 0x5f,
                0x43, 0x4f, 0x4e, 0x5f, 0x51, 0x4e, 0x4f, 0x5f, 0x4c, 0x56, 0x5f, 0x41, 0x43, 0x43, 0x45, 0x53, 0x53,
                0x4f, 0x52, 0x59, 0x5f, 0x43, 0x48, 0x5f, 0x34, 0x5f, 0x31, 0x01, 0x00, 0x00, 0x00, 0x00, 0x03, 0xfd,
                0x96, 0x00, 0x00, 0xf6, 0x97, 0x00, 0x00, 0xf5, 0x97, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x74,
                0xd6, 0x9b, 0x01, 0x99, 0x66, 0xc8, 0xf8, 0x9d, 0x44, 0xd9, 0xb1, 0xd3, 0xc2, 0x30, 0xb7, 0x3f, 0x44,
                0xd8, 0x11, 0x00, 0x01, 0x00, 0xd8, 0x11, 0x00, 0x00, 0x00, 0x00, 0x00, 0x9a, 0x99, 0x99, 0x41, 0x01,
                0x00, 0x70, 0x42, 0x00, 0x00, 0xc8, 0x42, 0x00, 0x0c, 0x00, 0x42, 0x6f, 0x57, 0x65, 0x52, 0x5f, 0x46,
                0x65, 0x4d, 0x61, 0x4c, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xdf, 0xbf, 0xe3, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x2b, 0xef, 0x27, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
                0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x63,
            ],
        }
    }
}

pub struct UnknownLargePacketD {
    pub unknown_1: u8, // 0
    pub inner: Vec<UnknownLargePacketDInner>,
}

pub struct UnknownLargePacketDInner {
    pub index: u32,
    pub unknown_1: u8, // 0x01
    pub inner: Vec<UnknownLargePacketDInnerInner>,
}

pub struct UnknownLargePacketDInnerInner {
    pub index: u64,
    pub data: u32,
}

impl UnknownLargePacket {
    pub fn new() -> Self {
        UnknownLargePacket {
            data: [
                0x00, 0x16, 0x01, 0x00, 0x00, 0x00, 0x01, 0x01, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01, 0x01, 0x13, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x01, 0x01, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x01, 0x01, 0x0c, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x01, 0x01, 0x0d, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x01, 0x01, 0x0e, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x01, 0x01, 0x0f, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x01, 0x01, 0x10,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00, 0x01, 0x01,
                0x11, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0b, 0x00, 0x00, 0x00, 0x01,
                0x01, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x17, 0x00, 0x00, 0x00,
                0x01, 0x01, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x00, 0x00,
                0x00, 0x01, 0x01, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0x00,
                0x00, 0x00, 0x01, 0x01, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1a,
                0x00, 0x00, 0x00, 0x01, 0x01, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x1b, 0x00, 0x00, 0x00, 0x01, 0x01, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x1c, 0x00, 0x00, 0x00, 0x01, 0x01, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x1d, 0x00, 0x00, 0x00, 0x01, 0x01, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x1e, 0x00, 0x00, 0x00, 0x01, 0x01, 0x0a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x1f, 0x00, 0x00, 0x00, 0x01, 0x01, 0x0b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x5b, 0x00, 0x00, 0x00, 0x01, 0x05, 0x77, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x78, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x79, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7a, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x5c, 0x00, 0x00, 0x00, 0x01, 0x05, 0x7c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x7d, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7e,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7f, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00,
            ],
        }
    }
}
