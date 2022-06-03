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

#[derive(Clone)]
pub enum ChatSource {
    All,
    AllGm,
    NPC,
    PrivateMessage { receiver: String },
    Party,
    Guild,
    Global,
    Stall,
    Union,
    Academy,
    Notice,
}

impl ChatSource {
    pub fn privatemessage(receiver: String) -> Self {
        ChatSource::PrivateMessage { receiver }
    }
}

impl Size for ChatSource {
    fn calculate_size(&self) -> usize {
        std::mem::size_of::<u8>()
            + match &self {
                ChatSource::All => 0,
                ChatSource::AllGm => 0,
                ChatSource::NPC => 0,
                ChatSource::PrivateMessage { receiver } => receiver.calculate_size(),
                ChatSource::Party => 0,
                ChatSource::Guild => 0,
                ChatSource::Global => 0,
                ChatSource::Stall => 0,
                ChatSource::Union => 0,
                ChatSource::Academy => 0,
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
            ChatSource::All => data_writer.put_u8(1),
            ChatSource::AllGm => data_writer.put_u8(3),
            ChatSource::NPC => data_writer.put_u8(13),
            ChatSource::PrivateMessage { receiver } => {
                data_writer.put_u8(2);
                data_writer.put_u16_le(receiver.len() as u16);
                data_writer.put_slice(receiver.as_bytes());
            },
            ChatSource::Party => data_writer.put_u8(4),
            ChatSource::Guild => data_writer.put_u8(5),
            ChatSource::Global => data_writer.put_u8(6),
            ChatSource::Stall => data_writer.put_u8(9),
            ChatSource::Union => data_writer.put_u8(11),
            ChatSource::Academy => data_writer.put_u8(16),
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
    pub target: ChatSource,
    pub index: u8,
    pub unknown_2: u16,
    pub message: String,
}

impl TryFrom<Bytes> for ChatMessage {
    type Error = ProtocolError;

    fn try_from(data: Bytes) -> Result<Self, Self::Error> {
        let mut data_reader = data.reader();
        let target = match data_reader.read_u8()? {
            1 => ChatSource::All,
            3 => ChatSource::AllGm,
            13 => ChatSource::NPC,
            2 => {
                let receiver_string_len = data_reader.read_u16::<byteorder::LittleEndian>()?;
                let mut receiver_bytes = Vec::with_capacity(receiver_string_len as usize);
                for _ in 0..receiver_string_len {
                    receiver_bytes.push(data_reader.read_u8()?);
                }
                let receiver = String::from_utf8(receiver_bytes)?;
                ChatSource::PrivateMessage { receiver }
            },
            4 => ChatSource::Party,
            5 => ChatSource::Guild,
            6 => ChatSource::Global,
            9 => ChatSource::Stall,
            11 => ChatSource::Union,
            16 => ChatSource::Academy,
            7 => ChatSource::Notice,
            unknown => return Err(ProtocolError::UnknownVariation(unknown, "ChatSource")),
        };
        let index = data_reader.read_u8()?;
        let unknown_2 = data_reader.read_u16::<byteorder::LittleEndian>()?;
        let message_string_len = data_reader.read_u16::<byteorder::LittleEndian>()?;
        let mut message_bytes = Vec::with_capacity(message_string_len as usize);
        for _ in 0..message_string_len {
            message_bytes.push(data_reader.read_u16::<byteorder::LittleEndian>()?);
        }
        let message = String::from_utf16(&message_bytes)?;
        Ok(ChatMessage {
            target,
            index,
            unknown_2,
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
    pub source: ChatSource,
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
        match &op.source {
            ChatSource::All => data_writer.put_u8(1),
            ChatSource::AllGm => data_writer.put_u8(3),
            ChatSource::NPC => data_writer.put_u8(13),
            ChatSource::PrivateMessage { receiver } => {
                data_writer.put_u8(2);
                data_writer.put_u16_le(receiver.len() as u16);
                data_writer.put_slice(receiver.as_bytes());
            },
            ChatSource::Party => data_writer.put_u8(4),
            ChatSource::Guild => data_writer.put_u8(5),
            ChatSource::Global => data_writer.put_u8(6),
            ChatSource::Stall => data_writer.put_u8(9),
            ChatSource::Union => data_writer.put_u8(11),
            ChatSource::Academy => data_writer.put_u8(16),
            ChatSource::Notice => data_writer.put_u8(7),
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
    pub fn new(result: ChatMessageResult, source: ChatSource, index: u8) -> Self {
        ChatMessageResponse { result, source, index }
    }
}

impl Size for ChatMessageResponse {
    fn calculate_size(&self) -> usize {
        self.result.calculate_size() + self.source.calculate_size() + self.index.calculate_size()
    }
}
