use crate::db::character::{
    CharacterData, CharacterHotbar, CharacterItem, CharacterMastery, CharacterSkill, HotbarEntry,
};
use itertools::Itertools;
use sqlx::PgPool;
use std::borrow::Borrow;
use tracing::{debug, instrument};

#[derive(Clone)]
pub struct DbCharacter {
    pub(crate) character_data: CharacterData,
    pub(crate) items: Vec<CharacterItem>,
    pub(crate) masteries: Vec<CharacterMastery>,
    pub(crate) skills: Vec<CharacterSkill>,
    pub(crate) hotbar: Vec<HotbarEntry>,
}

impl DbCharacter {
    #[instrument(name = "load-characters", skip(pool))]
    pub async fn load_characters_sparse<T: Borrow<PgPool>>(user_id: i32, server_id: u16, pool: T) -> Vec<DbCharacter> {
        let characters: Vec<CharacterData> = CharacterData::fetch_characters(user_id, server_id, pool.borrow())
            .await
            .unwrap();
        let character_ids = characters.iter().map(|char| char.id).collect::<Vec<_>>();
        let mut character_items = CharacterItem::fetch_bulk_character_items(&character_ids, pool.borrow())
            .await
            .unwrap();
        let mut character_masteries = CharacterMastery::fetch_for_characters(&character_ids, pool.borrow())
            .await
            .unwrap()
            .into_iter()
            .into_group_map_by(|r| r.character_id);
        let mut character_skills = CharacterSkill::fetch_for_character(&character_ids, pool.borrow())
            .await
            .unwrap()
            .into_iter()
            .into_group_map_by(|s| s.character_id);

        let mut hotbar_entries = CharacterHotbar::fetch_hotbar_entries(&character_ids, pool.borrow())
            .await
            .unwrap()
            .into_iter()
            .into_group_map_by(|e| e.character_id);

        let mut all_characters = Vec::new();

        for character in characters {
            let items = character_items.remove(&character.id).unwrap_or_default();
            let masteries = character_masteries.remove(&character.id).unwrap_or_default();
            let skills = character_skills.remove(&character.id).unwrap_or_default();
            let hotbar = hotbar_entries.remove(&character.id).unwrap_or_default();

            all_characters.push(DbCharacter {
                character_data: character,
                items,
                masteries,
                skills,
                hotbar,
            });
        }

        debug!(
            items = character_items.len(),
            characters = all_characters.len(),
            "Mapped characters."
        );

        all_characters
    }

    pub(crate) async fn start_delete_character<T: Borrow<PgPool>>(
        user_id: i32,
        name: String,
        server_id: u16,
        deletion_duration: u32,
        pool: T,
    ) -> bool {
        let result = sqlx::query!(
        "UPDATE characters SET deletion_end = CURRENT_TIMESTAMP + ($4 * INTERVAL '1 minute') WHERE user_id = $1 AND server_id = $2 AND charname = $3",
        user_id,
        server_id as i16,
        name,
        deletion_duration as i32
    )
                .execute(pool.borrow())
                .await
                .unwrap();
        result.rows_affected() == 1
    }

    pub(crate) async fn restore_character<T: Borrow<PgPool>>(
        user_id: i32,
        name: String,
        server_id: u16,
        pool: T,
    ) -> bool {
        let result =
                sqlx::query!(
                "UPDATE characters SET deletion_end = NULL WHERE user_id = $1 AND server_id = $2 AND charname = $3 AND deletion_end > CURRENT_TIMESTAMP",
                user_id,
                server_id as i16,
                name
            )
                        .execute(pool.borrow())
                        .await
                        .unwrap();
        result.rows_affected() == 1
    }

    pub(crate) async fn create_character<T: Borrow<PgPool>>(character: DbCharacter, pool: T) {
        let result = sqlx::query!(
        "INSERT INTO characters(user_id, server_id, charname, character_type, scale, level, max_level, strength, intelligence, stat_points, current_hp, current_mp, x, y, z, region, beginner_mark) VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17) RETURNING id",
        character.character_data.user_id,
        character.character_data.server_id,
        character.character_data.charname,
        character.character_data.character_type,
        character.character_data.scale,
        character.character_data.level,
        character.character_data.max_level,
        character.character_data.strength,
        character.character_data.intelligence,
        character.character_data.stat_points,
        character.character_data.current_hp,
        character.character_data.current_mp,
        character.character_data.x,
        character.character_data.y,
        character.character_data.z,
        character.character_data.region,
        character.character_data.beginner_mark
    )
                .fetch_one(pool.borrow())
                .await
                .unwrap();

        let id: i32 = result.id;
        for item in character.items.iter() {
            sqlx::query!(
                "INSERT INTO character_items(character_id, item_obj_id, upgrade_level, slot) VALUES($1, $2, $3, $4)",
                id,
                item.item_obj_id,
                item.upgrade_level,
                item.slot
            )
            .execute(pool.borrow())
            .await
            .unwrap();
        }
    }
}
