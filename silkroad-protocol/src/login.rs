use chrono::{DateTime, Utc};
use silkroad_serde::*;

#[derive(Clone, Eq, PartialEq, Copy, Serialize, ByteSize, Deserialize)]
pub enum SecurityCodeAction {
    #[silkroad(value = 1)]
    Define,
    #[silkroad(value = 4)]
    Enter,
    #[silkroad(value = 0xFF)]
    Unknown,
}

#[derive(Clone, Eq, PartialEq, Copy, Serialize, ByteSize)]
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

#[derive(Clone, Serialize, ByteSize)]
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

#[derive(Clone, Serialize, ByteSize)]
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

#[derive(Clone, Eq, PartialEq, Copy, Serialize, ByteSize)]
pub enum PasscodeAccountStatus {
    #[silkroad(value = 4)]
    Ok,
    #[silkroad(value = 2)]
    EmailUnverified,
}

#[derive(Clone, Serialize, ByteSize)]
pub enum BlockReason {
    #[silkroad(value = 2)]
    AccountInspection,
    #[silkroad(value = 1)]
    Punishment { reason: String, end: DateTime<Utc> },
}

impl BlockReason {
    pub fn punishment(reason: String, end: DateTime<Utc>) -> Self {
        BlockReason::Punishment { reason, end }
    }
}

#[derive(Clone, Serialize, ByteSize)]
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

#[derive(Clone, Serialize, ByteSize)]
pub enum LoginResult {
    #[silkroad(value = 1)]
    Success {
        session_id: u32,
        agent_ip: String,
        agent_port: u16,
        unknown: u8,
    },
    #[silkroad(value = 2)]
    Error { error: SecurityError },
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
        LoginResult::Error { error }
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

#[derive(Clone, Serialize, ByteSize)]
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

#[derive(Clone, Serialize, ByteSize)]
pub struct GatewayNotice {
    pub subject: String,
    pub article: String,
    pub published: DateTime<Utc>,
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

#[derive(Clone, Serialize, ByteSize)]
pub struct PingServer {
    pub index: u8,
    pub domain: String,
    pub unknown: u16,
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

#[derive(Clone, Serialize, ByteSize)]
pub struct Shard {
    pub id: u16,
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

#[derive(Clone, Serialize, ByteSize)]
pub struct Farm {
    pub id: u8,
    pub name: String,
}

impl Farm {
    pub fn new(id: u8, name: String) -> Self {
        Farm { id, name }
    }
}

#[derive(Clone, Deserialize, ByteSize)]
pub struct PatchRequest {
    pub content: u8,
    pub module: String,
    pub version: u32,
}

#[derive(Clone, Serialize, ByteSize)]
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

#[derive(Clone, Deserialize, ByteSize)]
pub struct LoginRequest {
    pub unknown_1: u8,
    pub username: String,
    pub password: String,
    pub shard_id: u16,
    pub unknown_2: u8,
}

#[derive(Clone, Serialize, ByteSize)]
pub struct LoginResponse {
    pub result: LoginResult,
}

impl LoginResponse {
    pub fn new(result: LoginResult) -> Self {
        LoginResponse { result }
    }

    pub fn error(error: SecurityError) -> Self {
        LoginResponse {
            result: LoginResult::Error { error },
        }
    }
}

#[derive(Clone, Deserialize, ByteSize)]
pub struct SecurityCodeInput {
    pub action: SecurityCodeAction,
    pub inner_size: u16,
    pub data: [u8; 8],
}

#[derive(Clone, Serialize, ByteSize)]
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

#[derive(Clone, Deserialize, ByteSize)]
pub struct GatewayNoticeRequest {
    pub unknown: u8,
}

#[derive(Clone, Serialize, ByteSize)]
pub struct GatewayNoticeResponse {
    #[silkroad(list_type = "length")]
    pub notices: Vec<GatewayNotice>,
}

impl GatewayNoticeResponse {
    pub fn new(notices: Vec<GatewayNotice>) -> Self {
        GatewayNoticeResponse { notices }
    }
}

#[derive(Clone, Deserialize, ByteSize)]
pub struct PingServerRequest;

#[derive(Clone, Serialize, ByteSize)]
pub struct PingServerResponse {
    #[silkroad(list_type = "length")]
    pub servers: Vec<PingServer>,
}

impl PingServerResponse {
    pub fn new(servers: Vec<PingServer>) -> Self {
        PingServerResponse { servers }
    }
}

#[derive(Clone, Deserialize, ByteSize)]
pub struct ShardListRequest;

#[derive(Clone, Serialize, ByteSize)]
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

#[derive(Clone, Serialize, ByteSize)]
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
#[derive(Clone, Serialize, ByteSize)]
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

#[derive(Clone, Serialize, ByteSize)]
pub struct QueueUpdate {
    pub still_in_queue: bool,
    pub status: QueueUpdateStatus,
}

impl QueueUpdate {
    pub fn new(still_in_queue: bool, status: QueueUpdateStatus) -> Self {
        QueueUpdate { still_in_queue, status }
    }
}
