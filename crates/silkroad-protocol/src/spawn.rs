use crate::community::GuildInformation;
use crate::inventory::{BagContent, CharacterSpawnItemData};
use crate::movement::{EntityMovementState, Position};
use crate::skill::{HotbarItem, MasteryData, SkillData};
use crate::wire_entity::{WireEntityCatalog, WireEntityKind};
use crate::world::{ActiveScroll, EntityState, InteractOptions, JobType, PlayerKillState, PvpCape};
use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use silkroad_definitions::rarity::EntityRarity;
use silkroad_definitions::type_id::{
    ObjectConsumable, ObjectEntity, ObjectItem, ObjectNonPlayer, ObjectNpc, ObjectType,
};
use skrillax_packet::Packet;
use skrillax_serde::*;
use skrillax_stream::registry::PacketRegistryBuilder;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Clone, Eq, PartialEq, Copy, Serialize, ByteSize, Deserialize, Debug)]
pub enum GroupSpawnType {
    #[silkroad(value = 1)]
    Spawn,
    #[silkroad(value = 2)]
    Despawn,
}

#[derive(Copy, Clone, Serialize, ByteSize, Deserialize, Debug)]
pub enum DroppedItemSource {
    #[silkroad(value = 0)]
    None,
    #[silkroad(value = 5)]
    Monster,
    #[silkroad(value = 6)]
    Player,
}

#[derive(Clone, Serialize, ByteSize, Deserialize, Packet, Debug)]
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

#[derive(Clone, Serialize, ByteSize, Deserialize, Debug)]
pub struct CollectionBookTheme {
    pub index: u32,
    pub start: PackedSilkroadTime,
    pub pages: u32,
}

#[derive(Clone, Serialize, ByteSize, Deserialize, Debug)]
pub struct JobInformation {
    pub job_rank: u8,
    pub job_title: u8,
    pub job_name: String,
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

#[derive(Clone, Serialize, ByteSize, Deserialize, Packet, Debug)]
#[packet(opcode = 0x3013)]
#[silkroad(
    after_serialize = "track_character_spawn",
    after_deserialize = "track_character_spawn"
)]
pub struct CharacterSpawn {
    pub time: PackedSilkroadTime,
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
    #[cfg(feature = "v594")]
    pub fortress_war_mark: u8,
    #[cfg(feature = "v657")]
    pub service_end: ServiceEndTime,
    #[cfg(feature = "v594")]
    pub service_end: ExpandedSilkroadTime,
    pub user_type: u8,
    pub server_max_level: u8,
    pub unknown_2_1: u8,
    pub unknown_2_2: u8,
    pub inventory: BagContent,
    pub avatar_items: BagContent,
    pub job_bag: BagContent,
    pub specialty_bag: BagContent,
    #[cfg(feature = "v657")]
    pub unknown_5: u8,
    #[cfg(feature = "v657")]
    pub unknown_6: u8,
    #[cfg(feature = "v594")]
    pub unknown_5: u16,
    #[silkroad(list_type = "break")]
    pub masteries: Vec<MasteryData>,
    #[cfg(feature = "v594")]
    pub unknown_6: u8,
    #[silkroad(list_type = "break")]
    pub skills: Vec<SkillData>,
    #[silkroad(size = 2)]
    pub completed_quests: Vec<u32>,
    pub active_quests: Vec<ActiveQuestData>,
    pub unknown_8: u8,
    #[cfg_attr(feature = "v657", silkroad(size = 3))]
    #[cfg_attr(feature = "v594", silkroad(size = 4))]
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
    pub hotkeys: Vec<HotbarItem>,
    pub unknown_20: u8,
    pub auto_hp: u16,
    pub auto_mp: u16,
    pub auto_pill: u16,
    pub potion_delay: u8,
    pub blocked_players: Vec<String>,
    pub unknown_21: u32,
}

fn track_character_spawn(packet: &CharacterSpawn, ctx: &SerdeContext) -> Result<(), SerializationError> {
    WireEntityCatalog::for_context(ctx).record_spawn(packet.unique_id, packet.ref_id, wire_entity_kind(packet.ref_id));
    Ok(())
}

impl CharacterSpawn {
    #[allow(clippy::useless_conversion)]
    pub fn new(
        time: PackedSilkroadTime,
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
        job_reward: u32,
        pvp_state: u8,
        transport_flag: bool,
        in_combat: u8,
        pvp_flag: u8,
        jid: u32,
        gm: bool,
        hotkeys: Vec<HotbarItem>,
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
            #[cfg(feature = "v594")]
            fortress_war_mark,
            service_end: service_end.try_into().expect("Should be able to expand date time"),
            user_type,
            server_max_level,
            unknown_2_1: 0x07,
            unknown_2_2: 0x01,
            inventory,
            avatar_items,
            specialty_bag: BagContent::empty(),
            job_bag: BagContent::new(0xb, Vec::new()),
            unknown_5: 0,
            unknown_6: 1,
            masteries,
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

#[derive(Clone, Serialize, ByteSize, Deserialize, Packet, Debug)]
#[packet(opcode = 0x34A6)]
pub struct CharacterSpawnEnd;

#[derive(Clone, Serialize, ByteSize, Deserialize, Packet, Debug)]
#[packet(opcode = 0x3016)]
#[silkroad(after_serialize = "track_entity_despawn", after_deserialize = "track_entity_despawn")]
pub struct EntityDespawn {
    pub entity_id: u32,
}

fn track_entity_despawn(packet: &EntityDespawn, ctx: &SerdeContext) -> Result<(), SerializationError> {
    WireEntityCatalog::for_context(ctx).retire(packet.entity_id);
    Ok(())
}

impl EntityDespawn {
    pub fn new(entity_id: u32) -> Self {
        EntityDespawn { entity_id }
    }
}

#[derive(Clone, Serialize, ByteSize, Deserialize, Packet, Debug)]
#[packet(opcode = 0x3015)]
#[silkroad(after_serialize = "track_entity_spawn", after_deserialize = "track_entity_spawn")]
pub struct EntitySpawn {
    pub data: EntityTypeSpawnData,
}

fn track_entity_spawn(packet: &EntitySpawn, ctx: &SerdeContext) -> Result<(), SerializationError> {
    let (unique_id, ref_id) = packet.data.wire_identity();
    WireEntityCatalog::for_context(ctx).record_spawn(unique_id, ref_id, wire_entity_kind(ref_id));
    Ok(())
}

impl EntitySpawn {
    pub fn new(spawn_data: EntityTypeSpawnData) -> Self {
        EntitySpawn { data: spawn_data }
    }
}

#[derive(Clone, Serialize, ByteSize, Deserialize, Packet, Debug)]
#[packet(opcode = 0x3017)]
#[silkroad(
    after_serialize = "spawn_start_serialize",
    after_deserialize = "spawn_start_deserialize"
)]
pub struct GroupEntitySpawnStart {
    pub kind: GroupSpawnType,
    pub amount: u16,
    pub unknown_1: u32,
    pub unknown_2: u16,
}

fn spawn_start_serialize(spawn: &GroupEntitySpawnStart, ctx: &SerdeContext) -> Result<(), SerializationError> {
    ctx.set(GroupEntitySpawnCount(spawn.amount as usize));
    ctx.set(match spawn.kind {
        GroupSpawnType::Spawn => GroupEntityType::Spawn,
        GroupSpawnType::Despawn => GroupEntityType::Despawn,
    });
    Ok(())
}

fn spawn_start_deserialize(spawn: &GroupEntitySpawnStart, ctx: &SerdeContext) -> Result<(), SerializationError> {
    ctx.set(GroupEntitySpawnCount(spawn.amount as usize));
    ctx.set(match spawn.kind {
        GroupSpawnType::Spawn => GroupEntityType::Spawn,
        GroupSpawnType::Despawn => GroupEntityType::Despawn,
    });
    Ok(())
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

#[derive(Copy, Clone, Default)]
pub struct GroupEntitySpawnCount(pub usize);

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum GroupEntityType {
    Spawn,
    Despawn,
}

impl From<GroupSpawnType> for GroupEntityType {
    fn from(value: GroupSpawnType) -> Self {
        match value {
            GroupSpawnType::Spawn => GroupEntityType::Spawn,
            GroupSpawnType::Despawn => GroupEntityType::Despawn,
        }
    }
}

#[derive(Clone, Serialize, ByteSize, Deserialize, Packet, Debug)]
#[packet(opcode = 0x3019)]
#[silkroad(
    after_serialize = "track_group_entity_data",
    after_deserialize = "track_group_entity_data"
)]
pub struct GroupEntitySpawnData {
    #[silkroad(
        list_type = "calculated",
        calculate = "ctx.get::<GroupEntitySpawnCount>().unwrap_or_default().0"
    )]
    pub content: Vec<GroupSpawnDataContent>,
}

fn track_group_entity_data(packet: &GroupEntitySpawnData, ctx: &SerdeContext) -> Result<(), SerializationError> {
    let catalog = WireEntityCatalog::for_context(ctx);
    for content in &packet.content {
        match content {
            GroupSpawnDataContent::Spawn { data } => {
                let (unique_id, ref_id) = data.wire_identity();
                catalog.record_spawn(unique_id, ref_id, wire_entity_kind(ref_id));
            },
            GroupSpawnDataContent::Despawn { id } => catalog.retire(*id),
        }
    }
    Ok(())
}

impl GroupEntitySpawnData {
    pub fn new(content: Vec<GroupSpawnDataContent>) -> Self {
        GroupEntitySpawnData { content }
    }
}

#[derive(Clone, Serialize, ByteSize, Deserialize, Packet, Debug)]
#[packet(opcode = 0x3018)]
#[silkroad(after_serialize = "spawn_end_serialize", after_deserialize = "spawn_end_deserialize")]
pub struct GroupEntitySpawnEnd;

fn spawn_end_serialize(_packet: &GroupEntitySpawnEnd, ctx: &SerdeContext) -> Result<(), SerializationError> {
    ctx.unset::<GroupEntitySpawnCount>();
    ctx.unset::<GroupEntityType>();
    Ok(())
}

fn spawn_end_deserialize(_packet: &GroupEntitySpawnEnd, ctx: &SerdeContext) -> Result<(), SerializationError> {
    ctx.unset::<GroupEntitySpawnCount>();
    ctx.unset::<GroupEntityType>();
    Ok(())
}

#[derive(Clone, Serialize, ByteSize, Deserialize, Debug)]
#[silkroad(size = 0)]
pub enum GroupSpawnDataContent {
    #[silkroad(when = "ctx.get::<GroupEntityType>().filter(|t| *t == GroupEntityType::Despawn).is_some()")]
    Despawn { id: u32 },
    #[silkroad(when = "ctx.get::<GroupEntityType>().filter(|t| *t == GroupEntityType::Spawn).is_some()")]
    Spawn { data: EntityTypeSpawnData },
}

impl GroupSpawnDataContent {
    pub fn despawn(id: u32) -> Self {
        GroupSpawnDataContent::Despawn { id }
    }

    pub fn spawn(spawn_data: EntityTypeSpawnData) -> Self {
        GroupSpawnDataContent::Spawn { data: spawn_data }
    }
}

#[derive(Clone, Serialize, ByteSize, Deserialize, Debug)]
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

#[derive(Clone, Serialize, ByteSize, Deserialize, Debug)]
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

static REF_ID_TO_OBJ: OnceLock<HashMap<u32, ObjectType>> = OnceLock::new();

pub fn register_ref_id(map: HashMap<u32, ObjectType>) {
    let _ = REF_ID_TO_OBJ.set(map);
}

fn wire_entity_kind(ref_id: u32) -> WireEntityKind {
    match REF_ID_TO_OBJ.get().and_then(|objects| objects.get(&ref_id)) {
        Some(ObjectType::Entity(ObjectEntity::NonPlayer(ObjectNonPlayer::Monster(_)))) => WireEntityKind::Monster,
        Some(ObjectType::Entity(ObjectEntity::NonPlayer(ObjectNonPlayer::NPC(_)))) => WireEntityKind::Npc,
        _ => WireEntityKind::Other,
    }
}

fn is_item_ref(id: u32) -> bool {
    let found_ref = REF_ID_TO_OBJ.get().expect("Should have been set").get(&id);
    found_ref
        .map(|obj| matches!(obj, ObjectType::Item(ObjectItem::Consumable(_))))
        .unwrap_or(false)
}

fn is_gold_ref(id: u32) -> bool {
    let found_ref = REF_ID_TO_OBJ.get().expect("Should have been set").get(&id);
    found_ref
        .map(|obj| {
            matches!(
                obj,
                ObjectType::Item(ObjectItem::Consumable(ObjectConsumable::Currency(_)))
            )
        })
        .unwrap_or(false)
}

fn is_equipment_ref(id: u32) -> bool {
    let found_ref = REF_ID_TO_OBJ.get().expect("Should have been set").get(&id);
    found_ref
        .map(|obj| matches!(obj, ObjectType::Item(ObjectItem::Equippable(_))))
        .unwrap_or(false)
}

fn is_character_ref(id: u32) -> bool {
    let found_ref = REF_ID_TO_OBJ.get().expect("Should have been set").get(&id);
    found_ref
        .map(|obj| matches!(obj, ObjectType::Entity(ObjectEntity::Player)))
        .unwrap_or(false)
}

fn is_monster_ref(id: u32) -> bool {
    let found_ref = REF_ID_TO_OBJ.get().expect("Should have been set").get(&id);
    found_ref
        .map(|obj| {
            matches!(
                obj,
                ObjectType::Entity(ObjectEntity::NonPlayer(ObjectNonPlayer::Monster(_)))
            )
        })
        .unwrap_or(false)
}

fn is_npc_ref(id: u32) -> bool {
    let found_ref = REF_ID_TO_OBJ.get().expect("Should have been set").get(&id);
    found_ref
        .map(|obj| {
            matches!(
                obj,
                ObjectType::Entity(ObjectEntity::NonPlayer(ObjectNonPlayer::NPC(ObjectNpc::Standard)))
            )
        })
        .unwrap_or(false)
}

fn is_portal_ref(id: u32) -> bool {
    let found_ref = REF_ID_TO_OBJ.get().expect("Should have been set").get(&id);
    found_ref
        .map(|obj| {
            matches!(
                obj,
                ObjectType::Entity(ObjectEntity::NonPlayer(ObjectNonPlayer::NPC(ObjectNpc::GateStructure)))
            )
        })
        .unwrap_or(false)
}

#[derive(Clone, Serialize, ByteSize, Deserialize, Debug)]
#[silkroad(size = 4)]
pub enum EntityTypeSpawnData {
    #[silkroad(when = "is_gold_ref(tag)")]
    GoldItem {
        #[silkroad(tag)]
        ref_id: u32,
        amount: u32,
        unique_id: u32,
        position: Position,
        owner: Option<u32>,
        rarity: u8,
    },
    #[silkroad(when = "is_equipment_ref(tag)")]
    EquipmentItem {
        #[silkroad(tag)]
        ref_id: u32,
        upgrade: u8,
        unique_id: u32,
        position: Position,
        owner: Option<u32>,
        rarity: u8,
        source: DroppedItemSource,
        source_id: u32,
    },
    #[silkroad(when = "is_item_ref(tag)")]
    ConsumableItem {
        #[silkroad(tag)]
        ref_id: u32,
        unique_id: u32,
        position: Position,
        owner: Option<u32>,
        rarity: u8,
        source: DroppedItemSource,
        source_id: u32,
    },
    #[silkroad(when = "is_character_ref(tag)")]
    Character {
        #[silkroad(tag)]
        ref_id: u32,
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
        #[cfg(feature = "v594")]
        unknown3: [u8; 9],
        #[cfg(feature = "v657")]
        unknown3: [u8; 11],
        equipment_cooldown: bool,
        pk_state: PlayerKillState,
        unknown4: u8,
        #[cfg(feature = "v657")]
        unknown5: u8,
    },
    #[silkroad(when = "is_npc_ref(tag)")]
    NPC {
        #[silkroad(tag)]
        ref_id: u32,
        unique_id: u32,
        position: Position,
        movement: EntityMovementState,
        entity_state: EntityState,
        interaction_options: InteractOptions,
    },
    #[silkroad(when = "is_monster_ref(tag)")]
    Monster {
        #[silkroad(tag)]
        ref_id: u32,
        unique_id: u32,
        position: Position,
        movement: EntityMovementState,
        entity_state: EntityState,
        interaction_options: InteractOptions,
        rarity: EntityRarity,
        unknown: u32,
    },
    #[silkroad(when = "is_portal_ref(tag)")]
    Portal {
        #[silkroad(tag)]
        ref_id: u32,
        unique_id: u32,
        position: Position,
        unknown_1: u8,
        unknown_2: u8,
        unknown_3: u32,
        unknown_4: u8,
        unknown_5: u8,
        data: PortalData,
    },
}

#[derive(Clone, Copy, Serialize, ByteSize, Deserialize, Debug)]
pub enum PortalData {
    #[silkroad(value = 1)]
    General { unknown_1: u32, unknown_2: u32 },
}

impl EntityTypeSpawnData {
    fn wire_identity(&self) -> (u32, u32) {
        match self {
            EntityTypeSpawnData::GoldItem { ref_id, unique_id, .. }
            | EntityTypeSpawnData::EquipmentItem { ref_id, unique_id, .. }
            | EntityTypeSpawnData::ConsumableItem { ref_id, unique_id, .. }
            | EntityTypeSpawnData::Character { ref_id, unique_id, .. }
            | EntityTypeSpawnData::NPC { ref_id, unique_id, .. }
            | EntityTypeSpawnData::Monster { ref_id, unique_id, .. }
            | EntityTypeSpawnData::Portal { ref_id, unique_id, .. } => (*unique_id, *ref_id),
        }
    }

    pub fn gold(ref_id: u32, amount: u32, unique_id: u32, position: Position, owner: Option<u32>, rarity: u8) -> Self {
        EntityTypeSpawnData::GoldItem {
            ref_id,
            amount,
            unique_id,
            position,
            owner,
            rarity,
        }
    }

    pub fn character(
        ref_id: u32,
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
        equipment_cooldown: bool,
        pk_state: PlayerKillState,
    ) -> Self {
        EntityTypeSpawnData::Character {
            ref_id,
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
            #[cfg(feature = "v594")]
            unknown3: [0; 9],
            #[cfg(feature = "v657")]
            unknown3: [0; 11],
            equipment_cooldown,
            pk_state,
            unknown4: 0xFF,
            #[cfg(feature = "v657")]
            unknown5: 0x01,
        }
    }

    pub fn monster(
        ref_id: u32,
        unique_id: u32,
        position: Position,
        movement: EntityMovementState,
        entity_state: EntityState,
        interaction_options: InteractOptions,
        rarity: EntityRarity,
        unknown: u32,
    ) -> Self {
        EntityTypeSpawnData::Monster {
            ref_id,
            unique_id,
            position,
            movement,
            entity_state,
            interaction_options,
            rarity,
            unknown,
        }
    }

    pub fn npc(
        ref_id: u32,
        unique_id: u32,
        position: Position,
        movement: EntityMovementState,
        entity_state: EntityState,
        interaction_options: InteractOptions,
    ) -> Self {
        EntityTypeSpawnData::NPC {
            ref_id,
            unique_id,
            position,
            movement,
            entity_state,
            interaction_options,
        }
    }

    pub fn consumable_item(
        ref_id: u32,
        unique_id: u32,
        position: Position,
        owner: Option<u32>,
        rarity: u8,
        source: DroppedItemSource,
        source_id: u32,
    ) -> Self {
        EntityTypeSpawnData::ConsumableItem {
            ref_id,
            unique_id,
            position,
            owner,
            rarity,
            source,
            source_id,
        }
    }

    pub fn equipable_item(
        ref_id: u32,
        upgrade: u8,
        unique_id: u32,
        position: Position,
        owner: Option<u32>,
        rarity: u8,
        source: DroppedItemSource,
        source_id: u32,
    ) -> Self {
        EntityTypeSpawnData::EquipmentItem {
            ref_id,
            upgrade,
            unique_id,
            position,
            owner,
            rarity,
            source,
            source_id,
        }
    }
}

pub trait SpawnPacketRegistryExt {
    fn register_spawn_packets(self) -> Self;
}

impl SpawnPacketRegistryExt for PacketRegistryBuilder {
    fn register_spawn_packets(self) -> Self {
        self.register::<CharacterSpawnStart>()
            .register::<CharacterSpawn>()
            .register::<CharacterSpawnEnd>()
            .register::<EntityDespawn>()
            .register::<EntitySpawn>()
            .register::<GroupEntitySpawnStart>()
            .register::<GroupEntitySpawnData>()
            .register::<GroupEntitySpawnEnd>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire_entity::{WireEntityRecord, WireEntityStatus};

    fn item_spawn(unique_id: u32) -> EntityTypeSpawnData {
        EntityTypeSpawnData::gold(9000, 1, unique_id, Position::new(1, 2.0, 3.0, 4.0, 5), None, 0)
    }

    #[test]
    fn grouped_spawns_are_tracked_and_despawns_become_tombstones() {
        let context = SerdeContext::default();
        let catalog = WireEntityCatalog::install(&context);
        let spawn = GroupEntitySpawnData::new(vec![GroupSpawnDataContent::spawn(item_spawn(77))]);

        track_group_entity_data(&spawn, &context).unwrap();
        assert_eq!(
            catalog.lookup(77),
            Some(WireEntityRecord {
                ref_id: 9000,
                kind: WireEntityKind::Other,
                status: WireEntityStatus::Active,
            })
        );

        let despawn = GroupEntitySpawnData::new(vec![GroupSpawnDataContent::despawn(77)]);
        track_group_entity_data(&despawn, &context).unwrap();
        assert_eq!(catalog.lookup(77).unwrap().status, WireEntityStatus::Retired);
    }

    #[test]
    fn a_visibility_respawn_reactivates_a_tombstone() {
        let context = SerdeContext::default();
        let catalog = WireEntityCatalog::install(&context);
        let despawn = EntityDespawn::new(77);
        let spawn = EntitySpawn::new(item_spawn(77));

        track_entity_spawn(&spawn, &context).unwrap();
        track_entity_despawn(&despawn, &context).unwrap();
        assert_eq!(catalog.lookup(77).unwrap().status, WireEntityStatus::Retired);

        track_entity_spawn(&spawn, &context).unwrap();
        assert_eq!(catalog.lookup(77).unwrap().status, WireEntityStatus::Active);
    }

    #[test]
    fn entity_despawn_hooks_retire_on_encode_and_decode() {
        let encode_context = SerdeContext::default();
        let encode_catalog = WireEntityCatalog::install(&encode_context);
        encode_catalog.record_spawn(77, 9000, WireEntityKind::Other);
        let mut bytes = bytes::BytesMut::new();

        EntityDespawn::new(77).write_to(&mut bytes, &encode_context).unwrap();

        assert_eq!(bytes.as_ref(), &[77, 0, 0, 0]);
        assert_eq!(encode_catalog.lookup(77).unwrap().status, WireEntityStatus::Retired);

        let decode_context = SerdeContext::default();
        let decode_catalog = WireEntityCatalog::install(&decode_context);
        decode_catalog.record_spawn(77, 9000, WireEntityKind::Other);
        EntityDespawn::read_from(&mut std::io::Cursor::new(bytes), &decode_context).unwrap();

        assert_eq!(decode_catalog.lookup(77).unwrap().status, WireEntityStatus::Retired);
    }
}
