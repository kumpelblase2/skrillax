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
pub enum PvpCape {
    None,
    Red,
    Gray,
    Blue,
    White,
    Yellow,
}

#[derive(Clone, PartialEq, PartialOrd)]
pub enum AliveState {
    Spawning,
    Alive,
    Dead,
}

#[derive(Clone, PartialEq, PartialOrd)]
pub enum JobType {
    None,
    Trader,
    Thief,
    Hunter,
}

#[derive(Clone, PartialEq, PartialOrd)]
pub enum PlayerKillState {
    None,
    Purple,
    Red,
}

#[derive(Clone, PartialEq, PartialOrd)]
pub enum ActiveScroll {
    None,
    ReturnScroll,
    JobScroll,
}

#[derive(Clone)]
pub enum InventoryItemContentData {
    Equipment {
        plus_level: u8,
        variance: u64,
        durability: u32,
        magic: Vec<InventoryItemMagicData>,
        bindings_1: InventoryItemBindingData,
        bindings_2: InventoryItemBindingData,
        bindings_3: InventoryItemBindingData,
        bindings_4: InventoryItemBindingData,
    }
    ,
    Expendable {
        stack_size: u8,
    }
    ,
}

#[derive(Clone)]
pub enum InteractOptions {
    None,
    Talk {
        options: Vec<u8>,
    }
    ,
}

#[derive(Clone, PartialEq, PartialOrd)]
pub enum BodyState {
    None,
    Berserk,
    Untouchable,
    GM_Invincible,
    GM_Invisible,
    Berserk2,
    Stealth,
    Invisible,
}

#[derive(Clone, PartialEq, PartialOrd)]
pub enum GroupSpawnType {
    Spawn,
    Despawn,
}

#[derive(Clone, PartialEq, PartialOrd)]
pub enum EntityRarity {
    Normal,
    Champion,
    Unique,
    Giant,
    Titan,
    Elite,
    Elite_String,
    Unique2,
    NormalParty,
    ChampionParty,
    UniqueParty,
    GiantParty,
    TitanParty,
    EliteParty,
    Unique2Party,
}

#[derive(Clone)]
pub enum GroupSpawnDataContent {
    Despawn {
        id: u32,
    }
    ,
    Spawn {
        object_id: u32,
        data: EntityTypeSpawnData,
    }
    ,
}

#[derive(Clone)]
pub enum EntityTypeSpawnData {
    Item,
    Gold {
        amount: u32,
        unique_id: u32,
        position: Position,
        owner: Option<u32>,
        rarity: u8,
    }
    ,
    Character {
        scale: u8,
        berserk_level: u8,
        pvp_cape: PvpCape,
        beginner: bool,
        title: u8,
        equipment: Vec<u32>,
        avatar_items: Vec<u32>,
        mask: Option<u32>,
        movement: EntityMovementState,
        entity_state: EntityState,
        name: String,
        job_type: JobType,
        job_level: u8,
        pk_state: PlayerKillState,
        mounted: bool,
        in_combat: bool,
        active_scroll: ActiveScroll,
        unknown2: u8,
    }
    ,
    Monster {
        unique_id: u32,
        position: Position,
        movement: EntityMovementState,
        entity_state: EntityState,
        interaction_options: InteractOptions,
        rarity: EntityRarity,
        unknown: u32,
    }
    ,
}

#[derive(Clone, PartialEq, PartialOrd)]
pub enum WeatherType {
    Clear,
    Rain,
    Snow,
}

#[derive(Clone, PartialEq, PartialOrd)]
pub enum ConsignmentErrorCode {
    NotEnoughGold,
}

#[derive(Clone)]
pub enum ConsignmentResult {
    Success {
        items: Vec<ConsignmentItem>,
    }
    ,
    Error {
        code: ConsignmentErrorCode,
    }
    ,
}

#[derive(Clone)]
pub enum GameNotificationContent {
    UniqueSpawned {
        unknown: u8,
        ref_id: u16,
    }
    ,
    UniqueKilled {
        ref_id: u16,
    }
    ,
}

#[derive(Clone, PartialEq, PartialOrd)]
pub enum MovementType {
    Running,
    Walking,
}

#[derive(Clone, PartialEq, PartialOrd)]
pub enum ActionState {
    None,
    Walking,
    Running,
    Sitting,
}

#[derive(Clone)]
pub enum MovementTarget {
    TargetLocation {
        region: u16,
        x: u16,
        y: u16,
        z: u16,
    }
    ,
    Direction {
        unknown: u8,
        angle: u16,
    }
    ,
}

#[derive(Clone)]
pub enum EntityMovementState {
    Moving {
        movement_type: MovementType,
        region: u16,
        x: u16,
        y: u16,
        z: u16,
    }
    ,
    Standing {
        movement_type: MovementType,
        unknown: u8,
        angle: u16,
    }
    ,
}

#[derive(Clone)]
pub struct Position {
    pub region: u16,
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
    pub heading: u16,
}

#[derive(Clone)]
pub struct EntityState {
    pub alive: AliveState,
    pub unknown1: u8,
    pub action_state: ActionState,
    pub body_state: BodyState,
    pub unknown2: u8,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub berserk_speed: f32,
    pub active_buffs: Vec<ActiveBuffData>,
}

#[derive(Clone)]
pub struct InventoryItemMagicData;

#[derive(Clone)]
pub struct InventoryItemBindingData {
    pub kind: u8,
    pub value: u8,
}

#[derive(Clone)]
pub struct MasteryData {
    pub id: u32,
    pub level: u8,
}

#[derive(Clone)]
pub struct ActiveQuestData {
    pub id: u32,
    pub repeat_count: u8,
    pub unknown_1: u8,
    pub unknown_2: u16,
    pub kind: u8,
    pub status: u8,
    pub objectives: Vec<ActiveQuestObjectData>,
}

#[derive(Clone)]
pub struct ActiveQuestObjectData {
    pub index: u8,
    pub incomplete: bool,
    pub name: String,
    pub tasks: Vec<u32>,
    pub task_ids: Vec<u32>,
}

#[derive(Clone)]
pub struct InventoryItemData {
    pub slot: u8,
    pub rent_data: u32,
    pub item_id: u32,
    pub content_data: InventoryItemContentData,
}

#[derive(Clone)]
pub struct InventoryAvatarItemData;

#[derive(Clone)]
pub struct ActiveBuffData {
    pub id: u32,
    pub token: u32,
}

#[derive(Clone)]
pub struct HotkeyData {
    pub slot: u8,
    pub kind: u8,
    pub data: u32,
}

#[derive(Clone)]
pub struct FriendListGroup {
    pub id: u16,
    pub name: String,
}

#[derive(Clone)]
pub struct FriendListEntry {
    pub char_id: u32,
    pub name: String,
    pub char_model: u32,
    pub group_id: u16,
    pub offline: bool,
}

#[derive(Clone)]
pub struct ConsignmentItem {
    pub personal_id: u32,
    pub status: u8,
    pub ref_item_id: u32,
    pub sell_count: u32,
    pub price: u64,
    pub deposit: u64,
    pub fee: u64,
    pub end_date: u32,
}

#[derive(Clone)]
pub struct MovementSource {
    pub region: u16,
    pub x: u16,
    pub y: f32,
    pub z: u16,
}

#[derive(Clone)]
pub struct CelestialUpdate {
    pub unique_id: u32,
    pub moon_position: u16,
    pub hour: u8,
    pub minute: u8,
}

impl From<CelestialUpdate> for Bytes {
    fn from(op: CelestialUpdate) -> Bytes {
        let mut data_writer = BytesMut::new();
        data_writer.put_u32_le(op.unique_id);
        data_writer.put_u16_le(op.moon_position);
        data_writer.put_u8(op.hour);
        data_writer.put_u8(op.minute);
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for CelestialUpdate {
    fn into(self) -> ServerPacket {
        ServerPacket::CelestialUpdate(self)
    }
}

impl CelestialUpdate {
    pub fn new(unique_id: u32, moon_position: u16, hour: u8, minute: u8) -> Self {
        CelestialUpdate { unique_id, moon_position, hour, minute,  }
    }
}

#[derive(Clone)]
pub struct LunarEventInfo {
    pub unknown_1: u8,
    pub unknown_2: u8,
    pub unknown_3: u32,
    pub unknown_4: u32,
    pub current_achieved: u32,
    pub total: u32,
}

impl From<LunarEventInfo> for Bytes {
    fn from(op: LunarEventInfo) -> Bytes {
        let mut data_writer = BytesMut::new();
        data_writer.put_u8(op.unknown_1);
        data_writer.put_u8(op.unknown_2);
        data_writer.put_u32_le(op.unknown_3);
        data_writer.put_u32_le(op.unknown_4);
        data_writer.put_u32_le(op.current_achieved);
        data_writer.put_u32_le(op.total);
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for LunarEventInfo {
    fn into(self) -> ServerPacket {
        ServerPacket::LunarEventInfo(self)
    }
}

impl LunarEventInfo {
    pub fn new(current_achieved: u32, total: u32) -> Self {
        LunarEventInfo { unknown_1: 2, unknown_2: 3, unknown_3: 1, unknown_4: 0x7535, current_achieved, total,  }
    }
}

#[derive(Clone)]
pub struct CharacterSpawnStart;

impl From<CharacterSpawnStart> for Bytes {
    fn from(op: CharacterSpawnStart) -> Bytes {
        let mut data_writer = BytesMut::new();
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for CharacterSpawnStart {
    fn into(self) -> ServerPacket {
        ServerPacket::CharacterSpawnStart(self)
    }
}

impl CharacterSpawnStart {
    pub fn new() -> Self {
        CharacterSpawnStart {  }
    }
}

#[derive(Clone)]
pub struct CharacterSpawn {
    pub time: u32,
    pub ref_id: u32,
    pub scale: u8,
    pub level: u8,
    pub max_level: u8,
    pub exp: u64,
    pub sp_exp: u32,
    pub gold: u64,
    pub sp: u32,
    pub stat_points: u16,
    pub berserk_points: u8,
    pub unknown_1: u32,
    pub hp: u32,
    pub mp: u32,
    pub beginner: bool,
    pub player_kills_today: u8,
    pub player_kills_total: u16,
    pub player_kills_penalty: u32,
    pub berserk_level: u8,
    pub free_pvp: u8,
    pub fortress_war_mark: u8,
    pub service_end: DateTime<Utc>,
    pub user_type: u8,
    pub server_max_level: u8,
    pub unknown_2: u16,
    pub inventory_size: u8,
    pub inventory_items: Vec<InventoryItemData>,
    pub avatar_item_size: u8,
    pub avatar_items: Vec<InventoryAvatarItemData>,
    pub unknown_3: u8,
    pub unknown_4: u8,
    pub unknown_5: u16,
    pub masteries: Vec<MasteryData>,
    pub unknown_6: u8,
    pub unknown_7: u8,
    pub completed_quests: Vec<u32>,
    pub active_quests: Vec<ActiveQuestData>,
    pub unknown_8: u8,
    pub unknown_9: u32,
    pub unique_id: u32,
    pub position: Position,
    pub destination_flag: u8,
    pub unknown_10: u8,
    pub unknown_11: u8,
    pub angle: u16,
    pub entity_state: EntityState,
    pub character_name: String,
    pub unknown_14: u16,
    pub job_name: String,
    pub job_type: JobType,
    pub job_level: u8,
    pub job_exp: u32,
    pub job_contribution: u32,
    pub job_reward: u32,
    pub pvp_state: u8,
    pub transport_flag: bool,
    pub in_combat: u8,
    pub unknown_15: u8,
    pub unknown_16: u8,
    pub pvp_flag: u8,
    pub unknown_17: u8,
    pub unknown_18: u64,
    pub jid: u32,
    pub gm: bool,
    pub unknown_19: u32,
    pub hotkeys: Vec<HotkeyData>,
    pub unknown_20: u8,
    pub auto_hp: u16,
    pub auto_mp: u16,
    pub auto_pill: u16,
    pub potion_delay: u8,
    pub blocked_players: Vec<String>,
    pub unknown_21: u32,
}

impl From<CharacterSpawn> for Bytes {
    fn from(op: CharacterSpawn) -> Bytes {
        let mut data_writer = BytesMut::new();
        data_writer.put_u32_le(op.time);
        data_writer.put_u32_le(op.ref_id);
        data_writer.put_u8(op.scale);
        data_writer.put_u8(op.level);
        data_writer.put_u8(op.max_level);
        data_writer.put_u64_le(op.exp);
        data_writer.put_u32_le(op.sp_exp);
        data_writer.put_u64_le(op.gold);
        data_writer.put_u32_le(op.sp);
        data_writer.put_u16_le(op.stat_points);
        data_writer.put_u8(op.berserk_points);
        data_writer.put_u32_le(op.unknown_1);
        data_writer.put_u32_le(op.hp);
        data_writer.put_u32_le(op.mp);
        data_writer.put_u8(op.beginner as u8);
        data_writer.put_u8(op.player_kills_today);
        data_writer.put_u16_le(op.player_kills_total);
        data_writer.put_u32_le(op.player_kills_penalty);
        data_writer.put_u8(op.berserk_level);
        data_writer.put_u8(op.free_pvp);
        data_writer.put_u8(op.fortress_war_mark);
        data_writer.put_u16_le(op.service_end.year() as u16);
        data_writer.put_u16_le(op.service_end.month() as u16);
        data_writer.put_u16_le(op.service_end.day() as u16);
        data_writer.put_u16_le(op.service_end.hour() as u16);
        data_writer.put_u16_le(op.service_end.minute() as u16);
        data_writer.put_u16_le(op.service_end.second() as u16);
        data_writer.put_u32_le(op.service_end.timestamp_millis() as u32);
        data_writer.put_u8(op.user_type);
        data_writer.put_u8(op.server_max_level);
        data_writer.put_u16_le(op.unknown_2);
        data_writer.put_u8(op.inventory_size);
        data_writer.put_u8(op.inventory_items.len() as u8);
        for element in op.inventory_items.iter() {
            data_writer.put_u8(element.slot);
            data_writer.put_u32_le(element.rent_data);
            data_writer.put_u32_le(element.item_id);
            match &element.content_data {
                InventoryItemContentData::Equipment { plus_level, variance, durability, magic, bindings_1, bindings_2, bindings_3, bindings_4,  } => {
                    data_writer.put_u8(*plus_level);
                    data_writer.put_u64_le(*variance);
                    data_writer.put_u32_le(*durability);
                    data_writer.put_u8(magic.len() as u8);
                    for element in magic.iter() {
                    }
                    data_writer.put_u8(bindings_1.kind);
                    data_writer.put_u8(bindings_1.value);
                    data_writer.put_u8(bindings_2.kind);
                    data_writer.put_u8(bindings_2.value);
                    data_writer.put_u8(bindings_3.kind);
                    data_writer.put_u8(bindings_3.value);
                    data_writer.put_u8(bindings_4.kind);
                    data_writer.put_u8(bindings_4.value);
                }
                InventoryItemContentData::Expendable { stack_size,  } => {
                    data_writer.put_u8(*stack_size);
                }
            }
        }
        data_writer.put_u8(op.avatar_item_size);
        data_writer.put_u8(op.avatar_items.len() as u8);
        for element in op.avatar_items.iter() {
        }
        data_writer.put_u8(op.unknown_3);
        data_writer.put_u8(op.unknown_4);
        data_writer.put_u16_le(op.unknown_5);
        for element in op.masteries.iter() {
            data_writer.put_u8(1);
            data_writer.put_u32_le(element.id);
            data_writer.put_u8(element.level);
        }
        data_writer.put_u8(2);
        data_writer.put_u8(op.unknown_6);
        data_writer.put_u8(op.unknown_7);
        data_writer.put_u16_le(op.completed_quests.len() as u16);
        for element in op.completed_quests.iter() {
            data_writer.put_u32_le(*element);
        }
        data_writer.put_u8(op.active_quests.len() as u8);
        for element in op.active_quests.iter() {
            data_writer.put_u32_le(element.id);
            data_writer.put_u8(element.repeat_count);
            data_writer.put_u8(element.unknown_1);
            data_writer.put_u16_le(element.unknown_2);
            data_writer.put_u8(element.kind);
            data_writer.put_u8(element.status);
            data_writer.put_u8(element.objectives.len() as u8);
            for element in element.objectives.iter() {
                data_writer.put_u8(element.index);
                data_writer.put_u8(element.incomplete as u8);
                data_writer.put_u16_le(element.name.len() as u16);
                data_writer.put_slice(element.name.as_bytes());
                data_writer.put_u8(element.tasks.len() as u8);
                for element in element.tasks.iter() {
                    data_writer.put_u32_le(*element);
                }
                data_writer.put_u8(element.task_ids.len() as u8);
                for element in element.task_ids.iter() {
                    data_writer.put_u32_le(*element);
                }
            }
        }
        data_writer.put_u8(op.unknown_8);
        data_writer.put_u32_le(op.unknown_9);
        data_writer.put_u32_le(op.unique_id);
        data_writer.put_u16_le(op.position.region);
        data_writer.put_f32_le(op.position.pos_x);
        data_writer.put_f32_le(op.position.pos_y);
        data_writer.put_f32_le(op.position.pos_z);
        data_writer.put_u16_le(op.position.heading);
        data_writer.put_u8(op.destination_flag);
        data_writer.put_u8(op.unknown_10);
        data_writer.put_u8(op.unknown_11);
        data_writer.put_u16_le(op.angle);
        match &op.entity_state.alive {
            AliveState::Spawning => data_writer.put_u8(0),
            AliveState::Alive => data_writer.put_u8(1),
            AliveState::Dead => data_writer.put_u8(2),
        }
        data_writer.put_u8(op.entity_state.unknown1);
        match &op.entity_state.action_state {
            ActionState::None => data_writer.put_u8(0),
            ActionState::Walking => data_writer.put_u8(2),
            ActionState::Running => data_writer.put_u8(3),
            ActionState::Sitting => data_writer.put_u8(4),
        }
        match &op.entity_state.body_state {
            BodyState::None => data_writer.put_u8(0),
            BodyState::Berserk => data_writer.put_u8(1),
            BodyState::Untouchable => data_writer.put_u8(2),
            BodyState::GM_Invincible => data_writer.put_u8(3),
            BodyState::GM_Invisible => data_writer.put_u8(4),
            BodyState::Berserk2 => data_writer.put_u8(5),
            BodyState::Stealth => data_writer.put_u8(6),
            BodyState::Invisible => data_writer.put_u8(7),
        }
        data_writer.put_u8(op.entity_state.unknown2);
        data_writer.put_f32_le(op.entity_state.walk_speed);
        data_writer.put_f32_le(op.entity_state.run_speed);
        data_writer.put_f32_le(op.entity_state.berserk_speed);
        data_writer.put_u8(op.entity_state.active_buffs.len() as u8);
        for element in op.entity_state.active_buffs.iter() {
            data_writer.put_u32_le(element.id);
            data_writer.put_u32_le(element.token);
        }
        data_writer.put_u16_le(op.character_name.len() as u16);
        data_writer.put_slice(op.character_name.as_bytes());
        data_writer.put_u16_le(op.unknown_14);
        data_writer.put_u16_le(op.job_name.len() as u16);
        data_writer.put_slice(op.job_name.as_bytes());
        match &op.job_type {
            JobType::None => data_writer.put_u8(0),
            JobType::Trader => data_writer.put_u8(1),
            JobType::Thief => data_writer.put_u8(2),
            JobType::Hunter => data_writer.put_u8(3),
        }
        data_writer.put_u8(op.job_level);
        data_writer.put_u32_le(op.job_exp);
        data_writer.put_u32_le(op.job_contribution);
        data_writer.put_u32_le(op.job_reward);
        data_writer.put_u8(op.pvp_state);
        data_writer.put_u8(op.transport_flag as u8);
        data_writer.put_u8(op.in_combat);
        data_writer.put_u8(op.unknown_15);
        data_writer.put_u8(op.unknown_16);
        data_writer.put_u8(op.pvp_flag);
        data_writer.put_u8(op.unknown_17);
        data_writer.put_u64_le(op.unknown_18);
        data_writer.put_u32_le(op.jid);
        data_writer.put_u8(op.gm as u8);
        data_writer.put_u32_le(op.unknown_19);
        data_writer.put_u8(op.hotkeys.len() as u8);
        for element in op.hotkeys.iter() {
            data_writer.put_u8(element.slot);
            data_writer.put_u8(element.kind);
            data_writer.put_u32_le(element.data);
        }
        data_writer.put_u8(op.unknown_20);
        data_writer.put_u16_le(op.auto_hp);
        data_writer.put_u16_le(op.auto_mp);
        data_writer.put_u16_le(op.auto_pill);
        data_writer.put_u8(op.potion_delay);
        data_writer.put_u8(op.blocked_players.len() as u8);
        for element in op.blocked_players.iter() {
            data_writer.put_u16_le(element.len() as u16);
            data_writer.put_slice(element.as_bytes());
        }
        data_writer.put_u32_le(op.unknown_21);
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for CharacterSpawn {
    fn into(self) -> ServerPacket {
        ServerPacket::CharacterSpawn(self)
    }
}

impl CharacterSpawn {
    pub fn new(time: u32, ref_id: u32, scale: u8, level: u8, max_level: u8, exp: u64, sp_exp: u32, gold: u64, sp: u32, stat_points: u16, berserk_points: u8, hp: u32, mp: u32, beginner: bool, player_kills_today: u8, player_kills_total: u16, player_kills_penalty: u32, berserk_level: u8, free_pvp: u8, fortress_war_mark: u8, service_end: DateTime<Utc>, user_type: u8, server_max_level: u8, inventory_size: u8, inventory_items: Vec<InventoryItemData>, avatar_item_size: u8, avatar_items: Vec<InventoryAvatarItemData>, masteries: Vec<MasteryData>, completed_quests: Vec<u32>, active_quests: Vec<ActiveQuestData>, unique_id: u32, position: Position, destination_flag: u8, angle: u16, entity_state: EntityState, character_name: String, job_name: String, job_type: JobType, job_level: u8, job_exp: u32, job_contribution: u32, job_reward: u32, pvp_state: u8, transport_flag: bool, in_combat: u8, pvp_flag: u8, jid: u32, gm: bool, hotkeys: Vec<HotkeyData>, auto_hp: u16, auto_mp: u16, auto_pill: u16, potion_delay: u8, blocked_players: Vec<String>) -> Self {
        CharacterSpawn { time, ref_id, scale, level, max_level, exp, sp_exp, gold, sp, stat_points, berserk_points, unknown_1: 0, hp, mp, beginner, player_kills_today, player_kills_total, player_kills_penalty, berserk_level, free_pvp, fortress_war_mark, service_end, user_type, server_max_level, unknown_2: 0x0107, inventory_size, inventory_items, avatar_item_size, avatar_items, unknown_3: 0, unknown_4: 0xb, unknown_5: 0, masteries, unknown_6: 0, unknown_7: 2, completed_quests, active_quests, unknown_8: 0, unknown_9: 0, unique_id, position, destination_flag, unknown_10: 1, unknown_11: 0, angle, entity_state, character_name, unknown_14: 0, job_name, job_type, job_level, job_exp, job_contribution, job_reward, pvp_state, transport_flag, in_combat, unknown_15: 0, unknown_16: 0, pvp_flag, unknown_17: 0xFF, unknown_18: 0x8000d7, jid, gm, unknown_19: 0x19, hotkeys, unknown_20: 0, auto_hp, auto_mp, auto_pill, potion_delay, blocked_players, unknown_21: 0x9f000000,  }
    }
}

#[derive(Clone)]
pub struct CharacterSpawnEnd;

impl From<CharacterSpawnEnd> for Bytes {
    fn from(op: CharacterSpawnEnd) -> Bytes {
        let mut data_writer = BytesMut::new();
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for CharacterSpawnEnd {
    fn into(self) -> ServerPacket {
        ServerPacket::CharacterSpawnEnd(self)
    }
}

impl CharacterSpawnEnd {
    pub fn new() -> Self {
        CharacterSpawnEnd {  }
    }
}

#[derive(Clone)]
pub struct CharacterFinished {
    pub unknown: u16,
}

impl From<CharacterFinished> for Bytes {
    fn from(op: CharacterFinished) -> Bytes {
        let mut data_writer = BytesMut::new();
        data_writer.put_u16_le(op.unknown);
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for CharacterFinished {
    fn into(self) -> ServerPacket {
        ServerPacket::CharacterFinished(self)
    }
}

impl CharacterFinished {
    pub fn new() -> Self {
        CharacterFinished { unknown: 0,  }
    }
}

#[derive(Clone)]
pub struct EntityDespawn {
    pub entity_id: u32,
}

impl From<EntityDespawn> for Bytes {
    fn from(op: EntityDespawn) -> Bytes {
        let mut data_writer = BytesMut::new();
        data_writer.put_u32_le(op.entity_id);
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for EntityDespawn {
    fn into(self) -> ServerPacket {
        ServerPacket::EntityDespawn(self)
    }
}

impl EntityDespawn {
    pub fn new(entity_id: u32) -> Self {
        EntityDespawn { entity_id,  }
    }
}

#[derive(Clone)]
pub struct EntitySpawn {
    pub spawn_data: EntityTypeSpawnData,
    pub unknown_3: u8,
    pub unknown_4: u32,
    pub unknown_5: u8,
}

impl From<EntitySpawn> for Bytes {
    fn from(op: EntitySpawn) -> Bytes {
        let mut data_writer = BytesMut::new();
        match &op.spawn_data {
            EntityTypeSpawnData::Item => {},
            EntityTypeSpawnData::Gold { amount, unique_id, position, owner, rarity,  } => {
                data_writer.put_u32_le(*amount);
                data_writer.put_u32_le(*unique_id);
                data_writer.put_u16_le(position.region);
                data_writer.put_f32_le(position.pos_x);
                data_writer.put_f32_le(position.pos_y);
                data_writer.put_f32_le(position.pos_z);
                data_writer.put_u16_le(position.heading);
                if let Some(owner) = &owner {
                    data_writer.put_u8(1);
                    data_writer.put_u32_le(*owner);
                }
                else {
                    data_writer.put_u8(0);
                }
                data_writer.put_u8(*rarity);
            }
            EntityTypeSpawnData::Character { scale, berserk_level, pvp_cape, beginner, title, equipment, avatar_items, mask, movement, entity_state, name, job_type, job_level, pk_state, mounted, in_combat, active_scroll, unknown2,  } => {
                data_writer.put_u8(*scale);
                data_writer.put_u8(*berserk_level);
                match &pvp_cape {
                    PvpCape::None => data_writer.put_u8(0),
                    PvpCape::Red => data_writer.put_u8(1),
                    PvpCape::Gray => data_writer.put_u8(2),
                    PvpCape::Blue => data_writer.put_u8(3),
                    PvpCape::White => data_writer.put_u8(4),
                    PvpCape::Yellow => data_writer.put_u8(5),
                }
                data_writer.put_u8(*beginner as u8);
                data_writer.put_u8(*title);
                data_writer.put_u8(equipment.len() as u8);
                for element in equipment.iter() {
                    data_writer.put_u32_le(*element);
                }
                data_writer.put_u8(avatar_items.len() as u8);
                for element in avatar_items.iter() {
                    data_writer.put_u32_le(*element);
                }
                if let Some(mask) = &mask {
                    data_writer.put_u8(1);
                    data_writer.put_u32_le(*mask);
                }
                else {
                    data_writer.put_u8(0);
                }
                match &movement {
                    EntityMovementState::Moving { movement_type, region, x, y, z,  } => {
                        data_writer.put_u8(1);
                        match &movement_type {
                            MovementType::Running => data_writer.put_u8(0),
                            MovementType::Walking => data_writer.put_u8(1),
                        }
                        data_writer.put_u16_le(*region);
                        data_writer.put_u16_le(*x);
                        data_writer.put_u16_le(*y);
                        data_writer.put_u16_le(*z);
                    }
                    EntityMovementState::Standing { movement_type, unknown, angle,  } => {
                        data_writer.put_u8(0);
                        match &movement_type {
                            MovementType::Running => data_writer.put_u8(0),
                            MovementType::Walking => data_writer.put_u8(1),
                        }
                        data_writer.put_u8(*unknown);
                        data_writer.put_u16_le(*angle);
                    }
                }
                match &entity_state.alive {
                    AliveState::Spawning => data_writer.put_u8(0),
                    AliveState::Alive => data_writer.put_u8(1),
                    AliveState::Dead => data_writer.put_u8(2),
                }
                data_writer.put_u8(entity_state.unknown1);
                match &entity_state.action_state {
                    ActionState::None => data_writer.put_u8(0),
                    ActionState::Walking => data_writer.put_u8(2),
                    ActionState::Running => data_writer.put_u8(3),
                    ActionState::Sitting => data_writer.put_u8(4),
                }
                match &entity_state.body_state {
                    BodyState::None => data_writer.put_u8(0),
                    BodyState::Berserk => data_writer.put_u8(1),
                    BodyState::Untouchable => data_writer.put_u8(2),
                    BodyState::GM_Invincible => data_writer.put_u8(3),
                    BodyState::GM_Invisible => data_writer.put_u8(4),
                    BodyState::Berserk2 => data_writer.put_u8(5),
                    BodyState::Stealth => data_writer.put_u8(6),
                    BodyState::Invisible => data_writer.put_u8(7),
                }
                data_writer.put_u8(entity_state.unknown2);
                data_writer.put_f32_le(entity_state.walk_speed);
                data_writer.put_f32_le(entity_state.run_speed);
                data_writer.put_f32_le(entity_state.berserk_speed);
                data_writer.put_u8(entity_state.active_buffs.len() as u8);
                for element in entity_state.active_buffs.iter() {
                    data_writer.put_u32_le(element.id);
                    data_writer.put_u32_le(element.token);
                }
                data_writer.put_u16_le(name.len() as u16);
                data_writer.put_slice(name.as_bytes());
                match &job_type {
                    JobType::None => data_writer.put_u8(0),
                    JobType::Trader => data_writer.put_u8(1),
                    JobType::Thief => data_writer.put_u8(2),
                    JobType::Hunter => data_writer.put_u8(3),
                }
                data_writer.put_u8(*job_level);
                match &pk_state {
                    PlayerKillState::None => data_writer.put_u8(0),
                    PlayerKillState::Purple => data_writer.put_u8(1),
                    PlayerKillState::Red => data_writer.put_u8(2),
                }
                data_writer.put_u8(*mounted as u8);
                data_writer.put_u8(*in_combat as u8);
                match &active_scroll {
                    ActiveScroll::None => data_writer.put_u8(0),
                    ActiveScroll::ReturnScroll => data_writer.put_u8(1),
                    ActiveScroll::JobScroll => data_writer.put_u8(2),
                }
                data_writer.put_u8(*unknown2);
            }
            EntityTypeSpawnData::Monster { unique_id, position, movement, entity_state, interaction_options, rarity, unknown,  } => {
                data_writer.put_u32_le(*unique_id);
                data_writer.put_u16_le(position.region);
                data_writer.put_f32_le(position.pos_x);
                data_writer.put_f32_le(position.pos_y);
                data_writer.put_f32_le(position.pos_z);
                data_writer.put_u16_le(position.heading);
                match &movement {
                    EntityMovementState::Moving { movement_type, region, x, y, z,  } => {
                        data_writer.put_u8(1);
                        match &movement_type {
                            MovementType::Running => data_writer.put_u8(0),
                            MovementType::Walking => data_writer.put_u8(1),
                        }
                        data_writer.put_u16_le(*region);
                        data_writer.put_u16_le(*x);
                        data_writer.put_u16_le(*y);
                        data_writer.put_u16_le(*z);
                    }
                    EntityMovementState::Standing { movement_type, unknown, angle,  } => {
                        data_writer.put_u8(0);
                        match &movement_type {
                            MovementType::Running => data_writer.put_u8(0),
                            MovementType::Walking => data_writer.put_u8(1),
                        }
                        data_writer.put_u8(*unknown);
                        data_writer.put_u16_le(*angle);
                    }
                }
                match &entity_state.alive {
                    AliveState::Spawning => data_writer.put_u8(0),
                    AliveState::Alive => data_writer.put_u8(1),
                    AliveState::Dead => data_writer.put_u8(2),
                }
                data_writer.put_u8(entity_state.unknown1);
                match &entity_state.action_state {
                    ActionState::None => data_writer.put_u8(0),
                    ActionState::Walking => data_writer.put_u8(2),
                    ActionState::Running => data_writer.put_u8(3),
                    ActionState::Sitting => data_writer.put_u8(4),
                }
                match &entity_state.body_state {
                    BodyState::None => data_writer.put_u8(0),
                    BodyState::Berserk => data_writer.put_u8(1),
                    BodyState::Untouchable => data_writer.put_u8(2),
                    BodyState::GM_Invincible => data_writer.put_u8(3),
                    BodyState::GM_Invisible => data_writer.put_u8(4),
                    BodyState::Berserk2 => data_writer.put_u8(5),
                    BodyState::Stealth => data_writer.put_u8(6),
                    BodyState::Invisible => data_writer.put_u8(7),
                }
                data_writer.put_u8(entity_state.unknown2);
                data_writer.put_f32_le(entity_state.walk_speed);
                data_writer.put_f32_le(entity_state.run_speed);
                data_writer.put_f32_le(entity_state.berserk_speed);
                data_writer.put_u8(entity_state.active_buffs.len() as u8);
                for element in entity_state.active_buffs.iter() {
                    data_writer.put_u32_le(element.id);
                    data_writer.put_u32_le(element.token);
                }
                match &interaction_options {
                    InteractOptions::None => data_writer.put_u8(0),
                    InteractOptions::Talk { options,  } => {
                        data_writer.put_u8(2);
                        data_writer.put_u8(options.len() as u8);
                        for element in options.iter() {
                            data_writer.put_u8(*element);
                        }
                    }
                }
                match &rarity {
                    EntityRarity::Normal => data_writer.put_u8(0),
                    EntityRarity::Champion => data_writer.put_u8(1),
                    EntityRarity::Unique => data_writer.put_u8(3),
                    EntityRarity::Giant => data_writer.put_u8(4),
                    EntityRarity::Titan => data_writer.put_u8(5),
                    EntityRarity::Elite => data_writer.put_u8(6),
                    EntityRarity::Elite_String => data_writer.put_u8(7),
                    EntityRarity::Unique2 => data_writer.put_u8(8),
                    EntityRarity::NormalParty => data_writer.put_u8(16),
                    EntityRarity::ChampionParty => data_writer.put_u8(17),
                    EntityRarity::UniqueParty => data_writer.put_u8(19),
                    EntityRarity::GiantParty => data_writer.put_u8(20),
                    EntityRarity::TitanParty => data_writer.put_u8(21),
                    EntityRarity::EliteParty => data_writer.put_u8(22),
                    EntityRarity::Unique2Party => data_writer.put_u8(24),
                }
                data_writer.put_u32_le(*unknown);
            }
        }
        data_writer.put_u8(op.unknown_3);
        data_writer.put_u32_le(op.unknown_4);
        data_writer.put_u8(op.unknown_5);
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for EntitySpawn {
    fn into(self) -> ServerPacket {
        ServerPacket::EntitySpawn(self)
    }
}

impl EntitySpawn {
    pub fn new(spawn_data: EntityTypeSpawnData) -> Self {
        EntitySpawn { spawn_data, unknown_3: 5, unknown_4: 0, unknown_5: 4,  }
    }
}

#[derive(Clone)]
pub struct GroupEntitySpawnStart {
    pub kind: GroupSpawnType,
    pub amount: u16,
    pub unknown_1: u32,
    pub unknown_2: u16,
}

impl From<GroupEntitySpawnStart> for Bytes {
    fn from(op: GroupEntitySpawnStart) -> Bytes {
        let mut data_writer = BytesMut::new();
        match &op.kind {
            GroupSpawnType::Spawn => data_writer.put_u8(1),
            GroupSpawnType::Despawn => data_writer.put_u8(2),
        }
        data_writer.put_u16_le(op.amount);
        data_writer.put_u32_le(op.unknown_1);
        data_writer.put_u16_le(op.unknown_2);
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for GroupEntitySpawnStart {
    fn into(self) -> ServerPacket {
        ServerPacket::GroupEntitySpawnStart(self)
    }
}

impl GroupEntitySpawnStart {
    pub fn new(kind: GroupSpawnType, amount: u16) -> Self {
        GroupEntitySpawnStart { kind, amount, unknown_1: 0, unknown_2: 0,  }
    }
}

#[derive(Clone)]
pub struct GroupEntitySpawnData {
    pub content: Vec<GroupSpawnDataContent>,
}

impl From<GroupEntitySpawnData> for Bytes {
    fn from(op: GroupEntitySpawnData) -> Bytes {
        let mut data_writer = BytesMut::new();
        for element in op.content.iter() {
            match &element {
                GroupSpawnDataContent::Despawn { id,  } => {
                    data_writer.put_u32_le(*id);
                }
                GroupSpawnDataContent::Spawn { object_id, data,  } => {
                    data_writer.put_u32_le(*object_id);
                    match &data {
                        EntityTypeSpawnData::Item => {},
                        EntityTypeSpawnData::Gold { amount, unique_id, position, owner, rarity,  } => {
                            data_writer.put_u32_le(*amount);
                            data_writer.put_u32_le(*unique_id);
                            data_writer.put_u16_le(position.region);
                            data_writer.put_f32_le(position.pos_x);
                            data_writer.put_f32_le(position.pos_y);
                            data_writer.put_f32_le(position.pos_z);
                            data_writer.put_u16_le(position.heading);
                            if let Some(owner) = &owner {
                                data_writer.put_u8(1);
                                data_writer.put_u32_le(*owner);
                            }
                            else {
                                data_writer.put_u8(0);
                            }
                            data_writer.put_u8(*rarity);
                        }
                        EntityTypeSpawnData::Character { scale, berserk_level, pvp_cape, beginner, title, equipment, avatar_items, mask, movement, entity_state, name, job_type, job_level, pk_state, mounted, in_combat, active_scroll, unknown2,  } => {
                            data_writer.put_u8(*scale);
                            data_writer.put_u8(*berserk_level);
                            match &pvp_cape {
                                PvpCape::None => data_writer.put_u8(0),
                                PvpCape::Red => data_writer.put_u8(1),
                                PvpCape::Gray => data_writer.put_u8(2),
                                PvpCape::Blue => data_writer.put_u8(3),
                                PvpCape::White => data_writer.put_u8(4),
                                PvpCape::Yellow => data_writer.put_u8(5),
                            }
                            data_writer.put_u8(*beginner as u8);
                            data_writer.put_u8(*title);
                            data_writer.put_u8(equipment.len() as u8);
                            for element in equipment.iter() {
                                data_writer.put_u32_le(*element);
                            }
                            data_writer.put_u8(avatar_items.len() as u8);
                            for element in avatar_items.iter() {
                                data_writer.put_u32_le(*element);
                            }
                            if let Some(mask) = &mask {
                                data_writer.put_u8(1);
                                data_writer.put_u32_le(*mask);
                            }
                            else {
                                data_writer.put_u8(0);
                            }
                            match &movement {
                                EntityMovementState::Moving { movement_type, region, x, y, z,  } => {
                                    data_writer.put_u8(1);
                                    match &movement_type {
                                        MovementType::Running => data_writer.put_u8(0),
                                        MovementType::Walking => data_writer.put_u8(1),
                                    }
                                    data_writer.put_u16_le(*region);
                                    data_writer.put_u16_le(*x);
                                    data_writer.put_u16_le(*y);
                                    data_writer.put_u16_le(*z);
                                }
                                EntityMovementState::Standing { movement_type, unknown, angle,  } => {
                                    data_writer.put_u8(0);
                                    match &movement_type {
                                        MovementType::Running => data_writer.put_u8(0),
                                        MovementType::Walking => data_writer.put_u8(1),
                                    }
                                    data_writer.put_u8(*unknown);
                                    data_writer.put_u16_le(*angle);
                                }
                            }
                            match &entity_state.alive {
                                AliveState::Spawning => data_writer.put_u8(0),
                                AliveState::Alive => data_writer.put_u8(1),
                                AliveState::Dead => data_writer.put_u8(2),
                            }
                            data_writer.put_u8(entity_state.unknown1);
                            match &entity_state.action_state {
                                ActionState::None => data_writer.put_u8(0),
                                ActionState::Walking => data_writer.put_u8(2),
                                ActionState::Running => data_writer.put_u8(3),
                                ActionState::Sitting => data_writer.put_u8(4),
                            }
                            match &entity_state.body_state {
                                BodyState::None => data_writer.put_u8(0),
                                BodyState::Berserk => data_writer.put_u8(1),
                                BodyState::Untouchable => data_writer.put_u8(2),
                                BodyState::GM_Invincible => data_writer.put_u8(3),
                                BodyState::GM_Invisible => data_writer.put_u8(4),
                                BodyState::Berserk2 => data_writer.put_u8(5),
                                BodyState::Stealth => data_writer.put_u8(6),
                                BodyState::Invisible => data_writer.put_u8(7),
                            }
                            data_writer.put_u8(entity_state.unknown2);
                            data_writer.put_f32_le(entity_state.walk_speed);
                            data_writer.put_f32_le(entity_state.run_speed);
                            data_writer.put_f32_le(entity_state.berserk_speed);
                            data_writer.put_u8(entity_state.active_buffs.len() as u8);
                            for element in entity_state.active_buffs.iter() {
                                data_writer.put_u32_le(element.id);
                                data_writer.put_u32_le(element.token);
                            }
                            data_writer.put_u16_le(name.len() as u16);
                            data_writer.put_slice(name.as_bytes());
                            match &job_type {
                                JobType::None => data_writer.put_u8(0),
                                JobType::Trader => data_writer.put_u8(1),
                                JobType::Thief => data_writer.put_u8(2),
                                JobType::Hunter => data_writer.put_u8(3),
                            }
                            data_writer.put_u8(*job_level);
                            match &pk_state {
                                PlayerKillState::None => data_writer.put_u8(0),
                                PlayerKillState::Purple => data_writer.put_u8(1),
                                PlayerKillState::Red => data_writer.put_u8(2),
                            }
                            data_writer.put_u8(*mounted as u8);
                            data_writer.put_u8(*in_combat as u8);
                            match &active_scroll {
                                ActiveScroll::None => data_writer.put_u8(0),
                                ActiveScroll::ReturnScroll => data_writer.put_u8(1),
                                ActiveScroll::JobScroll => data_writer.put_u8(2),
                            }
                            data_writer.put_u8(*unknown2);
                        }
                        EntityTypeSpawnData::Monster { unique_id, position, movement, entity_state, interaction_options, rarity, unknown,  } => {
                            data_writer.put_u32_le(*unique_id);
                            data_writer.put_u16_le(position.region);
                            data_writer.put_f32_le(position.pos_x);
                            data_writer.put_f32_le(position.pos_y);
                            data_writer.put_f32_le(position.pos_z);
                            data_writer.put_u16_le(position.heading);
                            match &movement {
                                EntityMovementState::Moving { movement_type, region, x, y, z,  } => {
                                    data_writer.put_u8(1);
                                    match &movement_type {
                                        MovementType::Running => data_writer.put_u8(0),
                                        MovementType::Walking => data_writer.put_u8(1),
                                    }
                                    data_writer.put_u16_le(*region);
                                    data_writer.put_u16_le(*x);
                                    data_writer.put_u16_le(*y);
                                    data_writer.put_u16_le(*z);
                                }
                                EntityMovementState::Standing { movement_type, unknown, angle,  } => {
                                    data_writer.put_u8(0);
                                    match &movement_type {
                                        MovementType::Running => data_writer.put_u8(0),
                                        MovementType::Walking => data_writer.put_u8(1),
                                    }
                                    data_writer.put_u8(*unknown);
                                    data_writer.put_u16_le(*angle);
                                }
                            }
                            match &entity_state.alive {
                                AliveState::Spawning => data_writer.put_u8(0),
                                AliveState::Alive => data_writer.put_u8(1),
                                AliveState::Dead => data_writer.put_u8(2),
                            }
                            data_writer.put_u8(entity_state.unknown1);
                            match &entity_state.action_state {
                                ActionState::None => data_writer.put_u8(0),
                                ActionState::Walking => data_writer.put_u8(2),
                                ActionState::Running => data_writer.put_u8(3),
                                ActionState::Sitting => data_writer.put_u8(4),
                            }
                            match &entity_state.body_state {
                                BodyState::None => data_writer.put_u8(0),
                                BodyState::Berserk => data_writer.put_u8(1),
                                BodyState::Untouchable => data_writer.put_u8(2),
                                BodyState::GM_Invincible => data_writer.put_u8(3),
                                BodyState::GM_Invisible => data_writer.put_u8(4),
                                BodyState::Berserk2 => data_writer.put_u8(5),
                                BodyState::Stealth => data_writer.put_u8(6),
                                BodyState::Invisible => data_writer.put_u8(7),
                            }
                            data_writer.put_u8(entity_state.unknown2);
                            data_writer.put_f32_le(entity_state.walk_speed);
                            data_writer.put_f32_le(entity_state.run_speed);
                            data_writer.put_f32_le(entity_state.berserk_speed);
                            data_writer.put_u8(entity_state.active_buffs.len() as u8);
                            for element in entity_state.active_buffs.iter() {
                                data_writer.put_u32_le(element.id);
                                data_writer.put_u32_le(element.token);
                            }
                            match &interaction_options {
                                InteractOptions::None => data_writer.put_u8(0),
                                InteractOptions::Talk { options,  } => {
                                    data_writer.put_u8(2);
                                    data_writer.put_u8(options.len() as u8);
                                    for element in options.iter() {
                                        data_writer.put_u8(*element);
                                    }
                                }
                            }
                            match &rarity {
                                EntityRarity::Normal => data_writer.put_u8(0),
                                EntityRarity::Champion => data_writer.put_u8(1),
                                EntityRarity::Unique => data_writer.put_u8(3),
                                EntityRarity::Giant => data_writer.put_u8(4),
                                EntityRarity::Titan => data_writer.put_u8(5),
                                EntityRarity::Elite => data_writer.put_u8(6),
                                EntityRarity::Elite_String => data_writer.put_u8(7),
                                EntityRarity::Unique2 => data_writer.put_u8(8),
                                EntityRarity::NormalParty => data_writer.put_u8(16),
                                EntityRarity::ChampionParty => data_writer.put_u8(17),
                                EntityRarity::UniqueParty => data_writer.put_u8(19),
                                EntityRarity::GiantParty => data_writer.put_u8(20),
                                EntityRarity::TitanParty => data_writer.put_u8(21),
                                EntityRarity::EliteParty => data_writer.put_u8(22),
                                EntityRarity::Unique2Party => data_writer.put_u8(24),
                            }
                            data_writer.put_u32_le(*unknown);
                        }
                    }
                }
            }
        }
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for GroupEntitySpawnData {
    fn into(self) -> ServerPacket {
        ServerPacket::GroupEntitySpawnData(self)
    }
}

impl GroupEntitySpawnData {
    pub fn new(content: Vec<GroupSpawnDataContent>) -> Self {
        GroupEntitySpawnData { content,  }
    }
}

#[derive(Clone)]
pub struct GroupEntitySpawnEnd;

impl From<GroupEntitySpawnEnd> for Bytes {
    fn from(op: GroupEntitySpawnEnd) -> Bytes {
        let mut data_writer = BytesMut::new();
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for GroupEntitySpawnEnd {
    fn into(self) -> ServerPacket {
        ServerPacket::GroupEntitySpawnEnd(self)
    }
}

impl GroupEntitySpawnEnd {
    pub fn new() -> Self {
        GroupEntitySpawnEnd {  }
    }
}

#[derive(Clone)]
pub struct ConsignmentList;

impl TryFrom<Bytes> for ConsignmentList {
    type Error = ProtocolError;

    fn try_from(data: Bytes) -> Result<Self, Self::Error> {
        let mut data_reader = data.reader();
        Ok(ConsignmentList {  })
    }
}

impl Into<ClientPacket> for ConsignmentList {
    fn into(self) -> ClientPacket {
        ClientPacket::ConsignmentList(self)
    }
}

#[derive(Clone)]
pub struct ConsignmentResponse {
    pub result: ConsignmentResult,
}

impl From<ConsignmentResponse> for Bytes {
    fn from(op: ConsignmentResponse) -> Bytes {
        let mut data_writer = BytesMut::new();
        match &op.result {
            ConsignmentResult::Success { items,  } => {
                data_writer.put_u8(1);
                data_writer.put_u8(items.len() as u8);
                for element in items.iter() {
                    data_writer.put_u32_le(element.personal_id);
                    data_writer.put_u8(element.status);
                    data_writer.put_u32_le(element.ref_item_id);
                    data_writer.put_u32_le(element.sell_count);
                    data_writer.put_u64_le(element.price);
                    data_writer.put_u64_le(element.deposit);
                    data_writer.put_u64_le(element.fee);
                    data_writer.put_u32_le(element.end_date);
                }
            }
            ConsignmentResult::Error { code,  } => {
                data_writer.put_u8(2);
                match &code {
                    ConsignmentErrorCode::NotEnoughGold => data_writer.put_u16_le(0x700D),
                }
            }
        }
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for ConsignmentResponse {
    fn into(self) -> ServerPacket {
        ServerPacket::ConsignmentResponse(self)
    }
}

impl ConsignmentResponse {
    pub fn new(result: ConsignmentResult) -> Self {
        ConsignmentResponse { result,  }
    }
}

#[derive(Clone)]
pub struct WeatherUpdate {
    pub kind: WeatherType,
    pub speed: u8,
}

impl From<WeatherUpdate> for Bytes {
    fn from(op: WeatherUpdate) -> Bytes {
        let mut data_writer = BytesMut::new();
        match &op.kind {
            WeatherType::Clear => data_writer.put_u8(1),
            WeatherType::Rain => data_writer.put_u8(2),
            WeatherType::Snow => data_writer.put_u8(3),
        }
        data_writer.put_u8(op.speed);
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for WeatherUpdate {
    fn into(self) -> ServerPacket {
        ServerPacket::WeatherUpdate(self)
    }
}

impl WeatherUpdate {
    pub fn new(kind: WeatherType, speed: u8) -> Self {
        WeatherUpdate { kind, speed,  }
    }
}

#[derive(Clone)]
pub struct FriendListInfo {
    pub groups: Vec<FriendListGroup>,
    pub friends: Vec<FriendListEntry>,
}

impl From<FriendListInfo> for Bytes {
    fn from(op: FriendListInfo) -> Bytes {
        let mut data_writer = BytesMut::new();
        data_writer.put_u8(op.groups.len() as u8);
        for element in op.groups.iter() {
            data_writer.put_u16_le(element.id);
            data_writer.put_u16_le(element.name.len() as u16);
            data_writer.put_slice(element.name.as_bytes());
        }
        data_writer.put_u8(op.friends.len() as u8);
        for element in op.friends.iter() {
            data_writer.put_u32_le(element.char_id);
            data_writer.put_u16_le(element.name.len() as u16);
            data_writer.put_slice(element.name.as_bytes());
            data_writer.put_u32_le(element.char_model);
            data_writer.put_u16_le(element.group_id);
            data_writer.put_u8(element.offline as u8);
        }
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for FriendListInfo {
    fn into(self) -> ServerPacket {
        ServerPacket::FriendListInfo(self)
    }
}

impl FriendListInfo {
    pub fn new(groups: Vec<FriendListGroup>, friends: Vec<FriendListEntry>) -> Self {
        FriendListInfo { groups, friends,  }
    }
}

#[derive(Clone)]
pub struct GameNotification {
    pub result: GameNotificationContent,
}

impl From<GameNotification> for Bytes {
    fn from(op: GameNotification) -> Bytes {
        let mut data_writer = BytesMut::new();
        match &op.result {
            GameNotificationContent::UniqueSpawned { unknown, ref_id,  } => {
                data_writer.put_u8(0x05);
                data_writer.put_u8(*unknown);
                data_writer.put_u16_le(*ref_id);
            }
            GameNotificationContent::UniqueKilled { ref_id,  } => {
                data_writer.put_u8(0x06);
                data_writer.put_u16_le(*ref_id);
            }
        }
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for GameNotification {
    fn into(self) -> ServerPacket {
        ServerPacket::GameNotification(self)
    }
}

impl GameNotification {
    pub fn new(result: GameNotificationContent) -> Self {
        GameNotification { result,  }
    }
}

#[derive(Clone)]
pub struct PlayerMovementRequest {
    pub kind: MovementTarget,
}

impl TryFrom<Bytes> for PlayerMovementRequest {
    type Error = ProtocolError;

    fn try_from(data: Bytes) -> Result<Self, Self::Error> {
        let mut data_reader = data.reader();
        let kind = match data_reader.read_u8()? {
            1 =>  {
                let region = data_reader.read_u16::<byteorder::LittleEndian>()?;
                let x = data_reader.read_u16::<byteorder::LittleEndian>()?;
                let y = data_reader.read_u16::<byteorder::LittleEndian>()?;
                let z = data_reader.read_u16::<byteorder::LittleEndian>()?;
                MovementTarget::TargetLocation { region, x, y, z,  }
            }
            0 =>  {
                let unknown = data_reader.read_u8()?;
                let angle = data_reader.read_u16::<byteorder::LittleEndian>()?;
                MovementTarget::Direction { unknown, angle,  }
            }
            unknown => return Err(ProtocolError::UnknownVariation(unknown, "MovementTarget"))
        };
        Ok(PlayerMovementRequest { kind,  })
    }
}

impl Into<ClientPacket> for PlayerMovementRequest {
    fn into(self) -> ClientPacket {
        ClientPacket::PlayerMovementRequest(self)
    }
}

#[derive(Clone)]
pub struct PlayerMovementResponse {
    pub player_id: u32,
    pub unknown: u8,
    pub region: u16,
    pub x: u16,
    pub y: u16,
    pub z: u16,
    pub source_position: Option<MovementSource>,
}

impl From<PlayerMovementResponse> for Bytes {
    fn from(op: PlayerMovementResponse) -> Bytes {
        let mut data_writer = BytesMut::new();
        data_writer.put_u32_le(op.player_id);
        data_writer.put_u8(op.unknown);
        data_writer.put_u16_le(op.region);
        data_writer.put_u16_le(op.x);
        data_writer.put_u16_le(op.y);
        data_writer.put_u16_le(op.z);
        if let Some(source_position) = &op.source_position {
            data_writer.put_u8(1);
            data_writer.put_u16_le(source_position.region);
            data_writer.put_u16_le(source_position.x);
            data_writer.put_f32_le(source_position.y);
            data_writer.put_u16_le(source_position.z);
        }
        else {
            data_writer.put_u8(0);
        }
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for PlayerMovementResponse {
    fn into(self) -> ServerPacket {
        ServerPacket::PlayerMovementResponse(self)
    }
}

impl PlayerMovementResponse {
    pub fn new(player_id: u32, region: u16, x: u16, y: u16, z: u16, source_position: Option<MovementSource>) -> Self {
        PlayerMovementResponse { player_id, unknown: 1, region, x, y, z, source_position,  }
    }
}

#[derive(Clone)]
pub struct AddFriend {
    pub name: String,
}

impl TryFrom<Bytes> for AddFriend {
    type Error = ProtocolError;

    fn try_from(data: Bytes) -> Result<Self, Self::Error> {
        let mut data_reader = data.reader();
        let name_string_len = data_reader.read_u16::<byteorder::LittleEndian>()?;
        let mut name_bytes = Vec::with_capacity(name_string_len as usize);
        for _ in 0..name_string_len {
            	name_bytes.push(data_reader.read_u8()?);
        }
        let name = String::from_utf8(name_bytes)?;
        Ok(AddFriend { name,  })
    }
}

impl Into<ClientPacket> for AddFriend {
    fn into(self) -> ClientPacket {
        ClientPacket::AddFriend(self)
    }
}

#[derive(Clone)]
pub struct CreateFriendGroup {
    pub name: String,
}

impl TryFrom<Bytes> for CreateFriendGroup {
    type Error = ProtocolError;

    fn try_from(data: Bytes) -> Result<Self, Self::Error> {
        let mut data_reader = data.reader();
        let name_string_len = data_reader.read_u16::<byteorder::LittleEndian>()?;
        let mut name_bytes = Vec::with_capacity(name_string_len as usize);
        for _ in 0..name_string_len {
            	name_bytes.push(data_reader.read_u8()?);
        }
        let name = String::from_utf8(name_bytes)?;
        Ok(CreateFriendGroup { name,  })
    }
}

impl Into<ClientPacket> for CreateFriendGroup {
    fn into(self) -> ClientPacket {
        ClientPacket::CreateFriendGroup(self)
    }
}

#[derive(Clone)]
pub struct DeleteFriend {
    pub friend_character_id: u32,
}

impl TryFrom<Bytes> for DeleteFriend {
    type Error = ProtocolError;

    fn try_from(data: Bytes) -> Result<Self, Self::Error> {
        let mut data_reader = data.reader();
        let friend_character_id = data_reader.read_u32::<byteorder::LittleEndian>()?;
        Ok(DeleteFriend { friend_character_id,  })
    }
}

impl Into<ClientPacket> for DeleteFriend {
    fn into(self) -> ClientPacket {
        ClientPacket::DeleteFriend(self)
    }
}

#[derive(Clone)]
pub struct Rotation {
    pub heading: u16,
}

impl TryFrom<Bytes> for Rotation {
    type Error = ProtocolError;

    fn try_from(data: Bytes) -> Result<Self, Self::Error> {
        let mut data_reader = data.reader();
        let heading = data_reader.read_u16::<byteorder::LittleEndian>()?;
        Ok(Rotation { heading,  })
    }
}

impl Into<ClientPacket> for Rotation {
    fn into(self) -> ClientPacket {
        ClientPacket::Rotation(self)
    }
}

#[derive(Clone)]
pub struct EntityUpdateState {
    pub unique_id: u32,
    pub kind: u8,
    pub value: u8,
}

impl From<EntityUpdateState> for Bytes {
    fn from(op: EntityUpdateState) -> Bytes {
        let mut data_writer = BytesMut::new();
        data_writer.put_u32_le(op.unique_id);
        data_writer.put_u8(op.kind);
        data_writer.put_u8(op.value);
        data_writer.freeze()
    }
}

impl Into<ServerPacket> for EntityUpdateState {
    fn into(self) -> ServerPacket {
        ServerPacket::EntityUpdateState(self)
    }
}

impl EntityUpdateState {
    pub fn new(unique_id: u32, kind: u8, value: u8) -> Self {
        EntityUpdateState { unique_id, kind, value,  }
    }
}