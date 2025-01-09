use skrillax_packet::Packet;
use skrillax_protocol::{define_inbound_protocol, define_outbound_protocol};
use skrillax_serde::*;
use std::fmt::{Debug, Formatter};

#[derive(Clone, Eq, PartialEq, PartialOrd, Copy, Serialize, Deserialize, ByteSize, Debug)]
pub enum CharacterListAction {
    #[silkroad(value = 1)]
    Create,
    #[silkroad(value = 2)]
    List,
    #[silkroad(value = 3)]
    Delete,
    #[silkroad(value = 4)]
    CheckName,
    #[silkroad(value = 5)]
    Restore,
    #[silkroad(value = 9)]
    ShowJobSpread,
    #[silkroad(value = 0x10)]
    AssignJob,
}

#[derive(Clone, Eq, PartialEq, PartialOrd, Copy, Serialize, ByteSize, Debug)]
#[silkroad(size = 2)]
pub enum CharacterListError {
    #[silkroad(value = 0x403)]
    InvalidCharacterData,
    #[silkroad(value = 0x404)]
    WeaponRequired,
    #[silkroad(value = 0x405)]
    TooManyCharacters,
    #[silkroad(value = 0x406)]
    CouldntCreateCharacter,
    #[silkroad(value = 0x409)]
    UnknownGameserver,
    #[silkroad(value = 0x40C)]
    NameTooLong,
    #[silkroad(value = 0x40D)]
    InvalidName,
    #[silkroad(value = 0x410)]
    NameAlreadyUsed,
    #[silkroad(value = 0x411)]
    ConnectionOverlay,
    #[silkroad(value = 0x414)]
    ReachedCapacity,
    #[silkroad(value = 0x415)]
    FailedToJoinWorld,
    #[silkroad(value = 0x418)]
    CouldntConnectToServer,
}

#[derive(Clone, Serialize, ByteSize, Debug)]
#[silkroad(size = 0)]
pub enum CharacterListContent {
    Characters {
        characters: Vec<CharacterListEntry>,
        job: u8,
    },
    Empty,
    JobSpread {
        hunters: u8,
        thieves: u8,
    },
}

impl CharacterListContent {
    pub fn characters(characters: Vec<CharacterListEntry>, job: u8) -> Self {
        CharacterListContent::Characters { characters, job }
    }

    pub fn jobspread(hunters: u8, thieves: u8) -> Self {
        CharacterListContent::JobSpread { hunters, thieves }
    }
}

#[derive(Clone, Serialize, ByteSize, Debug)]
pub enum CharacterListResult {
    #[silkroad(value = 1)]
    Ok { content: CharacterListContent },
    #[silkroad(value = 2)]
    Error { error: CharacterListError },
}

impl CharacterListResult {
    pub fn ok(content: CharacterListContent) -> Self {
        CharacterListResult::Ok { content }
    }

    pub fn error(error: CharacterListError) -> Self {
        CharacterListResult::Error { error }
    }
}

#[derive(Clone, Deserialize, ByteSize, Debug)]
pub enum CharacterListRequestAction {
    #[silkroad(value = 1)]
    Create {
        character_name: String,
        ref_id: u32,
        scale: u8,
        chest: u32,
        pants: u32,
        boots: u32,
        weapon: u32,
    },
    #[silkroad(value = 2)]
    List,
    #[silkroad(value = 3)]
    Delete { character_name: String },
    #[silkroad(value = 4)]
    CheckName { character_name: String },
    #[silkroad(value = 5)]
    Restore { character_name: String },
    #[silkroad(value = 9)]
    ShowJobSpread,
    #[silkroad(value = 0x10)]
    AssignJob { job: u8 },
}

impl CharacterListRequestAction {
    pub fn create(
        character_name: String,
        ref_id: u32,
        scale: u8,
        chest: u32,
        pants: u32,
        boots: u32,
        weapon: u32,
    ) -> Self {
        CharacterListRequestAction::Create {
            character_name,
            ref_id,
            scale,
            chest,
            pants,
            boots,
            weapon,
        }
    }

    pub fn delete(character_name: String) -> Self {
        CharacterListRequestAction::Delete { character_name }
    }

    pub fn checkname(character_name: String) -> Self {
        CharacterListRequestAction::CheckName { character_name }
    }

    pub fn restore(character_name: String) -> Self {
        CharacterListRequestAction::Restore { character_name }
    }

    pub fn assignjob(job: u8) -> Self {
        CharacterListRequestAction::AssignJob { job }
    }
}

#[derive(Clone, Copy, Serialize, ByteSize, Debug)]
pub enum CharacterJoinResult {
    #[silkroad(value = 1)]
    Success,
    #[silkroad(value = 2)]
    Error { error: CharacterListError },
}

impl CharacterJoinResult {
    pub fn error(error: CharacterListError) -> Self {
        CharacterJoinResult::Error { error }
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub enum TimeInformation {
    #[silkroad(value = 1)]
    Deleting {
        last_logout: SilkroadTime,
        deletion_time_remaining: u32,
    },
    #[silkroad(value = 0)]
    Playable { last_logout: SilkroadTime },
}

impl Debug for TimeInformation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeInformation::Deleting {
                last_logout,
                deletion_time_remaining,
            } => f
                .debug_struct("TimeInformation::Deleting")
                .field("last_logout", &last_logout.timestamp())
                .field("deletion_time_remaining", deletion_time_remaining)
                .finish(),
            TimeInformation::Playable { last_logout } => f
                .debug_struct("TimeInformation::Playable")
                .field("last_logout", &last_logout.timestamp())
                .finish(),
        }
    }
}

impl TimeInformation {
    pub fn deleting(last_logout: SilkroadTime, deletion_time_remaining: u32) -> Self {
        TimeInformation::Deleting {
            last_logout,
            deletion_time_remaining,
        }
    }

    pub fn playable(last_logout: SilkroadTime) -> Self {
        TimeInformation::Playable { last_logout }
    }
}

#[derive(Clone, Copy, Serialize, ByteSize, Debug)]
pub struct CharacterListEquippedItem {
    pub id: u32,
    pub upgrade_level: u8,
}

impl CharacterListEquippedItem {
    pub fn new(id: u32, upgrade_level: u8) -> Self {
        CharacterListEquippedItem { id, upgrade_level }
    }
}

#[derive(Clone, Debug, Serialize, ByteSize)]
pub struct CharacterListAvatarItem {
    pub id: u32,
}

impl CharacterListAvatarItem {
    pub fn new(id: u32) -> Self {
        CharacterListAvatarItem { id }
    }
}

#[derive(Clone, Serialize, ByteSize, Debug)]
pub struct CharacterListEntry {
    pub ref_id: u32,
    pub name: String,
    pub unknown: String,
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
    pub playtime_info: TimeInformation,
    pub guild_member_class: u8,
    pub guild_rename_required: Option<String>,
    pub academy_member_class: u8,
    pub equipped_items: Vec<CharacterListEquippedItem>,
    pub avatar_items: Vec<CharacterListAvatarItem>,
}

impl CharacterListEntry {
    pub fn new(
        ref_id: u32,
        name: String,
        scale: u8,
        level: u8,
        exp: u64,
        strength: u16,
        intelligence: u16,
        stat_points: u16,
        sp: u32,
        hp: u32,
        mp: u32,
        region: u16,
        playtime_info: TimeInformation,
        guild_member_class: u8,
        guild_rename_required: Option<String>,
        academy_member_class: u8,
        equipped_items: Vec<CharacterListEquippedItem>,
        avatar_items: Vec<CharacterListAvatarItem>,
    ) -> Self {
        CharacterListEntry {
            ref_id,
            name,
            unknown: String::new(),
            scale,
            level,
            exp,
            strength,
            intelligence,
            stat_points,
            sp,
            hp,
            mp,
            region,
            playtime_info,
            guild_member_class,
            guild_rename_required,
            academy_member_class,
            equipped_items,
            avatar_items,
        }
    }
}

#[derive(Clone, Serialize, ByteSize, Packet, Debug)]
#[packet(opcode = 0xB007)]
pub struct CharacterListResponse {
    pub action: CharacterListAction,
    pub result: CharacterListResult,
}

impl CharacterListResponse {
    pub fn new(action: CharacterListAction, result: CharacterListResult) -> Self {
        CharacterListResponse { action, result }
    }
}

#[derive(Clone, Deserialize, ByteSize, Packet, Debug)]
#[packet(opcode = 0x7007)]
pub struct CharacterListRequest {
    pub action: CharacterListRequestAction,
}

#[derive(Clone, Deserialize, ByteSize, Packet, Debug)]
#[packet(opcode = 0x7001)]
pub struct CharacterJoinRequest {
    pub character_name: String,
}

#[derive(Clone, Copy, Serialize, ByteSize, Packet, Debug)]
#[packet(opcode = 0xB001)]
pub struct CharacterJoinResponse {
    pub result: CharacterJoinResult,
}

impl CharacterJoinResponse {
    pub fn success() -> Self {
        CharacterJoinResponse {
            result: CharacterJoinResult::Success,
        }
    }

    pub fn error(error: CharacterListError) -> Self {
        CharacterJoinResponse {
            result: CharacterJoinResult::Error { error },
        }
    }
}

#[derive(Clone, Copy, Serialize, ByteSize, Packet, Debug)]
#[packet(opcode = 0x303D)]
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

impl CharacterStatsMessage {
    pub fn new(
        phys_attack_min: u32,
        phys_attack_max: u32,
        mag_attack_min: u32,
        mag_attack_max: u32,
        phys_defense: u16,
        mag_defense: u16,
        hit_rate: u16,
        parry_rate: u16,
        max_hp: u32,
        max_mp: u32,
        strength: u16,
        intelligence: u16,
    ) -> Self {
        CharacterStatsMessage {
            phys_attack_min,
            phys_attack_max,
            mag_attack_min,
            mag_attack_max,
            phys_defense,
            mag_defense,
            hit_rate,
            parry_rate,
            max_hp,
            max_mp,
            strength,
            intelligence,
        }
    }
}

#[derive(Clone, Serialize, ByteSize, Packet, Debug)]
#[packet(opcode = 0x3601)]
pub struct UnknownPacket {
    pub unknown_1: u8,
    #[silkroad(size = 4)]
    pub unknown_2: Vec<UnknownPacketInner>,
}

#[derive(Clone, Copy, Serialize, ByteSize, Debug)]
pub struct UnknownPacketInner {
    unknown: u32,
    unknown_2: Option<u32>,
}

impl UnknownPacket {
    pub fn new() -> Self {
        UnknownPacket {
            unknown_1: 4,
            unknown_2: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Serialize, ByteSize, Packet, Debug)]
#[packet(opcode = 0xB602)]
pub struct UnknownPacket2 {
    pub unknown_1: u8,
    pub id: u32,
    pub unknown_2: u32,
}

impl UnknownPacket2 {
    pub fn new(id: u32) -> Self {
        UnknownPacket2 {
            unknown_1: 1,
            id,
            unknown_2: 0,
        }
    }
}

pub const MACRO_POTION: u8 = 1;
pub const MACRO_SKILL: u8 = 2;
pub const MACRO_HUNT: u8 = 4;

#[derive(Serialize, ByteSize, Clone, Packet)]
#[packet(opcode = 0x3555)]
pub enum MacroStatus {
    #[silkroad(value = 0)]
    Possible(u8, u8),
    #[silkroad(value = 1)]
    Disabled(String, String, u8),
}

#[derive(Deserialize, Serialize, ByteSize, Copy, Clone, Packet, Debug)]
#[packet(opcode = 0x34c6)]
pub struct FinishLoading;

define_inbound_protocol! { CharselectClientProtocol =>
    CharacterListRequest,
    CharacterJoinRequest,
    FinishLoading
}

define_outbound_protocol! { CharselectServerProtocol =>
    CharacterJoinResponse,
    CharacterStatsMessage,
    UnknownPacket,
    UnknownPacket2
}
