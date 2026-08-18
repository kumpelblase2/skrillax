mod postgres;

use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Clone)]
pub struct CharacterPersistence {
    pool: PgPool,
}

impl CharacterPersistence {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn character_list(&self, owner: CharacterOwner) -> Result<Vec<ListedCharacter>, CharacterReadError> {
        postgres::load_character_list(&self.pool, owner).await
    }

    pub async fn world_join(&self, owner: CharacterOwner, name: &str) -> Result<WorldJoinCharacter, WorldJoinError> {
        postgres::load_world_join(&self.pool, owner, name).await
    }

    pub async fn is_name_available(&self, server_id: u16, name: &str) -> Result<bool, CharacterWriteError> {
        postgres::is_name_available(&self.pool, server_id, name).await
    }

    pub async fn create(&self, character: NewCharacter) -> Result<(), CharacterWriteError> {
        postgres::create_character(&self.pool, character).await
    }

    pub async fn start_deletion(
        &self,
        owner: CharacterOwner,
        name: &str,
        deletion_minutes: u32,
    ) -> Result<bool, CharacterWriteError> {
        postgres::start_character_deletion(&self.pool, owner, name, deletion_minutes).await
    }

    pub async fn restore(&self, owner: CharacterOwner, name: &str) -> Result<bool, CharacterWriteError> {
        postgres::restore_character(&self.pool, owner, name).await
    }

    pub async fn record_logout(&self, character_id: u32) -> Result<(), CharacterWriteError> {
        postgres::record_character_logout(&self.pool, character_id).await
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CharacterOwner {
    pub user_id: i32,
    pub server_id: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CharacterRace {
    Chinese,
    European,
}

#[derive(Clone, Debug)]
pub struct NewCharacter {
    pub owner: CharacterOwner,
    pub name: String,
    pub race: CharacterRace,
    pub reference_id: u32,
    pub scale: u8,
    pub level: u8,
    pub max_level: u8,
    pub strength: u16,
    pub intelligence: u16,
    pub stat_points: u16,
    pub current_hp: u32,
    pub current_mp: u32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub region: u16,
    pub beginner_mark: bool,
    pub gold: u64,
    pub items: Vec<NewCharacterItem>,
}

#[derive(Clone, Debug)]
pub struct NewCharacterItem {
    pub reference_id: u32,
    pub upgrade_level: u8,
    pub variance: Option<u64>,
    pub slot: u8,
    pub amount: u16,
}

#[derive(Clone, Debug)]
pub struct ListedCharacter {
    pub id: u32,
    pub name: String,
    pub reference_id: u32,
    pub scale: u8,
    pub level: u8,
    pub experience: u64,
    pub skill_points: u32,
    pub strength: u16,
    pub intelligence: u16,
    pub stat_points: u16,
    pub current_hp: u32,
    pub current_mp: u32,
    pub region: u16,
    pub deletion_end: Option<DateTime<Utc>>,
    pub last_logout: Option<DateTime<Utc>>,
    pub equipped_items: Vec<CharacterListItem>,
}

#[derive(Clone, Debug)]
pub struct CharacterListItem {
    pub slot: u8,
    pub reference_id: u32,
    pub upgrade_level: u8,
}

#[derive(Clone, Debug)]
pub struct WorldJoinCharacter {
    pub id: u32,
    pub name: String,
    pub race: CharacterRace,
    pub reference_id: u32,
    pub scale: u8,
    pub level: u8,
    pub max_level: u8,
    pub experience: u64,
    pub skill_points: u32,
    pub skill_experience: u32,
    pub strength: u16,
    pub intelligence: u16,
    pub stat_points: u16,
    pub current_hp: u32,
    pub current_mp: u32,
    pub berserk_points: u8,
    pub gold: u64,
    pub beginner_mark: bool,
    pub game_master: bool,
    pub location: SavedLocation,
    pub items: Vec<CharacterWorldItem>,
    pub masteries: Vec<(u32, u8)>,
    pub skills: Vec<(u32, u8)>,
    pub hotbar: Vec<(u8, u8, u32)>,
}

#[derive(Clone, Copy, Debug)]
pub struct SavedLocation {
    pub region: u16,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub heading: u16,
}

#[derive(Clone, Debug)]
pub struct CharacterWorldItem {
    pub reference_id: u32,
    pub upgrade_level: u8,
    pub variance: Option<u64>,
    pub slot: u8,
    pub amount: u16,
}

#[derive(Debug, thiserror::Error)]
pub enum CharacterReadError {
    #[error("character database query failed")]
    Database(#[from] sqlx::Error),
    #[error("character {character_id} contains invalid {field}: {reason}")]
    InvalidStoredData {
        character_id: i32,
        field: &'static str,
        reason: String,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum CharacterWriteError {
    #[error("character database query failed: {0}")]
    Database(#[from] sqlx::Error),
    #[error("character value for {field} is outside the database range")]
    ValueOutOfRange { field: &'static str },
}

#[derive(Debug, thiserror::Error)]
pub enum WorldJoinError {
    #[error("character was not found")]
    NotFound,
    #[error("character is pending deletion")]
    PendingDeletion,
    #[error(transparent)]
    Read(#[from] CharacterReadError),
}

pub(super) fn invalid_value(
    character_id: i32,
    field: &'static str,
    value: impl std::fmt::Display,
) -> CharacterReadError {
    CharacterReadError::InvalidStoredData {
        character_id,
        field,
        reason: format!("value {value} is outside the supported range"),
    }
}
