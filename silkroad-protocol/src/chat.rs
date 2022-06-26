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
pub enum ChatTarget {
    All,
    AllGm,
    NPC,
    PrivateMessage,
    Party,
    Guild,
    Global,
    Stall,
    Union,
    Academy,
    Notice,
}

impl Size for ChatTarget {
    fn calculate_size(&self) -> usize {
        std::mem::size_of::<u8>()
    }
}

#[derive(Clone)]
pub enum ChatSource {
    All { sender: u32 },
    AllGm { sender: u32 },
    NPC { sender: u32 },
    PrivateMessage { sender: String },
    Party { sender: String },
    Guild { sender: String },
    Global { sender: String },
    Stall { sender: String },
    Union { sender: String },
    Academy { sender: String },
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

impl Size for ChatSource {
    fn calculate_size(&self) -> usize {
        std::mem::size_of::<u8>()
            + match &self {
                ChatSource::All { sender } => sender.calculate_size(),
                ChatSource::AllGm { sender } => sender.calculate_size(),
                ChatSource::NPC { sender } => sender.calculate_size(),
                ChatSource::PrivateMessage { sender } => sender.calculate_size(),
                ChatSource::Party { sender } => sender.calculate_size(),
                ChatSource::Guild { sender } => sender.calculate_size(),
                ChatSource::Global { sender } => sender.calculate_size(),
                ChatSource::Stall { sender } => sender.calculate_size(),
                ChatSource::Union { sender } => sender.calculate_size(),
                ChatSource::Academy { sender } => sender.calculate_size(),
                ChatSource::Notice => 0,
            }
    }
}

#[derive(Clone)]
pub enum ChatMessageResult {
    Success,
    Error { code: u16 },
}

impl ChatMessageResult {
    pub fn error(code: u16) -> Self {
        ChatMessageResult::Error { code }
    }
}

impl Size for ChatMessageResult {
    fn calculate_size(&self) -> usize {
        std::mem::size_of::<u8>()
            + match &self {
                ChatMessageResult::Success => 0,
                ChatMessageResult::Error { code } => code.calculate_size(),
            }
    }
}

#[derive(Clone)]
pub struct TextCharacterInitialization {
    pub characters: Vec<u64>,
}

impl From<TextCharacterInitialization> for Bytes {
    fn from(op: TextCharacterInitialization) -> Bytes {
        let mut data_writer = BytesMut::with_capacity(op.calculate_size());
        data_writer.put_u8(op.characters.len() as u8);
        for element in op.characters.iter() {
            data_writer.put_u64_le(*element);
        }
        data_writer.freeze()
    }
}

impl From<TextCharacterInitialization> for ServerPacket {
    fn from(other: TextCharacterInitialization) -> Self {
        ServerPacket::TextCharacterInitialization(other)
    }
}

impl TextCharacterInitialization {
    pub fn new(characters: Vec<u64>) -> Self {
        TextCharacterInitialization { characters }
    }
}

impl Size for TextCharacterInitialization {
    fn calculate_size(&self) -> usize {
        2 + self
            .characters
            .iter()
            .map(|inner| inner.calculate_size())
            .sum::<usize>()
    }
}

#[derive(Clone)]
pub struct ChatUpdate {
    pub source: ChatSource,
    pub message: String,
}

impl From<ChatUpdate> for Bytes {
    fn from(op: ChatUpdate) -> Bytes {
        let mut data_writer = BytesMut::with_capacity(op.calculate_size());
        match &op.source {
            ChatSource::All { sender } => {
                data_writer.put_u8(1);
                data_writer.put_u32_le(*sender);
            },
            ChatSource::AllGm { sender } => {
                data_writer.put_u8(3);
                data_writer.put_u32_le(*sender);
            },
            ChatSource::NPC { sender } => {
                data_writer.put_u8(13);
                data_writer.put_u32_le(*sender);
            },
            ChatSource::PrivateMessage { sender } => {
                data_writer.put_u8(2);
                data_writer.put_u16_le(sender.len() as u16);
                data_writer.put_slice(sender.as_bytes());
            },
            ChatSource::Party { sender } => {
                data_writer.put_u8(4);
                data_writer.put_u16_le(sender.len() as u16);
                data_writer.put_slice(sender.as_bytes());
            },
            ChatSource::Guild { sender } => {
                data_writer.put_u8(5);
                data_writer.put_u16_le(sender.len() as u16);
                data_writer.put_slice(sender.as_bytes());
            },
            ChatSource::Global { sender } => {
                data_writer.put_u8(6);
                data_writer.put_u16_le(sender.len() as u16);
                data_writer.put_slice(sender.as_bytes());
            },
            ChatSource::Stall { sender } => {
                data_writer.put_u8(9);
                data_writer.put_u16_le(sender.len() as u16);
                data_writer.put_slice(sender.as_bytes());
            },
            ChatSource::Union { sender } => {
                data_writer.put_u8(11);
                data_writer.put_u16_le(sender.len() as u16);
                data_writer.put_slice(sender.as_bytes());
            },
            ChatSource::Academy { sender } => {
                data_writer.put_u8(16);
                data_writer.put_u16_le(sender.len() as u16);
                data_writer.put_slice(sender.as_bytes());
            },
            ChatSource::Notice => data_writer.put_u8(7),
        }
        data_writer.put_u16_le(op.message.len() as u16);
        for utf_char in op.message.encode_utf16() {
            data_writer.put_u16_le(utf_char);
        }
        data_writer.freeze()
    }
}

impl From<ChatUpdate> for ServerPacket {
    fn from(other: ChatUpdate) -> Self {
        ServerPacket::ChatUpdate(other)
    }
}

impl ChatUpdate {
    pub fn new(source: ChatSource, message: String) -> Self {
        ChatUpdate { source, message }
    }
}

impl Size for ChatUpdate {
    fn calculate_size(&self) -> usize {
        self.source.calculate_size() + self.message.calculate_size()
    }
}

#[derive(Clone)]
pub struct ChatMessage {
    pub target: ChatTarget,
    pub index: u8,
    pub contains_link: bool,
    pub unknown: u8,
    pub recipient: Option<String>,
    pub message: String,
}

impl TryFrom<Bytes> for ChatMessage {
    type Error = ProtocolError;

    fn try_from(data: Bytes) -> Result<Self, Self::Error> {
        let mut data_reader = data.reader();
        let target = match data_reader.read_u8()? {
            1 => ChatTarget::All,
            3 => ChatTarget::AllGm,
            13 => ChatTarget::NPC,
            2 => ChatTarget::PrivateMessage,
            4 => ChatTarget::Party,
            5 => ChatTarget::Guild,
            6 => ChatTarget::Global,
            9 => ChatTarget::Stall,
            11 => ChatTarget::Union,
            16 => ChatTarget::Academy,
            7 => ChatTarget::Notice,
            unknown => return Err(ProtocolError::UnknownVariation(unknown, "ChatTarget")),
        };
        let index = data_reader.read_u8()?;
        let contains_link = data_reader.read_u8()? == 1;
        let unknown = data_reader.read_u8()?;
        let recipient = if matches!(target, ChatTarget::PrivateMessage) {
            let inner_string_len = data_reader.read_u16::<byteorder::LittleEndian>()?;
            let mut inner_bytes = Vec::with_capacity(inner_string_len as usize);
            for _ in 0..inner_string_len {
                inner_bytes.push(data_reader.read_u8()?);
            }
            let inner = String::from_utf8(inner_bytes)?;
            Some(inner)
        } else {
            None
        };
        let message_string_len = data_reader.read_u16::<byteorder::LittleEndian>()?;
        let mut message_bytes = Vec::with_capacity(message_string_len as usize);
        for _ in 0..message_string_len {
            message_bytes.push(data_reader.read_u16::<byteorder::LittleEndian>()?);
        }
        let message = String::from_utf16(&message_bytes)?;
        Ok(ChatMessage {
            target,
            index,
            contains_link,
            unknown,
            recipient,
            message,
        })
    }
}

impl From<ChatMessage> for ClientPacket {
    fn from(other: ChatMessage) -> Self {
        ClientPacket::ChatMessage(other)
    }
}

#[derive(Clone)]
pub struct ChatMessageResponse {
    pub result: ChatMessageResult,
    pub target: ChatTarget,
    pub index: u8,
}

impl From<ChatMessageResponse> for Bytes {
    fn from(op: ChatMessageResponse) -> Bytes {
        let mut data_writer = BytesMut::with_capacity(op.calculate_size());
        match &op.result {
            ChatMessageResult::Success => data_writer.put_u8(1),
            ChatMessageResult::Error { code } => {
                data_writer.put_u8(2);
                data_writer.put_u16_le(*code);
            },
        }
        match &op.target {
            ChatTarget::All => data_writer.put_u8(1),
            ChatTarget::AllGm => data_writer.put_u8(3),
            ChatTarget::NPC => data_writer.put_u8(13),
            ChatTarget::PrivateMessage => data_writer.put_u8(2),
            ChatTarget::Party => data_writer.put_u8(4),
            ChatTarget::Guild => data_writer.put_u8(5),
            ChatTarget::Global => data_writer.put_u8(6),
            ChatTarget::Stall => data_writer.put_u8(9),
            ChatTarget::Union => data_writer.put_u8(11),
            ChatTarget::Academy => data_writer.put_u8(16),
            ChatTarget::Notice => data_writer.put_u8(7),
        }
        data_writer.put_u8(op.index);
        data_writer.freeze()
    }
}

impl From<ChatMessageResponse> for ServerPacket {
    fn from(other: ChatMessageResponse) -> Self {
        ServerPacket::ChatMessageResponse(other)
    }
}

impl ChatMessageResponse {
    pub fn new(result: ChatMessageResult, target: ChatTarget, index: u8) -> Self {
        ChatMessageResponse { result, target, index }
    }
}

impl Size for ChatMessageResponse {
    fn calculate_size(&self) -> usize {
        self.result.calculate_size() + self.target.calculate_size() + self.index.calculate_size()
    }
}
