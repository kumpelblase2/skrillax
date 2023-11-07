use crate::agent::states::StateTransitionQueue;
use crate::agent::{Agent, MovementState};
use crate::comp::damage::DamageReceiver;
use crate::comp::exp::{Experienced, Leveled, SP};
use crate::comp::inventory::PlayerInventory;
use crate::comp::pos::Position;
use crate::comp::visibility::Visibility;
use crate::comp::{GameEntity, Health, Mana};
use crate::db::character::CharacterData;
use crate::db::user::ServerUser;
use crate::game::mind::Mind;
use crate::input::PlayerInput;
use crate::persistence::Persistable;
use crate::sync::Reset;
use bevy_ecs::prelude::*;
use silkroad_game_base::{Character, Race, SpawningState, Stats};

#[derive(Component)]
pub(crate) struct Player {
    pub user: ServerUser,
    pub character: Character,
}

impl Player {
    fn from_db_character(data: &CharacterData) -> Character {
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
            masteries: Vec::new(),
            skills: Vec::new(),
        }
    }

    pub fn from_db_data(user: ServerUser, character: &CharacterData) -> Self {
        let char = Self::from_db_character(character);
        Player { user, character: char }
    }
}

#[derive(Component)]
pub(crate) struct Buffed {
    // pub buffs: Vec<Buff>
}

#[derive(Bundle)]
pub(crate) struct PlayerBundle {
    player: Player,
    inventory: PlayerInventory,
    game_entity: GameEntity,
    agent: Agent,
    pos: Position,
    buff: Buffed,
    visibility: Visibility,
    input: PlayerInput,
    state_queue: StateTransitionQueue,
    speed: MovementState,
    damage_receiver: DamageReceiver,
    health: Health,
    mana: Mana,
    level: Leveled,
    sp: SP,
    exp: Experienced,
    mind: Mind,
    persistence: Persistable,
    stat_points: StatPoints,
}

impl PlayerBundle {
    pub fn new(
        player: Player,
        game_entity: GameEntity,
        inventory: PlayerInventory,
        agent: Agent,
        pos: Position,
        visibility: Visibility,
    ) -> Self {
        let stat_points = StatPoints::new(player.character.stats, player.character.stat_points);
        let level = player.character.level;
        let max_hp = stat_points.stats().max_health(level);
        let max_mana = stat_points.stats().max_mana(level);
        let sp = player.character.sp;
        let sp_exp = player.character.sp_exp;
        let exp = player.character.exp;
        let max_level = player.character.max_level;
        Self {
            player,
            game_entity,
            inventory,
            agent,
            pos,
            buff: Buffed {},
            visibility,
            input: Default::default(),
            state_queue: Default::default(),
            speed: MovementState::default_player(),
            damage_receiver: DamageReceiver::default(),
            health: Health::new(max_hp),
            mana: Mana::with_max(max_mana),
            sp: SP::new(sp),
            level: Leveled::new(level, max_level),
            exp: Experienced::new(exp, sp_exp as u64),
            mind: Mind::default(),
            persistence: Persistable,
            stat_points,
        }
    }
}

#[derive(Component)]
pub(crate) struct StatPoints {
    stats: Stats,
    remaining_points: u16,
    has_gained_points: bool,
    has_spent_points: bool,
}

impl StatPoints {
    pub(crate) fn new(stats: Stats, remaining_points: u16) -> Self {
        StatPoints {
            stats,
            remaining_points,
            has_gained_points: false,
            has_spent_points: false,
        }
    }

    pub(crate) fn stats(&self) -> Stats {
        self.stats
    }

    pub(crate) fn remaining_points(&self) -> u16 {
        self.remaining_points
    }

    pub(crate) fn spend_str(&mut self) {
        self.stats.increase_strength(1);
        self.remaining_points -= 1;
        self.has_spent_points = true;
    }

    pub(crate) fn spend_int(&mut self) {
        self.stats.increase_intelligence(1);
        self.remaining_points -= 1;
        self.has_spent_points = true;
    }

    pub(crate) fn gain_points(&mut self, amount: u16) {
        self.remaining_points = self.remaining_points.saturating_add(amount);
        self.has_gained_points = true;
    }

    pub(crate) fn has_spent_points(&self) -> bool {
        self.has_spent_points
    }

    pub(crate) fn has_gained_points(&self) -> bool {
        self.has_gained_points
    }
}

impl Reset for StatPoints {
    fn reset(&mut self) {
        self.has_gained_points = false;
        self.has_spent_points = true;
    }
}
