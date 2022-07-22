use crate::inventory::{CharacterSpawnItemData, InventoryAvatarItemData, InventoryItemData};
use chrono::{DateTime, Utc};
use silkroad_serde::*;
use silkroad_serde_derive::*;

#[derive(Clone, PartialEq, PartialOrd, Copy, Serialize, Deserialize, ByteSize)]
pub enum PvpCape {
    #[silkroad(value = 0)]
    None,
    #[silkroad(value = 1)]
    Red,
    #[silkroad(value = 2)]
    Gray,
    #[silkroad(value = 3)]
    Blue,
    #[silkroad(value = 4)]
    White,
    #[silkroad(value = 5)]
    Yellow,
}

#[derive(Clone, PartialEq, PartialOrd, Copy, Serialize, ByteSize)]
pub enum AliveState {
    #[silkroad(value = 0)]
    Spawning,
    #[silkroad(value = 1)]
    Alive,
    #[silkroad(value = 2)]
    Dead,
}

#[derive(Clone, PartialEq, PartialOrd, Copy, Serialize, ByteSize)]
pub enum JobType {
    #[silkroad(value = 0)]
    None,
    #[silkroad(value = 1)]
    Trader,
    #[silkroad(value = 2)]
    Thief,
    #[silkroad(value = 3)]
    Hunter,
}

#[derive(Clone, PartialEq, PartialOrd, Copy, Serialize, ByteSize)]
pub enum PlayerKillState {
    #[silkroad(value = 0xFF)]
    None,
    #[silkroad(value = 1)]
    Purple,
    #[silkroad(value = 2)]
    Red,
}

#[derive(Clone, PartialEq, PartialOrd, Copy, Serialize, ByteSize)]
pub enum ActiveScroll {
    #[silkroad(value = 0)]
    None,
    #[silkroad(value = 1)]
    ReturnScroll,
    #[silkroad(value = 2)]
    JobScroll,
}

#[derive(Clone, Serialize, ByteSize)]
pub enum InteractOptions {
    #[silkroad(value = 0)]
    None,
    #[silkroad(value = 2)]
    Talk { options: Vec<u8> },
}

impl InteractOptions {
    pub fn talk(options: Vec<u8>) -> Self {
        InteractOptions::Talk { options }
    }
}

#[derive(Clone, PartialEq, PartialOrd, Copy, Serialize, ByteSize)]
pub enum BodyState {
    #[silkroad(value = 0)]
    None,
    #[silkroad(value = 1)]
    Berserk,
    #[silkroad(value = 2)]
    Untouchable,
    #[silkroad(value = 3)]
    GMInvincible,
    #[silkroad(value = 4)]
    GMInvisible,
    #[silkroad(value = 5)]
    Berserk2,
    #[silkroad(value = 6)]
    Stealth,
    #[silkroad(value = 7)]
    Invisible,
}

#[derive(Clone, PartialEq, PartialOrd, Copy, Serialize, ByteSize)]
pub enum GroupSpawnType {
    #[silkroad(value = 1)]
    Spawn,
    #[silkroad(value = 2)]
    Despawn,
}

#[derive(Clone, PartialEq, PartialOrd, Copy, Serialize, ByteSize)]
pub enum EntityRarity {
    #[silkroad(value = 0)]
    Normal,
    #[silkroad(value = 1)]
    Champion,
    #[silkroad(value = 3)]
    Unique,
    #[silkroad(value = 4)]
    Giant,
    #[silkroad(value = 5)]
    Titan,
    #[silkroad(value = 6)]
    Elite,
    #[silkroad(value = 7)]
    EliteString,
    #[silkroad(value = 8)]
    Unique2,
    #[silkroad(value = 16)]
    NormalParty,
    #[silkroad(value = 17)]
    ChampionParty,
    #[silkroad(value = 19)]
    UniqueParty,
    #[silkroad(value = 20)]
    GiantParty,
    #[silkroad(value = 21)]
    TitanParty,
    #[silkroad(value = 21)]
    EliteParty,
    #[silkroad(value = 24)]
    Unique2Party,
}

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
#[silkroad(size = 0)]
pub enum EntityTypeSpawnData {
    Item,
    Gold {
        amount: u32,
        unique_id: u32,
        position: Position,
        owner: Option<u32>,
        rarity: u8,
    },
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
        unknown3: [u8; 9],
        equipment_cooldown: bool,
        pk_state: PlayerKillState,
        unknown4: u8,
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
        EntityTypeSpawnData::Gold {
            amount,
            unique_id,
            position,
            owner,
            rarity,
        }
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
        unknown3: [u8; 9],
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

#[derive(Clone, PartialEq, PartialOrd, Copy, Serialize, ByteSize)]
pub enum WeatherType {
    #[silkroad(value = 1)]
    Clear,
    #[silkroad(value = 2)]
    Rain,
    #[silkroad(value = 3)]
    Snow,
}

#[derive(Clone, Serialize, ByteSize)]
pub enum GameNotificationContent {
    #[silkroad(value = 0x05)]
    UniqueSpawned { unknown: u8, ref_id: u16 },
    #[silkroad(value = 0x06)]
    UniqueKilled { ref_id: u16 },
}

impl GameNotificationContent {
    pub fn uniquespawned(ref_id: u16) -> Self {
        GameNotificationContent::UniqueSpawned { unknown: 1, ref_id }
    }

    pub fn uniquekilled(ref_id: u16) -> Self {
        GameNotificationContent::UniqueKilled { ref_id }
    }
}

#[derive(Clone, PartialEq, PartialOrd, Copy, Serialize, ByteSize)]
pub enum MovementType {
    #[silkroad(value = 0)]
    Running,
    #[silkroad(value = 1)]
    Walking,
}

#[derive(Clone, PartialEq, PartialOrd, Copy, Serialize, ByteSize)]
pub enum ActionState {
    #[silkroad(value = 0)]
    None,
    #[silkroad(value = 2)]
    Walking,
    #[silkroad(value = 3)]
    Running,
    #[silkroad(value = 4)]
    Sitting,
}

#[derive(Clone, Serialize, Deserialize, ByteSize)]
pub enum MovementTarget {
    #[silkroad(value = 1)]
    TargetLocation { region: u16, x: u16, y: u16, z: u16 },
    #[silkroad(value = 0)]
    Direction { unknown: u8, angle: u16 },
}

impl MovementTarget {
    pub fn targetlocation(region: u16, x: u16, y: u16, z: u16) -> Self {
        MovementTarget::TargetLocation { region, x, y, z }
    }

    pub fn direction(unknown: u8, angle: u16) -> Self {
        MovementTarget::Direction { unknown, angle }
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub enum EntityMovementState {
    #[silkroad(value = 1)]
    Moving {
        movement_type: MovementType,
        region: u16,
        x: u16,
        y: u16,
        z: u16,
    },
    #[silkroad(value = 0)]
    Standing {
        movement_type: MovementType,
        unknown: u8,
        angle: u16,
    },
}

impl EntityMovementState {
    pub fn moving(movement_type: MovementType, region: u16, x: u16, y: u16, z: u16) -> Self {
        EntityMovementState::Moving {
            movement_type,
            region,
            x,
            y,
            z,
        }
    }

    pub fn standing(movement_type: MovementType, unknown: u8, angle: u16) -> Self {
        EntityMovementState::Standing {
            movement_type,
            unknown,
            angle,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, ByteSize)]
pub enum MovementDestination {
    #[silkroad(value = 0)]
    Direction { moving: bool, heading: u16 },
    #[silkroad(value = 1)]
    Location { region: u16, x: u16, y: u16, z: u16 },
}

impl MovementDestination {
    pub fn direction(moving: bool, heading: u16) -> Self {
        MovementDestination::Direction { moving, heading }
    }

    pub fn location(region: u16, x: u16, y: u16, z: u16) -> Self {
        MovementDestination::Location { region, x, y, z }
    }
}

#[derive(Clone, PartialEq, PartialOrd, Copy, Serialize, ByteSize)]
pub enum TargetEntityError {
    #[silkroad(value = 0)]
    InvalidTarget,
}

#[derive(Clone, Serialize, ByteSize)]
#[silkroad(size = 0)]
pub enum TargetEntityData {
    Monster { unknown: u32, interact_data: Option<u8> },
    NPC { talk_options: Option<InteractOptions> },
}

#[derive(Clone, Serialize, ByteSize)]
pub enum TargetEntityResult {
    #[silkroad(value = 2)]
    Failure { error: TargetEntityError },
    #[silkroad(value = 1)]
    Success {
        unique_id: u32,
        health: Option<u32>,
        entity_data: TargetEntityData,
    },
}

impl TargetEntityResult {
    pub fn failure(error: TargetEntityError) -> Self {
        TargetEntityResult::Failure { error }
    }

    pub fn success_monster(unique_id: u32, health: u32) -> Self {
        TargetEntityResult::Success {
            unique_id,
            health: Some(health),
            entity_data: TargetEntityData::Monster {
                unknown: 0,
                interact_data: Some(5),
            },
        }
    }

    pub fn success_npc(unique_id: u32) -> Self {
        TargetEntityResult::Success {
            unique_id,
            health: None,
            entity_data: TargetEntityData::NPC {
                talk_options: Some(InteractOptions::talk(vec![])),
            },
        }
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub struct Position {
    pub region: u16,
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
    pub heading: u16,
}

impl Position {
    pub fn new(region: u16, pos_x: f32, pos_y: f32, pos_z: f32, heading: u16) -> Self {
        Position {
            region,
            pos_x,
            pos_y,
            pos_z,
            heading,
        }
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub struct GuildInformation {
    pub name: String,
    pub id: u32,
    pub member: String,
    pub last_icon_rev: u32,
    pub union_id: u32,
    pub last_union_icon_rev: u32,
    pub is_friendly: u8,
    pub siege_unknown: u8,
}

impl GuildInformation {
    pub fn new(
        name: String,
        id: u32,
        member: String,
        last_icon_rev: u32,
        union_id: u32,
        last_union_icon_rev: u32,
        is_friendly: u8,
    ) -> Self {
        GuildInformation {
            name,
            id,
            member,
            last_icon_rev,
            union_id,
            last_union_icon_rev,
            is_friendly,
            siege_unknown: 0,
        }
    }
}

#[derive(Clone, Serialize, ByteSize)]
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

impl EntityState {
    pub fn new(
        alive: AliveState,
        action_state: ActionState,
        body_state: BodyState,
        walk_speed: f32,
        run_speed: f32,
        berserk_speed: f32,
        active_buffs: Vec<ActiveBuffData>,
    ) -> Self {
        EntityState {
            alive,
            unknown1: 0,
            action_state,
            body_state,
            unknown2: 0,
            walk_speed,
            run_speed,
            berserk_speed,
            active_buffs,
        }
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub struct MasteryData {
    pub id: u32,
    pub level: u8,
}

impl MasteryData {
    pub fn new(id: u32, level: u8) -> Self {
        MasteryData { id, level }
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
pub struct ActiveBuffData {
    pub id: u32,
    pub token: u32,
}

impl ActiveBuffData {
    pub fn new(id: u32, token: u32) -> Self {
        ActiveBuffData { id, token }
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub struct HotkeyData {
    pub slot: u8,
    pub kind: u8,
    pub data: u32,
}

impl HotkeyData {
    pub fn new(slot: u8, kind: u8, data: u32) -> Self {
        HotkeyData { slot, kind, data }
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub struct FriendListGroup {
    pub id: u16,
    pub name: String,
}

impl FriendListGroup {
    pub fn new(id: u16, name: String) -> Self {
        FriendListGroup { id, name }
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub struct FriendListEntry {
    pub char_id: u32,
    pub name: String,
    pub char_model: u32,
    pub group_id: u16,
    pub offline: bool,
}

impl FriendListEntry {
    pub fn new(char_id: u32, name: String, char_model: u32, group_id: u16, offline: bool) -> Self {
        FriendListEntry {
            char_id,
            name,
            char_model,
            group_id,
            offline,
        }
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub struct MovementSource {
    pub region: u16,
    pub x: u16,
    pub y: f32,
    pub z: u16,
}

impl MovementSource {
    pub fn new(region: u16, x: u16, y: f32, z: u16) -> Self {
        MovementSource { region, x, y, z }
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub struct SkillData {
    pub id: u32,
    pub enabled: bool,
}

impl SkillData {
    pub fn new(id: u32, enabled: bool) -> Self {
        SkillData { id, enabled }
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub struct CelestialUpdate {
    pub unique_id: u32,
    pub moon_position: u16,
    pub hour: u8,
    pub minute: u8,
}

impl CelestialUpdate {
    pub fn new(unique_id: u32, moon_position: u16, hour: u8, minute: u8) -> Self {
        CelestialUpdate {
            unique_id,
            moon_position,
            hour,
            minute,
        }
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub struct LunarEventInfo {
    pub unknown_1: u8,
    pub unknown_2: u8,
    pub unknown_3: u32,
    pub unknown_4: u32,
    pub current_achieved: u32,
    pub total: u32,
}

impl LunarEventInfo {
    pub fn new(current_achieved: u32, total: u32) -> Self {
        LunarEventInfo {
            unknown_1: 2,
            unknown_2: 3,
            unknown_3: 1,
            unknown_4: 0x7535,
            current_achieved,
            total,
        }
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub struct CharacterSpawnStart;

#[derive(Clone, Serialize, ByteSize)]
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
    #[silkroad(list_type = "break")]
    pub masteries: Vec<MasteryData>,
    pub unknown_6: u8,
    #[silkroad(list_type = "break")]
    pub skills: Vec<SkillData>,
    #[silkroad(size = 2)]
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

impl CharacterSpawn {
    pub fn new(
        time: u32,
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
        inventory_size: u8,
        inventory_items: Vec<InventoryItemData>,
        avatar_item_size: u8,
        avatar_items: Vec<InventoryAvatarItemData>,
        masteries: Vec<MasteryData>,
        skills: Vec<SkillData>,
        completed_quests: Vec<u32>,
        active_quests: Vec<ActiveQuestData>,
        unique_id: u32,
        position: Position,
        destination_flag: u8,
        angle: u16,
        entity_state: EntityState,
        character_name: String,
        job_name: String,
        job_type: JobType,
        job_level: u8,
        job_exp: u32,
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
            fortress_war_mark,
            service_end,
            user_type,
            server_max_level,
            unknown_2: 0x0107,
            inventory_size,
            inventory_items,
            avatar_item_size,
            avatar_items,
            unknown_3: 0,
            unknown_4: 0xb,
            unknown_5: 0,
            masteries,
            unknown_6: 0,
            skills,
            completed_quests,
            active_quests,
            unknown_8: 0,
            unknown_9: 0,
            unique_id,
            position,
            destination_flag,
            unknown_10: 1,
            unknown_11: 0,
            angle,
            entity_state,
            character_name,
            unknown_14: 0,
            job_name,
            job_type,
            job_level,
            job_exp,
            job_contribution,
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

#[derive(Clone, Serialize, ByteSize)]
pub struct CharacterSpawnEnd;

#[derive(Clone, Serialize, ByteSize)]
pub struct CharacterFinished {
    pub unknown: u16,
}

impl CharacterFinished {
    pub fn new() -> Self {
        CharacterFinished { unknown: 0 }
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub struct EntityDespawn {
    pub entity_id: u32,
}

impl EntityDespawn {
    pub fn new(entity_id: u32) -> Self {
        EntityDespawn { entity_id }
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub struct EntitySpawn {
    pub spawn_data: EntityTypeSpawnData,
    pub unknown_3: u8,
    pub unknown_4: u32,
    pub unknown_5: u8,
}

impl EntitySpawn {
    pub fn new(spawn_data: EntityTypeSpawnData) -> Self {
        EntitySpawn {
            spawn_data,
            unknown_3: 5,
            unknown_4: 0,
            unknown_5: 4,
        }
    }
}

#[derive(Clone, Serialize, ByteSize)]
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

#[derive(Clone, Serialize, ByteSize)]
pub struct GroupEntitySpawnData {
    #[silkroad(list_type = "none")]
    pub content: Vec<GroupSpawnDataContent>,
}

impl GroupEntitySpawnData {
    pub fn new(content: Vec<GroupSpawnDataContent>) -> Self {
        GroupEntitySpawnData { content }
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub struct GroupEntitySpawnEnd;

#[derive(Clone, Serialize, ByteSize)]
pub struct WeatherUpdate {
    pub kind: WeatherType,
    pub speed: u8,
}

impl WeatherUpdate {
    pub fn new(kind: WeatherType, speed: u8) -> Self {
        WeatherUpdate { kind, speed }
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub struct FriendListInfo {
    pub groups: Vec<FriendListGroup>,
    pub friends: Vec<FriendListEntry>,
}

impl FriendListInfo {
    pub fn new(groups: Vec<FriendListGroup>, friends: Vec<FriendListEntry>) -> Self {
        FriendListInfo { groups, friends }
    }
}

#[derive(Clone, Serialize, ByteSize)]
pub struct GameNotification {
    pub result: GameNotificationContent,
}

impl GameNotification {
    pub fn new(result: GameNotificationContent) -> Self {
        GameNotification { result }
    }
}

#[derive(Clone, Deserialize, ByteSize)]
pub struct PlayerMovementRequest {
    pub kind: MovementTarget,
}

#[derive(Clone, Serialize, ByteSize)]
pub struct PlayerMovementResponse {
    pub player_id: u32,
    pub destination: MovementDestination,
    pub source_position: Option<MovementSource>,
}

impl PlayerMovementResponse {
    pub fn new(player_id: u32, destination: MovementDestination, source_position: Option<MovementSource>) -> Self {
        PlayerMovementResponse {
            player_id,
            destination,
            source_position,
        }
    }
}

#[derive(Clone, Deserialize, ByteSize)]
pub struct AddFriend {
    pub name: String,
}

#[derive(Clone, Deserialize, ByteSize)]
pub struct CreateFriendGroup {
    pub name: String,
}

#[derive(Clone, Deserialize, ByteSize)]
pub struct DeleteFriend {
    pub friend_character_id: u32,
}

#[derive(Clone, Serialize, Deserialize, ByteSize)]
pub struct Rotation {
    pub heading: u16,
}

#[derive(Clone, Serialize, ByteSize)]
pub struct EntityUpdateState {
    pub unique_id: u32,
    pub kind: u8,
    pub value: u8,
}

impl EntityUpdateState {
    pub fn new(unique_id: u32, kind: u8, value: u8) -> Self {
        EntityUpdateState { unique_id, kind, value }
    }
}

#[derive(Clone, Deserialize, ByteSize)]
pub struct TargetEntity {
    pub unique_id: u32,
}

#[derive(Clone, Serialize, ByteSize)]
pub struct TargetEntityResponse {
    pub result: TargetEntityResult,
}

impl TargetEntityResponse {
    pub fn new(result: TargetEntityResult) -> Self {
        TargetEntityResponse { result }
    }
}

#[derive(Clone, Deserialize, ByteSize)]
pub struct UnTargetEntity {
    pub unique_id: u32,
}

#[derive(Serialize, ByteSize)]
pub struct UnTargetEntityResponse {
    pub success: bool,
}

impl UnTargetEntityResponse {
    pub fn new(success: bool) -> Self {
        UnTargetEntityResponse { success }
    }
}

// Better name for this...
#[derive(Serialize, ByteSize)]
pub struct EntityBarsUpdate {
    pub unique_id: u32,
    pub mp: u16,
    pub hp: u16,
    #[silkroad(value = 1)]
    pub unknown1: u8,
    #[silkroad(value = 0)]
    pub unknown2: u16,
}

impl EntityBarsUpdate {
    pub fn new(unique_id: u32, mp: u16, hp: u16) -> Self {
        Self {
            unique_id,
            mp,
            hp,
            unknown1: 1,
            unknown2: 0,
        }
    }
}
