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
pub enum CharacterListAction {
    Create,
    List,
    Delete,
    CheckName,
    Restore,
    ShowJobSpread,
    AssignJob,
}

#[derive(Clone, PartialEq, PartialOrd)]
pub enum CharacterListError {
    CloudntCreateCharacter,
    WeaponRequired,
    TooManyCharacters,
    NameTooLong,
    InvalidName,
    NameAlreadyUsed,
    ConnectionOverlay,
    ReachedCapacity,
}

#[derive(Clone)]
pub enum CharacterListContent {
    Characters {
        characters: Vec<CharacterListEntry>,
        job: u8,
    }
    ,
    Empty,
    JobSpread {
        hunters: u8,
        thieves: u8,
    }
    ,
}

#[derive(Clone)]
pub enum CharacterListResult {
    Ok {
        content: CharacterListContent,
    }
    ,
    Error {
        error: CharacterListError,
    }
    ,
}

#[derive(Clone)]
pub enum CharacterListRequestAction {
    Create {
        character_name: String,
        ref_id: u32,
        scale: u8,
        chest: u32,
        pants: u32,
        boots: u32,
        weapon: u32,
    }
    ,
    List,
    Delete {
        character_name: String,
    }
    ,
    CheckName {
        character_name: String,
    }
    ,
    Restore {
        character_name: String,
    }
    ,
    ShowJobSpread,
    AssignJob {
        job: u8,
    }
    ,
}

#[derive(Clone)]
pub enum CharacterJoinResult {
    Success,
    Error {
        error: CharacterListError,
    }
    ,
}

#[derive(Clone)]
pub struct CharacterListEquippedItem {
    pub id: u32,
    pub upgrade_level: u8,
}

#[derive(Clone)]
pub struct CharacterListAvatarItem {
    pub id: u32,
}

#[derive(Clone)]
pub struct CharacterListEntry {
    pub ref_id: u32,
    pub name: String,
    pub unknown_1: u8,
    pub unknown_2: u8,
    pub scale: u8,
    pub level: u8,
    pub exp: u64,
    pub strength: u16,
    pub intelligence: u16,
    pub stat_points: u16,
    pub sp: u32,
    pub hp: u32,
    pub mp: u32,
    pub region: u16,
    pub remaining_deletion_time: Option<u32>,
    pub last_logout: u32,
    pub guild_member_class: u8,
    pub guild_rename_required: Option<String>,
    pub academy_member_class: u8,
    pub equipped_items: Vec<CharacterListEquippedItem>,
    pub avatar_items: Vec<CharacterListAvatarItem>,
}

#[derive(Clone)]
pub struct CharacterListResponse {
    pub action: CharacterListAction,
    pub result: CharacterListResult,
}

impl From<CharacterListResponse> for Bytes {
    fn from(op: CharacterListResponse) -> Bytes {
        let mut data_writer = BytesMut::new();
        match &op.action {
            CharacterListAction::Create => data_writer.put_u8(1),
            CharacterListAction::List => data_writer.put_u8(2),
            CharacterListAction::Delete => data_writer.put_u8(3),
            CharacterListAction::CheckName => data_writer.put_u8(4),
            CharacterListAction::Restore => data_writer.put_u8(5),
            CharacterListAction::ShowJobSpread => data_writer.put_u8(9),
            CharacterListAction::AssignJob => data_writer.put_u8(0x10),
        }
        match &op.result {
            CharacterListResult::Ok { content,  } => {
                data_writer.put_u8(1);
                match &content {
                    CharacterListContent::Characters { characters, job,  } => {
                        data_writer.put_u8(characters.len() as u8);
                        for element in characters.iter() {
                            data_writer.put_u32_le(element.ref_id);
                            data_writer.put_u16_le(element.name.len() as u16);
                            data_writer.put_slice(element.name.as_bytes());
                            data_writer.put_u8(element.unknown_1);
                            data_writer.put_u8(element.unknown_2);
                            data_writer.put_u8(element.scale);
                            data_writer.put_u8(element.level);
                            data_writer.put_u64_le(element.exp);
                            data_writer.put_u16_le(element.strength);
                            data_writer.put_u16_le(element.intelligence);
                            data_writer.put_u16_le(element.stat_points);
                            data_writer.put_u32_le(element.sp);
                            data_writer.put_u32_le(element.hp);
                            data_writer.put_u32_le(element.mp);
                            data_writer.put_u16_le(element.region);
                            if let Some(remaining_deletion_time) = &element.remaining_deletion_time {
                                data_writer.put_u8(1);
                                data_writer.put_u32_le(*remaining_deletion_time);
                            }
                            else {
                                data_writer.put_u8(0);
                            }
                            data_writer.put_u32_le(element.last_logout);
                            data_writer.put_u8(element.guild_member_class);
                            if let Some(guild_rename_required) = &element.guild_rename_required {
                                data_writer.put_u8(1);
                                data_writer.put_u16_le(guild_rename_required.len() as u16);
                                data_writer.put_slice(guild_rename_required.as_bytes());
                            }
                            else {
                                data_writer.put_u8(0);
                            }
                            data_writer.put_u8(element.academy_member_class);
                            data_writer.put_u8(element.equipped_items.len() as u8);
                            for element in element.equipped_items.iter() {
                                data_writer.put_u32_le(element.id);
                                data_writer.put_u8(element.upgrade_level);
                            }
                            data_writer.put_u8(element.avatar_items.len() as u8);
                            for element in element.avatar_items.iter() {
                                data_writer.put_u32_le(element.id);
                            }
                        }
                        data_writer.put_u8(*job);
                    }
                    CharacterListContent::Empty => {},
                    CharacterListContent::JobSpread { hunters, thieves,  } => {
                        data_writer.put_u8(*hunters);
                        data_writer.put_u8(*thieves);
                    }
                }
            }
            CharacterListResult::Error { error,  } => {
                data_writer.put_u8(2);
                match &error {
                    CharacterListError::CloudntCreateCharacter => data_writer.put_u16_le(0x403),
                    CharacterListError::WeaponRequired => data_writer.put_u16_le(0x404),
                    CharacterListError::TooManyCharacters => data_writer.put_u16_le(0x405),
                    CharacterListError::NameTooLong => data_writer.put_u16_le(0x40C),
                    CharacterListError::InvalidName => data_writer.put_u16_le(0x40D),
                    CharacterListError::NameAlreadyUsed => data_writer.put_u16_le(0x410),
                    CharacterListError::ConnectionOverlay => data_writer.put_u16_le(0x411),
                    CharacterListError::ReachedCapacity => data_writer.put_u16_le(0x414),
                }
            }
        }
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for CharacterListResponse {
    fn into(self) -> ServerPacket {
        ServerPacket::CharacterListResponse(self)
    }
}

impl CharacterListResponse {
    pub fn new(action: CharacterListAction, result: CharacterListResult) -> Self {
        CharacterListResponse { action, result,  }
    }
}

#[derive(Clone)]
pub struct CharacterListRequest {
    pub action: CharacterListRequestAction,
}

impl TryFrom<Bytes> for CharacterListRequest {
    type Error = ProtocolError;

    fn try_from(data: Bytes) -> Result<Self, Self::Error> {
        let mut data_reader = data.reader();
        let action = match data_reader.read_u8()? {
            1 =>  {
                let character_name_string_len = data_reader.read_u16::<byteorder::LittleEndian>()?;
                let mut character_name_bytes = Vec::with_capacity(character_name_string_len as usize);
                for _ in 0..character_name_string_len {
                    	character_name_bytes.push(data_reader.read_u8()?);
                }
                let character_name = String::from_utf8(character_name_bytes)?;
                let ref_id = data_reader.read_u32::<byteorder::LittleEndian>()?;
                let scale = data_reader.read_u8()?;
                let chest = data_reader.read_u32::<byteorder::LittleEndian>()?;
                let pants = data_reader.read_u32::<byteorder::LittleEndian>()?;
                let boots = data_reader.read_u32::<byteorder::LittleEndian>()?;
                let weapon = data_reader.read_u32::<byteorder::LittleEndian>()?;
                CharacterListRequestAction::Create { character_name, ref_id, scale, chest, pants, boots, weapon,  }
            }
            2 => CharacterListRequestAction::List,
            3 =>  {
                let character_name_string_len = data_reader.read_u16::<byteorder::LittleEndian>()?;
                let mut character_name_bytes = Vec::with_capacity(character_name_string_len as usize);
                for _ in 0..character_name_string_len {
                    	character_name_bytes.push(data_reader.read_u8()?);
                }
                let character_name = String::from_utf8(character_name_bytes)?;
                CharacterListRequestAction::Delete { character_name,  }
            }
            4 =>  {
                let character_name_string_len = data_reader.read_u16::<byteorder::LittleEndian>()?;
                let mut character_name_bytes = Vec::with_capacity(character_name_string_len as usize);
                for _ in 0..character_name_string_len {
                    	character_name_bytes.push(data_reader.read_u8()?);
                }
                let character_name = String::from_utf8(character_name_bytes)?;
                CharacterListRequestAction::CheckName { character_name,  }
            }
            5 =>  {
                let character_name_string_len = data_reader.read_u16::<byteorder::LittleEndian>()?;
                let mut character_name_bytes = Vec::with_capacity(character_name_string_len as usize);
                for _ in 0..character_name_string_len {
                    	character_name_bytes.push(data_reader.read_u8()?);
                }
                let character_name = String::from_utf8(character_name_bytes)?;
                CharacterListRequestAction::Restore { character_name,  }
            }
            9 => CharacterListRequestAction::ShowJobSpread,
            0x10 =>  {
                let job = data_reader.read_u8()?;
                CharacterListRequestAction::AssignJob { job,  }
            }
            unknown => return Err(ProtocolError::UnknownVariation(unknown, "CharacterListRequestAction"))
        };
        Ok(CharacterListRequest { action,  })
    }
}

impl Into<ClientPacket> for CharacterListRequest {
    fn into(self) -> ClientPacket {
        ClientPacket::CharacterListRequest(self)
    }
}

#[derive(Clone)]
pub struct CharacterJoinRequest {
    pub character_name: String,
}

impl TryFrom<Bytes> for CharacterJoinRequest {
    type Error = ProtocolError;

    fn try_from(data: Bytes) -> Result<Self, Self::Error> {
        let mut data_reader = data.reader();
        let character_name_string_len = data_reader.read_u16::<byteorder::LittleEndian>()?;
        let mut character_name_bytes = Vec::with_capacity(character_name_string_len as usize);
        for _ in 0..character_name_string_len {
            	character_name_bytes.push(data_reader.read_u8()?);
        }
        let character_name = String::from_utf8(character_name_bytes)?;
        Ok(CharacterJoinRequest { character_name,  })
    }
}

impl Into<ClientPacket> for CharacterJoinRequest {
    fn into(self) -> ClientPacket {
        ClientPacket::CharacterJoinRequest(self)
    }
}

#[derive(Clone)]
pub struct CharacterJoinResponse {
    pub result: CharacterJoinResult,
}

impl From<CharacterJoinResponse> for Bytes {
    fn from(op: CharacterJoinResponse) -> Bytes {
        let mut data_writer = BytesMut::new();
        match &op.result {
            CharacterJoinResult::Success => data_writer.put_u8(1),
            CharacterJoinResult::Error { error,  } => {
                data_writer.put_u8(2);
                match &error {
                    CharacterListError::CloudntCreateCharacter => data_writer.put_u16_le(0x403),
                    CharacterListError::WeaponRequired => data_writer.put_u16_le(0x404),
                    CharacterListError::TooManyCharacters => data_writer.put_u16_le(0x405),
                    CharacterListError::NameTooLong => data_writer.put_u16_le(0x40C),
                    CharacterListError::InvalidName => data_writer.put_u16_le(0x40D),
                    CharacterListError::NameAlreadyUsed => data_writer.put_u16_le(0x410),
                    CharacterListError::ConnectionOverlay => data_writer.put_u16_le(0x411),
                    CharacterListError::ReachedCapacity => data_writer.put_u16_le(0x414),
                }
            }
        }
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for CharacterJoinResponse {
    fn into(self) -> ServerPacket {
        ServerPacket::CharacterJoinResponse(self)
    }
}

impl CharacterJoinResponse {
    pub fn new(result: CharacterJoinResult) -> Self {
        CharacterJoinResponse { result,  }
    }
}

#[derive(Clone)]
pub struct CharacterStatsMessage {
    pub phys_attack_min: u32,
    pub phys_attack_max: u32,
    pub mag_attack_min: u32,
    pub mag_attack_max: u32,
    pub phys_defense: u16,
    pub mag_defense: u16,
    pub hit_rate: u16,
    pub parry_rate: u16,
    pub max_hp: u32,
    pub max_mp: u32,
    pub strength: u16,
    pub intelligence: u16,
}

impl From<CharacterStatsMessage> for Bytes {
    fn from(op: CharacterStatsMessage) -> Bytes {
        let mut data_writer = BytesMut::new();
        data_writer.put_u32_le(op.phys_attack_min);
        data_writer.put_u32_le(op.phys_attack_max);
        data_writer.put_u32_le(op.mag_attack_min);
        data_writer.put_u32_le(op.mag_attack_max);
        data_writer.put_u16_le(op.phys_defense);
        data_writer.put_u16_le(op.mag_defense);
        data_writer.put_u16_le(op.hit_rate);
        data_writer.put_u16_le(op.parry_rate);
        data_writer.put_u32_le(op.max_hp);
        data_writer.put_u32_le(op.max_mp);
        data_writer.put_u16_le(op.strength);
        data_writer.put_u16_le(op.intelligence);
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for CharacterStatsMessage {
    fn into(self) -> ServerPacket {
        ServerPacket::CharacterStatsMessage(self)
    }
}

impl CharacterStatsMessage {
    pub fn new(phys_attack_min: u32, phys_attack_max: u32, mag_attack_min: u32, mag_attack_max: u32, phys_defense: u16, mag_defense: u16, hit_rate: u16, parry_rate: u16, max_hp: u32, max_mp: u32, strength: u16, intelligence: u16) -> Self {
        CharacterStatsMessage { phys_attack_min, phys_attack_max, mag_attack_min, mag_attack_max, phys_defense, mag_defense, hit_rate, parry_rate, max_hp, max_mp, strength, intelligence,  }
    }
}

#[derive(Clone)]
pub struct UnknownPacket {
    pub unknown_2: u32,
    pub unknown_1: u8,
}

impl From<UnknownPacket> for Bytes {
    fn from(op: UnknownPacket) -> Bytes {
        let mut data_writer = BytesMut::new();
        data_writer.put_u32_le(op.unknown_2);
        data_writer.put_u8(op.unknown_1);
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for UnknownPacket {
    fn into(self) -> ServerPacket {
        ServerPacket::UnknownPacket(self)
    }
}

impl UnknownPacket {
    pub fn new() -> Self {
        UnknownPacket { unknown_2: 4, unknown_1: 0,  }
    }
}

#[derive(Clone)]
pub struct UnknownPacket2 {
    pub unknown_1: u8,
    pub id: u32,
    pub unknown_2: u32,
}

impl From<UnknownPacket2> for Bytes {
    fn from(op: UnknownPacket2) -> Bytes {
        let mut data_writer = BytesMut::new();
        data_writer.put_u8(op.unknown_1);
        data_writer.put_u32_le(op.id);
        data_writer.put_u32_le(op.unknown_2);
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for UnknownPacket2 {
    fn into(self) -> ServerPacket {
        ServerPacket::UnknownPacket2(self)
    }
}

impl UnknownPacket2 {
    pub fn new(id: u32) -> Self {
        UnknownPacket2 { unknown_1: 1, id, unknown_2: 0,  }
    }
}

#[derive(Clone)]
pub struct FinishLoading;

impl TryFrom<Bytes> for FinishLoading {
    type Error = ProtocolError;

    fn try_from(data: Bytes) -> Result<Self, Self::Error> {
        let mut data_reader = data.reader();
        Ok(FinishLoading {  })
    }
}

impl Into<ClientPacket> for FinishLoading {
    fn into(self) -> ClientPacket {
        ClientPacket::FinishLoading(self)
    }
}