use skrillax_packet::Packet;
use skrillax_protocol::define_protocol;
use skrillax_serde::*;

#[derive(Clone, Eq, PartialEq, PartialOrd, Copy, Serialize, ByteSize, Deserialize, Debug)]
pub enum LogoutMode {
    #[silkroad(value = 1)]
    Logout,
    #[silkroad(value = 2)]
    Restart,
}

#[derive(Clone, Serialize, ByteSize, Deserialize, Debug)]
pub enum LogoutResult {
    #[silkroad(value = 1)]
    Success { seconds_to_logout: u32, mode: LogoutMode },
    #[silkroad(value = 2)]
    Failure { error: u16 },
}

impl LogoutResult {
    pub fn success(seconds_to_logout: u32, mode: LogoutMode) -> Self {
        LogoutResult::Success {
            seconds_to_logout,
            mode,
        }
    }

    pub fn error(error: u16) -> Self {
        LogoutResult::Failure { error }
    }

    pub fn wait_30_seconds() -> Self {
        LogoutResult::Failure { error: 0x0804 }
    }
}

#[derive(Clone, Eq, PartialEq, PartialOrd, Copy, Serialize, Deserialize, ByteSize, Debug)]
pub enum AuthResultError {
    #[silkroad(value = 2)]
    InvalidData,
    #[silkroad(value = 3)]
    NotInService,
    #[silkroad(value = 4)]
    ServerFull,
    #[silkroad(value = 5)]
    IpLimit,
}

#[derive(Clone, Serialize, ByteSize, Deserialize, Debug)]
pub enum AuthResult {
    #[silkroad(value = 1)]
    Success { unknown_1: u8, unknown_2: u8 },
    #[silkroad(value = 2)]
    Failure { code: AuthResultError },
}

impl AuthResult {
    pub fn success() -> Self {
        AuthResult::Success {
            unknown_1: 3,
            unknown_2: 0,
        }
    }

    pub fn error(code: AuthResultError) -> Self {
        AuthResult::Failure { code }
    }
}

#[derive(Clone, ByteSize, Deserialize, Serialize, Packet, Debug)]
#[packet(opcode = 0x6103)]
pub struct AuthRequest {
    pub token: u32,
    pub username: String,
    pub password: String,
    pub unknown: u8,
    pub mac_bytes: [u8; 6],
}

#[derive(Clone, Serialize, ByteSize, Deserialize, Packet, Debug)]
#[packet(opcode = 0xA103)]
pub struct AuthResponse {
    pub result: AuthResult,
}

impl AuthResponse {
    pub fn new(result: AuthResult) -> Self {
        AuthResponse { result }
    }
}

#[derive(Clone, Deserialize, ByteSize, Serialize, Packet, Debug)]
#[packet(opcode = 0x7005)]
pub struct LogoutRequest {
    pub mode: LogoutMode,
}

#[derive(Clone, Serialize, ByteSize, Deserialize, Packet, Debug)]
#[packet(opcode = 0xB005)]
pub struct LogoutResponse {
    pub result: LogoutResult,
}

impl LogoutResponse {
    pub fn new(result: LogoutResult) -> Self {
        LogoutResponse { result }
    }
}

#[derive(Clone, Serialize, ByteSize, Deserialize, Packet, Debug)]
#[packet(opcode = 0x300A)]
pub struct LogoutFinished;

#[derive(Clone, Serialize, ByteSize, Deserialize, Packet, Debug)]
#[packet(opcode = 0x2212)]
pub struct Disconnect {
    pub unknown: u8,
}

impl Disconnect {
    pub fn new() -> Self {
        Disconnect { unknown: 0xFF }
    }
}

define_protocol! { AuthProtocol =>
    LogoutRequest,
    LogoutResponse,
    LogoutFinished,
    AuthRequest,
    AuthResponse,
    Disconnect
}
