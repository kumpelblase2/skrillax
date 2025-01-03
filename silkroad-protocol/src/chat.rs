use skrillax_packet::Packet;
use skrillax_protocol::{define_inbound_protocol, define_outbound_protocol};
use skrillax_serde::*;

#[derive(Clone, Eq, PartialEq, PartialOrd, Copy, Debug, Serialize, ByteSize, Deserialize)]
pub enum ChatTarget {
    #[silkroad(value = 1)]
    All,
    #[silkroad(value = 3)]
    AllGm,
    #[silkroad(value = 13)]
    NPC,
    #[silkroad(value = 2)]
    PrivateMessage,
    #[silkroad(value = 4)]
    Party,
    #[silkroad(value = 5)]
    Guild,
    #[silkroad(value = 6)]
    Global,
    #[silkroad(value = 9)]
    Stall,
    #[silkroad(value = 11)]
    Union,
    #[silkroad(value = 16)]
    Academy,
    #[silkroad(value = 7)]
    Notice,
}

#[derive(Clone, ByteSize, Serialize, Deserialize, Debug)]
pub enum ChatSource {
    #[silkroad(value = 1)]
    All { sender: u32 },
    #[silkroad(value = 3)]
    AllGm { sender: u32 },
    #[silkroad(value = 13)]
    NPC { sender: u32 },
    #[silkroad(value = 2)]
    PrivateMessage { sender: String },
    #[silkroad(value = 4)]
    Party { sender: String },
    #[silkroad(value = 5)]
    Guild { sender: String },
    #[silkroad(value = 6)]
    Global { sender: String },
    #[silkroad(value = 9)]
    Stall { sender: String },
    #[silkroad(value = 11)]
    Union { sender: String },
    #[silkroad(value = 16)]
    Academy { sender: String },
    #[silkroad(value = 7)]
    Notice,
}

impl ChatSource {
    pub fn all(sender: u32) -> Self {
        ChatSource::All { sender }
    }

    pub fn allgm(sender: u32) -> Self {
        ChatSource::AllGm { sender }
    }

    pub fn npc(sender: u32) -> Self {
        ChatSource::NPC { sender }
    }

    pub fn privatemessage(sender: String) -> Self {
        ChatSource::PrivateMessage { sender }
    }

    pub fn party(sender: String) -> Self {
        ChatSource::Party { sender }
    }

    pub fn guild(sender: String) -> Self {
        ChatSource::Guild { sender }
    }

    pub fn global(sender: String) -> Self {
        ChatSource::Global { sender }
    }

    pub fn stall(sender: String) -> Self {
        ChatSource::Stall { sender }
    }

    pub fn union(sender: String) -> Self {
        ChatSource::Union { sender }
    }

    pub fn academy(sender: String) -> Self {
        ChatSource::Academy { sender }
    }

    pub fn system() -> Self {
        ChatSource::Global {
            sender: String::from("System"),
        }
    }
}

#[derive(Copy, Clone, Serialize, ByteSize, Deserialize, Debug)]
#[silkroad(size = 2)]
pub enum ChatErrorCode {
    #[silkroad(value = 3)]
    InvalidTarget,
    #[silkroad(value = 0x2006)]
    WhisperMuted,
    #[silkroad(value = 0x2008)]
    InvalidCommand,
}

#[derive(Clone, Copy, Serialize, ByteSize, Deserialize, Debug)]
pub enum ChatMessageResult {
    #[silkroad(value = 1)]
    Success,
    #[silkroad(value = 2)]
    Failure { code: ChatErrorCode },
}

impl ChatMessageResult {
    pub fn error(code: ChatErrorCode) -> Self {
        ChatMessageResult::Failure { code }
    }
}

#[derive(Clone, Deserialize, Serialize, ByteSize, Packet, Debug)]
#[packet(opcode = 0x3535)]
pub struct TextCharacterInitialization {
    // TODO this should be raw
    pub characters: Vec<u64>,
}

impl TextCharacterInitialization {
    pub fn new(characters: Vec<u64>) -> Self {
        TextCharacterInitialization { characters }
    }
}

#[derive(Clone, Serialize, ByteSize, Packet, Deserialize, Debug)]
#[packet(opcode = 0x3026)]
pub struct ChatUpdate {
    pub source: ChatSource,
    #[silkroad(size = 2)]
    pub message: String,
}

impl ChatUpdate {
    pub fn new(source: ChatSource, message: String) -> Self {
        ChatUpdate { source, message }
    }
}

#[derive(Clone, Deserialize, ByteSize, Serialize, Packet, Debug)]
#[packet(opcode = 0x7025)]
pub struct ChatMessage {
    pub target: ChatTarget,
    pub index: u8,
    pub contains_link: bool,
    pub unknown: u8,
    #[silkroad(when = "matches!(target, ChatTarget::PrivateMessage)")]
    pub recipient: Option<String>,
    #[silkroad(size = 2)]
    pub message: String,
}

#[derive(Clone, Copy, Deserialize, Serialize, ByteSize, Packet, Debug)]
#[packet(opcode = 0xB025)]
pub struct ChatMessageResponse {
    pub result: ChatMessageResult,
    pub target: ChatTarget,
    pub index: u8,
}

impl ChatMessageResponse {
    pub fn new(result: ChatMessageResult, target: ChatTarget, index: u8) -> Self {
        ChatMessageResponse { result, target, index }
    }
}

define_inbound_protocol! { ChatClientProtocol =>
    ChatMessage
}

define_outbound_protocol! { ChatServerProtocol =>
    ChatMessageResponse,
    ChatUpdate,
    TextCharacterInitialization
}
