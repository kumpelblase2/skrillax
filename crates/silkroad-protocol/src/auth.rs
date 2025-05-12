use skrillax_packet::Packet;
use skrillax_serde::*;
use skrillax_stream::registry::PacketRegistryBuilder;

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

impl Default for Disconnect {
    fn default() -> Self {
        Disconnect::new()
    }
}

impl Disconnect {
    pub fn new() -> Self {
        Disconnect { unknown: 0xFF }
    }
}

#[derive(Clone, Deserialize, Serialize, ByteSize, Debug)]
struct UnknownPacketEntry2 {
    kind: u32,
    data: u64,
}

#[derive(Clone, Deserialize, Serialize, ByteSize, Debug)]
struct UnknownPacketEntry {
    entry_id: u32,
    unknown_flag: u8,
    elements: Vec<UnknownPacketEntry2>,
}

#[derive(Clone, Deserialize, Serialize, ByteSize, Debug, Packet)]
#[packet(opcode = 0x3612)]
pub struct UnknownLargePacket {
    unknown: u8,
    entries: Vec<UnknownPacketEntry>,
}

impl UnknownLargePacket {
    pub fn new() -> Self {
        UnknownLargePacket {
            unknown: 0,
            entries: vec![],
        }
    }

    pub fn known() -> Self {
        UnknownLargePacket {
            unknown: 0,
            entries: vec![
                UnknownPacketEntry {
                    entry_id: 1,
                    unknown_flag: 1,
                    elements: vec![UnknownPacketEntry2 { kind: 18, data: 0 }],
                },
                UnknownPacketEntry {
                    entry_id: 2,
                    unknown_flag: 1,
                    elements: vec![UnknownPacketEntry2 { kind: 19, data: 0 }],
                },
                UnknownPacketEntry {
                    entry_id: 3,
                    unknown_flag: 1,
                    elements: vec![UnknownPacketEntry2 { kind: 1, data: 0 }],
                },
                UnknownPacketEntry {
                    entry_id: 4,
                    unknown_flag: 1,
                    elements: vec![UnknownPacketEntry2 { kind: 2, data: 0 }],
                },
                UnknownPacketEntry {
                    entry_id: 5,
                    unknown_flag: 1,
                    elements: vec![UnknownPacketEntry2 { kind: 12, data: 0 }],
                },
                UnknownPacketEntry {
                    entry_id: 6,
                    unknown_flag: 1,
                    elements: vec![UnknownPacketEntry2 { kind: 13, data: 0 }],
                },
                UnknownPacketEntry {
                    entry_id: 7,
                    unknown_flag: 1,
                    elements: vec![UnknownPacketEntry2 { kind: 14, data: 0 }],
                },
                UnknownPacketEntry {
                    entry_id: 8,
                    unknown_flag: 1,
                    elements: vec![UnknownPacketEntry2 { kind: 15, data: 0 }],
                },
                UnknownPacketEntry {
                    entry_id: 9,
                    unknown_flag: 1,
                    elements: vec![UnknownPacketEntry2 { kind: 16, data: 0 }],
                },
                UnknownPacketEntry {
                    entry_id: 10,
                    unknown_flag: 1,
                    elements: vec![UnknownPacketEntry2 { kind: 17, data: 0 }],
                },
                UnknownPacketEntry {
                    entry_id: 11,
                    unknown_flag: 1,
                    elements: vec![UnknownPacketEntry2 { kind: 24, data: 0 }],
                },
                UnknownPacketEntry {
                    entry_id: 23,
                    unknown_flag: 1,
                    elements: vec![UnknownPacketEntry2 { kind: 3, data: 0 }],
                },
                UnknownPacketEntry {
                    entry_id: 24,
                    unknown_flag: 1,
                    elements: vec![UnknownPacketEntry2 { kind: 4, data: 0 }],
                },
                UnknownPacketEntry {
                    entry_id: 25,
                    unknown_flag: 1,
                    elements: vec![UnknownPacketEntry2 { kind: 5, data: 0 }],
                },
                UnknownPacketEntry {
                    entry_id: 26,
                    unknown_flag: 1,
                    elements: vec![UnknownPacketEntry2 { kind: 6, data: 0 }],
                },
                UnknownPacketEntry {
                    entry_id: 27,
                    unknown_flag: 1,
                    elements: vec![UnknownPacketEntry2 { kind: 7, data: 0 }],
                },
                UnknownPacketEntry {
                    entry_id: 28,
                    unknown_flag: 1,
                    elements: vec![UnknownPacketEntry2 { kind: 8, data: 0 }],
                },
                UnknownPacketEntry {
                    entry_id: 29,
                    unknown_flag: 1,
                    elements: vec![UnknownPacketEntry2 { kind: 9, data: 0 }],
                },
                UnknownPacketEntry {
                    entry_id: 30,
                    unknown_flag: 1,
                    elements: vec![UnknownPacketEntry2 { kind: 10, data: 0 }],
                },
                UnknownPacketEntry {
                    entry_id: 31,
                    unknown_flag: 1,
                    elements: vec![UnknownPacketEntry2 { kind: 11, data: 0 }],
                },
                UnknownPacketEntry {
                    entry_id: 91,
                    unknown_flag: 1,
                    elements: vec![
                        UnknownPacketEntry2 { kind: 119, data: 0 },
                        UnknownPacketEntry2 { kind: 120, data: 0 },
                        UnknownPacketEntry2 { kind: 121, data: 0 },
                        UnknownPacketEntry2 { kind: 122, data: 0 },
                        UnknownPacketEntry2 { kind: 123, data: 0 },
                    ],
                },
                UnknownPacketEntry {
                    entry_id: 92,
                    unknown_flag: 1,
                    elements: vec![
                        UnknownPacketEntry2 { kind: 124, data: 0 },
                        UnknownPacketEntry2 { kind: 125, data: 0 },
                        UnknownPacketEntry2 { kind: 126, data: 0 },
                        UnknownPacketEntry2 { kind: 127, data: 0 },
                        UnknownPacketEntry2 { kind: 128, data: 0 },
                    ],
                },
            ],
        }
    }
}

pub trait AuthPacketRegistryExt {
    fn register_auth_packets(self) -> Self;
}

impl AuthPacketRegistryExt for PacketRegistryBuilder {
    fn register_auth_packets(self) -> Self {
        self.register_incoming::<AuthRequest>()
            .register_outgoing::<AuthResponse>()
            .register_incoming::<LogoutRequest>()
            .register_outgoing::<LogoutResponse>()
            .register_outgoing::<LogoutFinished>()
            .register_outgoing::<Disconnect>()
            .register::<UnknownLargePacket>()
    }
}
