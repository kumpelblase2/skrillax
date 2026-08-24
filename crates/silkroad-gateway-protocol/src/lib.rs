pub use silkroad_base_protocol::*;
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

#[derive(Clone, Deserialize, Serialize, ByteSize, Debug)]
pub enum BlockReason {
    #[silkroad(value = 2)]
    AccountInspection,
    #[silkroad(value = 1)]
    Punishment { reason: String, end: ExpandedSilkroadTime },
}

impl BlockReason {
    pub fn punishment(reason: String, end: ExpandedSilkroadTime) -> Self {
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
    #[cfg_attr(feature = "v657", silkroad(size = 2))]
    pub subject: String,
    #[cfg_attr(feature = "v657", silkroad(size = 2))]
    pub article: String,
    pub published: ExpandedSilkroadTime,
}

impl GatewayNotice {
    pub fn new(subject: String, article: String, published: ExpandedSilkroadTime) -> Self {
        GatewayNotice {
            subject,
            article,
            published,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, ByteSize, Debug)]
pub struct PingServer {
    #[cfg(feature = "v594")]
    pub index: u8,
    pub domain: String,
    pub unknown: u16,
    #[cfg(feature = "v657")]
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
    #[cfg_attr(feature = "v657", silkroad(size = 2))]
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

#[derive(Clone, Serialize, Deserialize, ByteSize, Packet)]
#[packet(opcode = 0x610A)]
pub struct LoginRequest {
    pub unknown_1: u8,
    pub username: String,
    pub password: String,
    pub shard_id: u16,
    pub unknown_2: u8,
}

impl std::fmt::Debug for LoginRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoginRequest")
            .field("unknown_1", &self.unknown_1)
            .field("username", &self.username)
            .field("password", &"[REDACTED]")
            .field("shard_id", &self.shard_id)
            .field("unknown_2", &self.unknown_2)
            .finish()
    }
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

#[derive(Clone, Deserialize, Serialize, ByteSize, Packet, Debug)]
#[packet(opcode = 0x2005)]
pub struct FrameworkStateUpdate {
    pub flag: u8,
    #[silkroad(when = "flag & 0x01 != 0")]
    pub server_body: Option<FrameworkStateServerBodyCont>,
    #[silkroad(when = "flag & 0x02 != 0")]
    pub server_cord: Option<FrameworkStateServerCordCont>,
}

#[derive(Clone, Deserialize, Serialize, ByteSize, Debug)]
pub struct FrameworkStateServerBodyCont {
    pub _empty: u8,
    #[silkroad(list_type = "break")]
    pub inner: Vec<FrameworkStateServerBody>,
}

#[derive(Clone, Deserialize, Serialize, ByteSize, Debug)]
pub struct FrameworkStateServerBody {
    pub id: u16,
    pub state: u32,
}

#[derive(Clone, Deserialize, Serialize, ByteSize, Debug)]
pub struct FrameworkStateServerCordCont {
    pub _empty: u8,
    #[silkroad(list_type = "break")]
    pub inner: Vec<FrameworkStateServerCord>,
}

#[derive(Clone, Deserialize, Serialize, ByteSize, Debug)]
pub struct FrameworkStateServerCord {
    pub cord: u32,
    pub state: u32,
}

#[derive(Clone, Deserialize, Serialize, ByteSize, Packet, Debug)]
#[packet(opcode = 0x6005)]
pub struct FrameworkStateRequest {
    pub flag: u8,
    #[silkroad(when = "flag & 0x01 != 0")]
    pub server_body: Option<FrameworkRequestServerBodyCont>,
    #[silkroad(when = "flag & 0x02 != 0")]
    pub server_cord: Option<FrameworkRequestServerCordCont>,
}

#[derive(Clone, Deserialize, Serialize, ByteSize, Debug)]
pub struct FrameworkRequestServerBodyCont {
    pub _empty: u8,
    #[silkroad(list_type = "break")]
    pub inner: Vec<u32>,
}

#[derive(Clone, Deserialize, Serialize, ByteSize, Debug)]
pub struct FrameworkRequestServerCordCont {
    pub _empty: u8,
    #[silkroad(list_type = "break")]
    pub inner: Vec<u32>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_request_debug_redacts_password() {
        let request = LoginRequest {
            unknown_1: 1,
            username: "test-user".to_owned(),
            password: "login-password-canary".to_owned(),
            shard_id: 7,
            unknown_2: 2,
        };

        for output in [format!("{request:?}"), format!("{request:#?}")] {
            assert!(!output.contains("login-password-canary"));
            assert!(output.contains("[REDACTED]"));
            assert!(output.contains("test-user"));
        }
    }
}
