use crate::comp::net::InputBundle;
use crate::comp::pos::{GlobalPosition, Heading, Position};
use crate::comp::stats::Stats;
use crate::comp::sync::Synchronize;
use crate::comp::visibility::Visibility;
use crate::comp::GameEntity;
use crate::db::character::CharacterItem;
use crate::db::user::ServerUser;
use bevy_ecs::prelude::*;
use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::time::Instant;

pub(crate) struct Item {
    pub ref_id: i32,
    pub variance: Option<u64>,
    pub upgrade_level: u8,
    pub type_data: ItemTypeData,
}

pub(crate) enum ItemTypeData {
    Default,
    Equipment,
    COS,
    Consumable(),
}

pub(crate) struct Inventory {
    size: usize,
    items: HashMap<u8, Item>,
}

impl Inventory {
    pub fn size(&self) -> usize {
        self.size
    }

    pub fn from(items: &[CharacterItem], size: usize) -> Inventory {
        let mut my_items = HashMap::new();

        for item in items {
            my_items.insert(
                item.slot as u8,
                Item {
                    ref_id: item.item_obj_id,
                    variance: item.variance.map(|v| v as u64),
                    upgrade_level: item.upgrade_level as u8,
                    type_data: ItemTypeData::Default,
                },
            );
        }

        Inventory { items: my_items, size }
    }

    pub fn items(&self) -> Iter<u8, Item> {
        self.items.iter()
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Inventory {
            items: HashMap::new(),
            size: 45,
        }
    }
}

#[derive(Eq, PartialEq)]
pub enum SpawningState {
    Loading,
    Spawning,
    Finished,
}

pub(crate) enum Race {
    European,
    Chinese,
}

pub enum MovementState {
    Sitting,
    Standing,
    Moving,
    Walking,
}

pub(crate) struct Character {
    pub id: u32,
    pub name: String,
    pub race: Race,
    pub scale: u8,
    pub level: u8,
    pub max_level: u8,
    pub exp: u64,
    pub sp: u32,
    pub sp_exp: u32,
    pub stats: Stats,
    pub stat_points: u16,
    pub current_hp: u32,
    pub current_mp: u32,
    pub berserk_points: u8,
    pub gold: u64,
    pub beginner_mark: bool,
    pub gm: bool,
    pub state: SpawningState,
}

impl Character {
    pub fn max_hp(&self) -> u32 {
        self.stats.max_health(self.level)
    }

    pub fn max_mp(&self) -> u32 {
        self.stats.max_mana(self.level)
    }

    pub fn from_db_character(data: &crate::db::character::CharacterData) -> Character {
        Character {
            id: data.id as u32,
            name: data.charname.clone(),
            race: Race::Chinese,
            scale: data.scale as u8,
            level: data.level as u8,
            max_level: data.max_level as u8,
            exp: data.exp as u64,
            sp: data.sp as u32,
            sp_exp: data.sp_exp as u32,
            stats: Stats::new_preallocated(data.strength as u16, data.intelligence as u16),
            stat_points: data.stat_points as u16,
            current_hp: data.current_hp as u32,
            current_mp: data.current_mp as u32,
            berserk_points: data.berserk_points as u8,
            gold: data.gold as u64,
            beginner_mark: data.beginner_mark,
            gm: data.gm,
            state: SpawningState::Loading,
        }
    }
}

#[derive(Component)]
pub(crate) struct Player {
    pub user: ServerUser,
    pub character: Character,
    pub inventory: Inventory,
    pub logout: Option<Instant>,
    pub target: Option<Entity>,
}

pub(crate) enum MovementTarget {
    Location(GlobalPosition),
    Direction(Heading),
    Turn(Heading),
}

#[derive(Component)]
pub(crate) struct Agent {
    pub movement_speed: f32,
    pub movement_state: MovementState,
    pub movement_target: Option<MovementTarget>,
}

impl Agent {
    pub fn new(movement_speed: f32) -> Self {
        Self {
            movement_speed,
            movement_state: MovementState::Standing,
            movement_target: None,
        }
    }
}

#[derive(Component)]
pub(crate) struct Buffed {
    // pub buffs: Vec<Buff>
}

#[derive(Bundle)]
pub(crate) struct PlayerBundle {
    player: Player,
    game_entity: GameEntity,
    agent: Agent,
    sync: Synchronize,
    pos: Position,
    buff: Buffed,
    visibility: Visibility,
    #[bundle]
    inputs: InputBundle,
}

impl PlayerBundle {
    pub fn new(player: Player, game_entity: GameEntity, agent: Agent, pos: Position, visibility: Visibility) -> Self {
        Self {
            player,
            game_entity,
            agent,
            sync: Synchronize::default(),
            pos,
            buff: Buffed {},
            visibility,
            inputs: InputBundle::default(),
        }
    }
}
