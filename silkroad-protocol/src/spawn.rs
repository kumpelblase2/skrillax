use crate::community::GuildInformation;
use crate::inventory::{BagContent, CharacterSpawnItemData};
use crate::movement::{EntityMovementState, Position};
use crate::skill::{HotkeyData, MasteryData, SkillData};
use crate::world::{ActiveScroll, EntityState, InteractOptions, JobType, PlayerKillState, PvpCape};
use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use silkroad_definitions::rarity::EntityRarity;
use skrillax_packet::Packet;
use skrillax_serde::*;

#[derive(Clone, Eq, PartialEq, Copy, Serialize, ByteSize)]
pub enum GroupSpawnType {
    #[silkroad(value = 1)]
    Spawn,
    #[silkroad(value = 2)]
    Despawn,
}

#[derive(Copy, Clone, Serialize, ByteSize)]
pub enum DroppedItemSource {
    #[silkroad(value = 0)]
    None,
    #[silkroad(value = 5)]
    Monster,
    #[silkroad(value = 6)]
    Player,
}

#[derive(Clone, Serialize, ByteSize)]
#[silkroad(size = 0)]
pub enum ItemSpawnData {
    Gold {
        amount: u32,
        unique_id: u32,
        position: Position,
        owner: Option<u32>,
        rarity: u8,
    },
    Consumable {
        unique_id: u32,
        position: Position,
        owner: Option<u32>,
        rarity: u8,
        source: DroppedItemSource,
        source_id: u32,
    },
    Equipment {
        upgrade: u8,
        unique_id: u32,
        position: Position,
        owner: Option<u32>,
        rarity: u8,
        source: DroppedItemSource,
        source_id: u32,
    },
}

#[derive(Clone, Serialize, ByteSize, Packet)]
#[packet(opcode = 0x34A5)]
pub struct CharacterSpawnStart;

#[derive(Clone, Serialize, ByteSize, Deserialize, Copy)]
pub struct ServiceEndTime {
    day: u8,
    year: u8,
    month: u8,
    hour: u8,
    minute: u8,
}

impl<T: TimeZone> From<DateTime<T>> for ServiceEndTime {
    fn from(value: DateTime<T>) -> Self {
        ServiceEndTime {
            day: value.day() as u8,
            year: (value.year() - 2000) as u8,
            month: value.month0() as u8,
            hour: value.hour() as u8,
            minute: value.minute() as u8,
        }
    }
}

#[derive(Clone, Serialize, ByteSize, Deserialize)]
pub struct CollectionBookTheme {
    pub index: u32,
    pub start: SilkroadTime,
    pub pages: u32,
}

#[derive(Clone, Serialize, ByteSize, Deserialize)]
pub struct JobInformation {
    pub job_name: String,
    pub job_rank: u8,
    pub job_title: u8,
    pub job_type: JobType,
    pub job_level: u8,
    pub job_exp: u64,
}

impl JobInformation {
    pub fn empty() -> Self {
        Self {
            job_name: String::new(),
            job_rank: 0,
            job_title: 0,
            job_type: JobType::None,
            job_level: 0,
            job_exp: 0,
        }
    }
}

#[derive(Clone, Serialize, ByteSize, Packet)]
#[packet(opcode = 0x3013)]
pub struct CharacterSpawn {
    pub time: SilkroadTime,
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
    // pub fortress_war_mark: u8,
    pub service_end: ServiceEndTime,
    pub user_type: u8,
    pub server_max_level: u8,
    pub unknown_2: u16,
    pub inventory: BagContent,
    pub avatar_items: BagContent,
    pub specialty_bag: BagContent,
    pub job_bag: BagContent,
    pub unknown_5: u8,
    #[silkroad(list_type = "break")]
    pub masteries: Vec<MasteryData>,
    pub unknown_6: u8,
    #[silkroad(list_type = "break")]
    pub skills: Vec<SkillData>,
    #[silkroad(size = 2)]
    pub completed_quests: Vec<u32>,
    pub active_quests: Vec<ActiveQuestData>,
    pub unknown_8: u8,
    #[silkroad(size = 4)]
    pub collection_book: Vec<CollectionBookTheme>,
    pub unique_id: u32,
    pub position: Position,
    pub movement: EntityMovementState,
    pub entity_state: EntityState,
    pub character_name: String,
    pub job_information: JobInformation,
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

impl CharacterSpawn {
    pub fn new(
        time: SilkroadTime,
        ref_id: u32,
        scale: u8,
        level: u8,
        max_level: u8,
        exp: u64,
        sp_exp: u32,
        gold: u64,
        sp: u32,
        stat_points: u16,
        berserk_points: u8,
        hp: u32,
        mp: u32,
        beginner: bool,
        player_kills_today: u8,
        player_kills_total: u16,
        player_kills_penalty: u32,
        berserk_level: u8,
        free_pvp: u8,
        fortress_war_mark: u8,
        service_end: DateTime<Utc>,
        user_type: u8,
        server_max_level: u8,
        inventory: BagContent,
        avatar_items: BagContent,
        masteries: Vec<MasteryData>,
        skills: Vec<SkillData>,
        completed_quests: Vec<u32>,
        active_quests: Vec<ActiveQuestData>,
        unique_id: u32,
        position: Position,
        movement: EntityMovementState,
        entity_state: EntityState,
        character_name: String,
        job_information: JobInformation,
        job_contribution: u32,
        job_reward: u32,
        pvp_state: u8,
        transport_flag: bool,
        in_combat: u8,
        pvp_flag: u8,
        jid: u32,
        gm: bool,
        hotkeys: Vec<HotkeyData>,
        auto_hp: u16,
        auto_mp: u16,
        auto_pill: u16,
        potion_delay: u8,
        blocked_players: Vec<String>,
    ) -> Self {
        CharacterSpawn {
            time,
            ref_id,
            scale,
            level,
            max_level,
            exp,
            sp_exp,
            gold,
            sp,
            stat_points,
            berserk_points,
            unknown_1: 0,
            hp,
            mp,
            beginner,
            player_kills_today,
            player_kills_total,
            player_kills_penalty,
            berserk_level,
            free_pvp,
            // fortress_war_mark,
            service_end: service_end.into(),
            user_type,
            server_max_level,
            unknown_2: 0x0107,
            inventory,
            avatar_items,
            specialty_bag: BagContent::empty(),
            job_bag: BagContent::new(0xb, Vec::new()),
            unknown_5: 0,
            masteries,
            unknown_6: 0,
            skills,
            completed_quests,
            active_quests,
            unknown_8: 0,
            collection_book: Vec::new(),
            unique_id,
            position,
            movement,
            entity_state,
            character_name,
            job_information,
            job_reward,
            pvp_state,
            transport_flag,
            in_combat,
            unknown_15: 0,
            unknown_16: 0,
            pvp_flag,
            unknown_17: 0xFF,
            unknown_18: 0x8000d7,
            jid,
            gm,
            unknown_19: 0x19,
            hotkeys,
            unknown_20: 0,
            auto_hp,
            auto_mp,
            auto_pill,
            potion_delay,
            blocked_players,
            unknown_21: 0x9f000000,
        }
    }
}

#[derive(Clone, Serialize, ByteSize, Packet)]
#[packet(opcode = 0x34A6)]
pub struct CharacterSpawnEnd;

#[derive(Clone, Serialize, ByteSize, Packet)]
#[packet(opcode = 0x3016)]
pub struct EntityDespawn {
    pub entity_id: u32,
}

impl EntityDespawn {
    pub fn new(entity_id: u32) -> Self {
        EntityDespawn { entity_id }
    }
}

#[derive(Clone, Serialize, ByteSize, Packet)]
#[packet(opcode = 0x3015)]
pub struct EntitySpawn {
    pub ref_id: u32,
    pub spawn_data: EntityTypeSpawnData,
}

impl EntitySpawn {
    pub fn new(ref_id: u32, spawn_data: EntityTypeSpawnData) -> Self {
        EntitySpawn { ref_id, spawn_data }
    }
}

#[derive(Clone, Serialize, ByteSize, Packet)]
#[packet(opcode = 0x3017)]
pub struct GroupEntitySpawnStart {
    pub kind: GroupSpawnType,
    pub amount: u16,
    pub unknown_1: u32,
    pub unknown_2: u16,
}

impl GroupEntitySpawnStart {
    pub fn new(kind: GroupSpawnType, amount: u16) -> Self {
        GroupEntitySpawnStart {
            kind,
            amount,
            unknown_1: 0,
            unknown_2: 0,
        }
    }
}

#[derive(Clone, Serialize, ByteSize, Packet)]
#[packet(opcode = 0x3019)]
pub struct GroupEntitySpawnData {
    #[silkroad(list_type = "none")]
    pub content: Vec<GroupSpawnDataContent>,
}

impl GroupEntitySpawnData {
    pub fn new(content: Vec<GroupSpawnDataContent>) -> Self {
        GroupEntitySpawnData { content }
    }
}

#[derive(Clone, Serialize, ByteSize, Packet)]
#[packet(opcode = 0x3018)]
pub struct GroupEntitySpawnEnd;

#[derive(Clone, Serialize, ByteSize)]
#[silkroad(size = 0)]
pub enum GroupSpawnDataContent {
    Despawn { id: u32 },
    Spawn { object_id: u32, data: EntityTypeSpawnData },
}

impl GroupSpawnDataContent {
    pub fn despawn(id: u32) -> Self {
        GroupSpawnDataContent::Despawn { id }
    }

    pub fn spawn(object_id: u32, data: EntityTypeSpawnData) -> Self {
        GroupSpawnDataContent::Spawn { object_id, data }
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub struct ActiveQuestData {
    pub id: u32,
    pub repeat_count: u8,
    pub unknown_1: u8,
    pub unknown_2: u16,
    pub kind: u8,
    pub status: u8,
    pub objectives: Vec<ActiveQuestObjectData>,
}

impl ActiveQuestData {
    pub fn new(id: u32, repeat_count: u8, kind: u8, status: u8, objectives: Vec<ActiveQuestObjectData>) -> Self {
        ActiveQuestData {
            id,
            repeat_count,
            unknown_1: 1,
            unknown_2: 0,
            kind,
            status,
            objectives,
        }
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub struct ActiveQuestObjectData {
    pub index: u8,
    pub incomplete: bool,
    pub name: String,
    pub tasks: Vec<u32>,
    pub task_ids: Vec<u32>,
}

impl ActiveQuestObjectData {
    pub fn new(index: u8, incomplete: bool, name: String, tasks: Vec<u32>, task_ids: Vec<u32>) -> Self {
        ActiveQuestObjectData {
            index,
            incomplete,
            name,
            tasks,
            task_ids,
        }
    }
}

#[derive(Clone, Serialize, ByteSize)]
#[silkroad(size = 0)]
pub enum EntityTypeSpawnData {
    Item(ItemSpawnData),
    Character {
        scale: u8,
        berserk_level: u8,
        pvp_cape: PvpCape,
        beginner: bool,
        title: u8,
        inventory_size: u8,
        equipment: Vec<CharacterSpawnItemData>,
        avatar_inventory_size: u8,
        avatar_items: Vec<CharacterSpawnItemData>,
        mask: Option<u32>,
        unique_id: u32,
        position: Position,
        movement: EntityMovementState,
        entity_state: EntityState,
        name: String,
        job_type: JobType,
        mounted: bool,
        in_combat: bool,
        active_scroll: ActiveScroll,
        unknown2: u8,
        guild: GuildInformation,
        unknown3: [u8; 11],
        equipment_cooldown: bool,
        pk_state: PlayerKillState,
        unknown4: u8,
        unknown5: u8,
    },
    NPC {
        unique_id: u32,
        position: Position,
        movement: EntityMovementState,
        entity_state: EntityState,
        interaction_options: InteractOptions,
    },
    Monster {
        unique_id: u32,
        position: Position,
        movement: EntityMovementState,
        entity_state: EntityState,
        interaction_options: InteractOptions,
        rarity: EntityRarity,
        unknown: u32,
    },
}

impl EntityTypeSpawnData {
    pub fn gold(amount: u32, unique_id: u32, position: Position, owner: Option<u32>, rarity: u8) -> Self {
        EntityTypeSpawnData::Item(ItemSpawnData::Gold {
            amount,
            unique_id,
            position,
            owner,
            rarity,
        })
    }

    pub fn character(
        scale: u8,
        berserk_level: u8,
        pvp_cape: PvpCape,
        beginner: bool,
        title: u8,
        inventory_size: u8,
        equipment: Vec<CharacterSpawnItemData>,
        avatar_inventory_size: u8,
        avatar_items: Vec<CharacterSpawnItemData>,
        mask: Option<u32>,
        unique_id: u32,
        position: Position,
        movement: EntityMovementState,
        entity_state: EntityState,
        name: String,
        job_type: JobType,
        mounted: bool,
        in_combat: bool,
        active_scroll: ActiveScroll,
        guild: GuildInformation,
        unknown3: [u8; 11],
        equipment_cooldown: bool,
        pk_state: PlayerKillState,
    ) -> Self {
        EntityTypeSpawnData::Character {
            scale,
            berserk_level,
            pvp_cape,
            beginner,
            title,
            inventory_size,
            equipment,
            avatar_inventory_size,
            avatar_items,
            mask,
            unique_id,
            position,
            movement,
            entity_state,
            name,
            job_type,
            mounted,
            in_combat,
            active_scroll,
            unknown2: 0,
            guild,
            unknown3,
            equipment_cooldown,
            pk_state,
            unknown4: 0xFF,
            unknown5: 0x01,
        }
    }

    pub fn monster(
        unique_id: u32,
        position: Position,
        movement: EntityMovementState,
        entity_state: EntityState,
        interaction_options: InteractOptions,
        rarity: EntityRarity,
        unknown: u32,
    ) -> Self {
        EntityTypeSpawnData::Monster {
            unique_id,
            position,
            movement,
            entity_state,
            interaction_options,
            rarity,
            unknown,
        }
    }
}
