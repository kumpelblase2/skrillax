use crate::agent::states::StateTransitionQueue;
use crate::agent::{Agent, MovementState};
use crate::comp::damage::DamageReceiver;
use crate::comp::inventory::PlayerInventory;
use crate::comp::pos::Position;
use crate::comp::sync::Synchronize;
use crate::comp::visibility::Visibility;
use crate::comp::{GameEntity, Health, Mana};
use crate::db::character::CharacterData;
use crate::db::user::ServerUser;
use crate::input::PlayerInput;
use bevy_ecs::prelude::*;
use silkroad_game_base::{Character, Race, SpawningState, Stats};

#[derive(Component, Clone)]
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
            berserk_points: data.berserk_points as u8,
            gold: data.gold as u64,
            beginner_mark: data.beginner_mark,
            gm: data.gm,
            state: SpawningState::Loading,
            masteries: Vec::new(),
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
    pub player: Player,
    pub inventory: PlayerInventory,
    pub game_entity: GameEntity,
    pub agent: Agent,
    pub sync: Synchronize,
    pub pos: Position,
    pub buff: Buffed,
    pub visibility: Visibility,
    pub input: PlayerInput,
    pub state_queue: StateTransitionQueue,
    pub speed: MovementState,
    pub damage_receiver: DamageReceiver,
    pub health: Health,
    pub mana: Mana,
}

impl PlayerBundle {
    pub fn new(
        server_user: ServerUser,
        character: &CharacterData,
        game_entity: GameEntity,
        inventory: PlayerInventory,
        agent: Agent,
        pos: Position,
        visibility: Visibility,
    ) -> Self {
        let player = Player::from_db_data(server_user, &character);
        let max_hp = player.character.max_hp();
        let max_mp = player.character.max_mp();
        Self {
            player,
            game_entity,
            inventory,
            agent,
            pos,
            buff: Buffed {},
            visibility,
            sync: Default::default(),
            input: Default::default(),
            state_queue: Default::default(),
            speed: MovementState::default_player(),
            damage_receiver: DamageReceiver::default(),
            health: Health::new_with_current(character.current_hp as u32, max_hp),
            mana: Mana::new_with_current(character.current_mp as u32, max_mp),
        }
    }
}
