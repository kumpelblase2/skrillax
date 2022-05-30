use chrono::{DateTime, Utc};
use itertools::Itertools;
use sqlx::{Error, PgPool};
use std::collections::HashMap;

#[derive(sqlx::FromRow, Clone)]
pub(crate) struct CharacterData {
    pub id: i32,
    pub user_id: i32,
    pub server_id: i32,
    pub charname: String,
    pub character_type: i32,
    pub scale: i16,
    pub levels: i16,
    pub max_level: i16,
    pub exp: i64,
    pub sp: i32,
    pub sp_exp: i32,
    pub strength: i16,
    pub intelligence: i16,
    pub stat_points: i16,
    pub current_hp: i32,
    pub current_mp: i32,
    pub deletion_started: Option<DateTime<Utc>>,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub region: i16,
    pub berserk_points: i16,
    pub gold: i64,
    pub beginner_mark: bool,
    pub gm: bool,
    pub last_logout: Option<DateTime<Utc>>,
}

#[derive(sqlx::FromRow, Clone)]
pub struct CharacterItem {
    pub id: i32,
    pub character_id: i32,
    pub item_obj_id: i32,
    pub upgrade_level: i16,
    pub variance: Option<i64>,
    pub slot: i16,
}

#[derive(sqlx::FromRow)]
pub struct CharacterMastery {}

pub(crate) async fn fetch_characters(pool: &PgPool, user: i32, shard: u16) -> Result<Vec<CharacterData>, Error> {
    sqlx::query_as("SELECT * FROM characters WHERE user_id = $1 AND server_id = $2 ORDER BY id ASC")
        .bind(user)
        .bind(shard as i32)
        .fetch_all(pool)
        .await
}

pub(crate) async fn fetch_character_items(pool: &PgPool, character_id: i32) -> Result<Vec<CharacterItem>, Error> {
    sqlx::query_as("SELECT * FROM character_items WHERE character_id = $1")
        .bind(character_id)
        .fetch_all(pool)
        .await
}

pub(crate) async fn fetch_characters_items(
    pool: &PgPool,
    character_ids: Vec<i32>,
) -> Result<HashMap<i32, Vec<CharacterItem>>, Error> {
    let all_items: Vec<CharacterItem> =
        sqlx::query_as("SELECT * FROM character_items WHERE character_id in (SELECT * FROM UNNEST($1))")
            .bind(character_ids)
            .fetch_all(pool)
            .await?;

    let character_item_map = all_items.into_iter().into_group_map_by(|item| item.character_id);
    Ok(character_item_map)
}
