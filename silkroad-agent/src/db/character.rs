use crate::comp::player::Player;
use crate::comp::pos::Position;
use crate::comp::Health;
use chrono::{DateTime, Utc};
use itertools::Itertools;
use sqlx::{Error, PgPool};
use std::borrow::Borrow;
use std::collections::HashMap;

#[derive(sqlx::FromRow, Clone)]
pub struct CharacterData {
    pub id: i32,
    pub user_id: i32,
    pub server_id: i32,
    pub charname: String,
    pub character_type: i32,
    pub scale: i16,
    pub level: i16,
    pub max_level: i16,
    pub exp: i64,
    pub sp: i32,
    pub sp_exp: i32,
    pub strength: i16,
    pub intelligence: i16,
    pub stat_points: i16,
    pub current_hp: i32,
    pub current_mp: i32,
    pub deletion_end: Option<DateTime<Utc>>,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rotation: i16,
    pub region: i16,
    pub berserk_points: i16,
    pub gold: i64,
    pub beginner_mark: bool,
    pub gm: bool,
    pub last_logout: Option<DateTime<Utc>>,
}

impl CharacterData {
    pub async fn fetch_characters<T: Borrow<PgPool>>(
        user: i32,
        shard: u16,
        pool: T,
    ) -> Result<Vec<CharacterData>, Error> {
        sqlx::query_as!(
            CharacterData,
            "SELECT id, user_id, server_id, charname, character_type, scale, level, max_level, exp, sp, sp_exp, strength, intelligence, stat_points, current_hp, current_mp, deletion_end, x, y, z, rotation, region, berserk_points, gold, beginner_mark, gm, last_logout FROM characters WHERE user_id = $1 AND server_id = $2 AND (deletion_end > NOW() OR deletion_end is null) ORDER BY id ASC",
            user,
            shard as i32
        ).fetch_all(pool.borrow()).await
    }

    pub async fn check_name_available<T: Borrow<PgPool>>(name: String, server_id: u16, pool: T) -> (String, bool) {
        let result = sqlx::query!(
            "SELECT COUNT(*) as \"count!\" FROM characters WHERE LOWER(charname) = LOWER($1) and server_id = $2",
            name.clone(),
            server_id as i16
        )
        .fetch_one(pool.borrow())
        .await
        .unwrap();

        (name, result.count == 0)
    }

    pub async fn update_last_played_of<T: Borrow<PgPool>>(character_id: u32, pool: T) {
        sqlx::query!(
            "UPDATE characters SET last_logout = CURRENT_TIMESTAMP WHERE id = $1",
            character_id as i32
        )
        .execute(pool.borrow())
        .await
        .expect("Should be able to update last played.");
    }

    pub(crate) async fn update_character_info<T: Borrow<PgPool>>(
        player: Player,
        health: Health,
        pos: Position,
        pool: T,
    ) {
        let position = pos.location.to_local();
        sqlx::query!(
            "UPDATE characters SET level = $1, exp = $2, strength = $3, intelligence = $4, current_hp = $5, x = $6, y = $7, z = $8, region = $9, sp = $10, sp_exp = $11, stat_points = $12 WHERE id = $13",
            player.character.level as i16,
            player.character.exp as i64,
            player.character.stats.strength() as i16,
            player.character.stats.intelligence() as i16,
            health.current_health as i32,
            position.1.x as f32,
            position.1.y as f32,
            position.1.z as f32,
            position.0.id() as i16,
            player.character.sp as i32,
            player.character.sp_exp as i32,
            player.character.stat_points as i16,
            player.character.id as i32
        )
        .execute(pool.borrow())
        .await
        .expect("Should be able to update stats of player");
    }
}

#[derive(sqlx::FromRow, Clone)]
pub struct CharacterItem {
    pub id: i32,
    pub character_id: i32,
    pub item_obj_id: i32,
    pub upgrade_level: i16,
    pub variance: Option<i64>,
    pub slot: i16,
    pub amount: i16,
}

impl CharacterItem {
    pub async fn fetch_bulk_character_items<T: Borrow<PgPool>>(
        character_ids: &[i32],
        pool: T,
    ) -> Result<HashMap<i32, Vec<CharacterItem>>, Error> {
        let all_items: Vec<CharacterItem> = sqlx::query_as!(
            CharacterItem,
            "SELECT * FROM character_items WHERE character_id in (SELECT * FROM UNNEST($1::INTEGER[]))",
            character_ids
        )
        .fetch_all(pool.borrow())
        .await?;

        let character_item_map = all_items.into_iter().into_group_map_by(|item| item.character_id);
        Ok(character_item_map)
    }
}

#[derive(sqlx::FromRow, Copy, Clone)]
pub struct CharacterMastery {
    pub character_id: i32,
    pub mastery_id: i32,
    pub level: i16,
}

impl CharacterMastery {
    pub async fn fetch_for_characters<T: Borrow<PgPool>>(
        character_ids: &[i32],
        pool: T,
    ) -> Result<Vec<CharacterMastery>, Error> {
        let masteries = sqlx::query_as!(
            CharacterMastery,
            "SELECT mastery_id, character_id, level FROM character_masteries WHERE character_id in (SELECT * FROM UNNEST($1::INTEGER[]))",
            character_ids
        )
                .fetch_all(pool.borrow())
                .await?;
        Ok(masteries)
    }
}
