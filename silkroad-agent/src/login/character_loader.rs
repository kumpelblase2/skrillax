use crate::db::character::{fetch_characters, fetch_characters_items, CharacterData, CharacterItem};
use sqlx::{PgPool, Row};
use tracing::trace_span;

#[derive(Clone)]
pub struct Character {
    pub(crate) character_data: CharacterData,
    pub(crate) items: Vec<CharacterItem>,
}

pub(crate) async fn load_characters_sparse(pool: PgPool, user_id: i32, server_id: u16) -> Vec<Character> {
    let span = trace_span!("Load Characters", id = user_id);
    let _guard = span.enter();
    let characters: Vec<CharacterData> = fetch_characters(&pool, user_id, server_id).await.unwrap();

    let character_ids = characters.iter().map(|char| char.id).collect();
    let mut character_items = fetch_characters_items(&pool, character_ids).await.unwrap();

    let mut all_characters = Vec::new();

    for character in characters {
        let items = character_items.remove(&character.id).unwrap_or_default();

        all_characters.push(Character {
            character_data: character,
            items,
        });
    }

    all_characters
}

pub(crate) async fn check_name_available(pool: PgPool, name: String, server_id: u16) -> bool {
    let result = sqlx::query("SELECT COUNT(*) FROM characters WHERE LOWER(charname) = LOWER($1) and server_id = $2")
        .bind(name)
        .bind(server_id as i16)
        .fetch_one(&pool)
        .await
        .unwrap();

    let count: i64 = result.get(0);
    count == 0
}

pub(crate) async fn start_delete_character(
    pool: PgPool,
    user_id: i32,
    name: String,
    server_id: u16,
    deletion_duration: u32,
) -> bool {
    let result =
            sqlx::query("UPDATE characters SET deletion_end = CURRENT_TIMESTAMP + ($4 * INTERVAL '1 minute') WHERE user_id = $1 AND server_id = $2 AND charname = $3")
                    .bind(user_id)
                    .bind(server_id as i16)
                    .bind(name)
                    .bind(deletion_duration as i32)
                    .execute(&pool)
                    .await
                    .unwrap();
    return result.rows_affected() == 1;
}

pub(crate) async fn restore_character(pool: PgPool, user_id: i32, name: String, server_id: u16) -> bool {
    let result =
            sqlx::query("UPDATE characters SET deletion_end = NULL WHERE user_id = $1 AND server_id = $2 AND charname = $3 AND deletion_end > CURRENT_TIMESTAMP")
                    .bind(user_id)
                    .bind(server_id as i16)
                    .bind(name)
                    .execute(&pool)
                    .await
                    .unwrap();
    return result.rows_affected() == 1;
}

pub(crate) async fn create_character(pool: PgPool, character: Character) {
    let result = sqlx::query("INSERT INTO characters(user_id, server_id, charname, character_type, scale, levels, max_level, strength, intelligence, stat_points, current_hp, current_mp, x, y, z, region, beginner_mark) VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17) RETURNING id")
            .bind(character.character_data.user_id)
            .bind(character.character_data.server_id)
            .bind(character.character_data.charname)
            .bind(character.character_data.character_type)
            .bind(character.character_data.scale)
            .bind(character.character_data.levels)
            .bind(character.character_data.max_level)
            .bind(character.character_data.strength)
            .bind(character.character_data.intelligence)
            .bind(character.character_data.stat_points)
            .bind(character.character_data.current_hp)
            .bind(character.character_data.current_mp)
            .bind(character.character_data.x)
            .bind(character.character_data.y)
            .bind(character.character_data.z)
            .bind(character.character_data.region)
            .bind(character.character_data.beginner_mark)
            .fetch_one(&pool)
            .await.unwrap();

    let id: i32 = result.get(0);
    for item in character.items.iter() {
        sqlx::query(
            "INSERT INTO character_items(character_id, item_obj_id, upgrade_level, slot) VALUES($1, $2, $3, $4)",
        )
        .bind(id)
        .bind(item.item_obj_id)
        .bind(item.upgrade_level)
        .bind(item.slot)
        .execute(&pool)
        .await
        .unwrap();
    }
}
