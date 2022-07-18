use silkroad_serde::*;
use silkroad_serde_derive::*;

#[derive(Clone, PartialEq, PartialOrd, Copy, Serialize, ByteSize, Deserialize)]
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

#[derive(Clone, ByteSize, Serialize)]
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
}

#[derive(Clone, Serialize, ByteSize)]
pub enum ChatMessageResult {
    #[silkroad(value = 1)]
    Success,
    #[silkroad(value = 2)]
    Error { code: u16 },
}

impl ChatMessageResult {
    pub fn error(code: u16) -> Self {
        ChatMessageResult::Error { code }
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub struct TextCharacterInitialization {
    // TODO this should be raw
    pub characters: Vec<u64>,
}

impl TextCharacterInitialization {
    pub fn new(characters: Vec<u64>) -> Self {
        TextCharacterInitialization { characters }
    }
}

#[derive(Clone, Serialize, ByteSize)]
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

#[derive(Clone, Deserialize, ByteSize)]
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

#[derive(Clone, Serialize, ByteSize)]
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
