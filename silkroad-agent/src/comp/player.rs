use crate::comp::net::InputBundle;
use crate::comp::pos::{GlobalPosition, Heading, Position};
use crate::comp::stats::Stats;
use crate::comp::sync::Synchronize;
use crate::comp::visibility::Visibility;
use crate::comp::GameEntity;
use crate::db::character::CharacterItem;
use crate::db::user::ServerUser;
use crate::world::WorldData;
use bevy_core::Timer;
use bevy_ecs::prelude::*;
use silkroad_data::itemdata::RefItemData;
use silkroad_data::skilldata::RefSkillData;
use silkroad_data::DataEntry;
use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::time::Instant;

const WEAPON_SLOT: u8 = 6;

pub(crate) struct Item {
    pub reference: &'static RefItemData,
    pub variance: Option<u64>,
    pub upgrade_level: u8,
    pub type_data: ItemTypeData,
    pub amount: u16,
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

pub enum MoveError {
    ItemDoesNotExist,
    Impossible,
}

impl Inventory {
    pub fn size(&self) -> usize {
        self.size
    }

    pub fn from(items: &[CharacterItem], size: usize) -> Inventory {
        let item_map = WorldData::items();
        let mut my_items = HashMap::new();

        for item in items {
            let item_def = item_map.find_id(item.item_obj_id as u32).unwrap();

            my_items.insert(
                item.slot as u8,
                Item {
                    reference: item_def,
                    variance: item.variance.map(|v| v as u64),
                    upgrade_level: item.upgrade_level as u8,
                    type_data: ItemTypeData::Default,
                    amount: item.amount as u16,
                },
            );
        }

        Inventory { items: my_items, size }
    }

    pub fn get_item_at(&self, slot: u8) -> Option<&Item> {
        self.items.get(&slot)
    }

    pub fn equipment_items(&self) -> impl Iterator<Item = (&u8, &Item)> {
        self.items.iter().filter(|(index, _)| Self::is_equipment_slot(**index))
    }

    pub fn items(&self) -> Iter<u8, Item> {
        self.items.iter()
    }

    pub fn weapon(&self) -> Option<&Item> {
        self.items.get(&WEAPON_SLOT)
    }

    pub(crate) fn move_item(&mut self, source: u8, target: u8, amount: u16) -> Result<u16, MoveError> {
        if let Some(mut source_item) = self.items.remove(&source) {
            if let Some(mut target_item) = self.items.remove(&target) {
                if source_item.reference.ref_id() == target_item.reference.ref_id()
                    && source_item.reference.max_stack_size > 1
                {
                    let available_on_target_stack = target_item.reference.max_stack_size - target_item.amount;
                    if available_on_target_stack == 0 {
                        return Ok(0);
                    }

                    if available_on_target_stack >= amount {
                        target_item.amount += amount;
                        self.items.insert(target, target_item);
                    } else {
                        target_item.amount += available_on_target_stack;
                        source_item.amount -= available_on_target_stack;

                        self.items.insert(source, source_item);
                        self.items.insert(target, target_item);
                        return Ok(available_on_target_stack);
                    }
                } else {
                    self.items.insert(target, source_item);
                    self.items.insert(source, target_item);
                }
            } else {
                self.items.insert(target, source_item);
            }
        } else {
            return Err(MoveError::ItemDoesNotExist);
        }

        Ok(amount)
    }

    pub fn is_equipment_slot(slot: u8) -> bool {
        slot <= 0xCu8
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

#[derive(Copy, Clone, Eq, PartialEq)]
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
}

pub(crate) enum MovementTarget {
    Location(GlobalPosition),
    Direction(Heading),
    Turn(Heading),
}

pub(crate) enum SkillState {
    Before,
    Prepare(Timer),
    Cast(Timer),
    Default(Timer),
    Cooldown(Timer),
}

pub(crate) enum AgentAction {
    // what about stunned, knockdown, etc. that prevent action?
    Movement(MovementTarget),
    Skill {
        reference: &'static RefSkillData,
        target: Option<Entity>,
        state: SkillState,
    },
    Attack {
        target: Entity,
        range: f32,
        reference: &'static RefSkillData,
        current_destination: GlobalPosition,
        refresh_timer: Timer,
    },
}

impl AgentAction {
    pub(crate) fn can_cancel(&self) -> bool {
        matches!(self, AgentAction::Movement(_))
    }
}

#[derive(Component)]
pub(crate) struct Agent {
    pub movement_speed: f32,
    pub movement_state: MovementState,
    pub current_action: Option<AgentAction>,
    pub next_action: Option<AgentAction>,
    pub target: Option<Entity>,
}

impl Agent {
    pub fn new(movement_speed: f32) -> Self {
        Self {
            movement_speed,
            movement_state: MovementState::Standing,
            current_action: None,
            next_action: None,
            target: None,
        }
    }

    pub fn move_to<T: Into<GlobalPosition>>(&mut self, position: T) {
        self.next_action = Some(AgentAction::Movement(MovementTarget::Location(position.into())));
    }

    pub fn move_in_direction(&mut self, direction: Heading) {
        self.next_action = Some(AgentAction::Movement(MovementTarget::Direction(direction)));
    }

    pub fn turn(&mut self, heading: Heading) {
        if self.current_action.is_none() {
            self.next_action = Some(AgentAction::Movement(MovementTarget::Turn(heading)));
        }
    }

    pub fn is_in_action(&self) -> bool {
        self.current_action.is_some()
    }

    pub fn finish_current_action(&mut self) {
        self.current_action = None;
        self.movement_state = MovementState::Standing;
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
