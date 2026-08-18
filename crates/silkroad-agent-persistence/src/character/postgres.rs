use super::{
    invalid_value, CharacterListItem, CharacterOwner, CharacterRace, CharacterReadError, CharacterWorldItem,
    CharacterWriteError, ListedCharacter, NewCharacter, SavedLocation, WorldJoinCharacter, WorldJoinError,
};
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use std::collections::HashMap;
use std::fmt::Display;

#[derive(sqlx::Type, Clone, Copy)]
#[sqlx(type_name = "race")]
#[sqlx(rename_all = "lowercase")]
enum DbRace {
    Chinese,
    European,
}

impl From<DbRace> for CharacterRace {
    fn from(value: DbRace) -> Self {
        match value {
            DbRace::Chinese => Self::Chinese,
            DbRace::European => Self::European,
        }
    }
}

impl From<CharacterRace> for DbRace {
    fn from(value: CharacterRace) -> Self {
        match value {
            CharacterRace::Chinese => Self::Chinese,
            CharacterRace::European => Self::European,
        }
    }
}

struct CharacterListRow {
    id: i32,
    charname: String,
    character_type: i32,
    scale: i16,
    level: i16,
    exp: i64,
    sp: i32,
    strength: i16,
    intelligence: i16,
    stat_points: i16,
    current_hp: i32,
    current_mp: i32,
    region: i16,
    deletion_end: Option<DateTime<Utc>>,
    last_logout: Option<DateTime<Utc>>,
}

struct CharacterListItemRow {
    character_id: i32,
    item_obj_id: i32,
    upgrade_level: i16,
    slot: i16,
}

struct WorldJoinRow {
    id: i32,
    charname: String,
    race: DbRace,
    character_type: i32,
    scale: i16,
    level: i16,
    max_level: i16,
    exp: i64,
    sp: i32,
    sp_exp: i32,
    strength: i16,
    intelligence: i16,
    stat_points: i16,
    current_hp: i32,
    current_mp: i32,
    x: f32,
    y: f32,
    z: f32,
    rotation: i16,
    region: i16,
    berserk_points: i16,
    gold: i64,
    beginner_mark: bool,
    gm: bool,
    deletion_end: Option<DateTime<Utc>>,
}

struct WorldItemRow {
    item_obj_id: i32,
    upgrade_level: i16,
    variance: Option<i64>,
    slot: i16,
    amount: i16,
}

struct MasteryRow {
    mastery_id: i32,
    level: i16,
}

struct SkillRow {
    skill_group_id: i32,
    level: i16,
}

struct HotbarRow {
    slot: i16,
    kind: i16,
    data: i32,
}

pub(super) async fn is_name_available(pool: &PgPool, server_id: u16, name: &str) -> Result<bool, CharacterWriteError> {
    let result = sqlx::query!(
        "SELECT COUNT(*) as \"count!\" FROM characters WHERE LOWER(charname) = LOWER($1) and server_id = $2",
        name,
        i32::from(server_id),
    )
    .fetch_one(pool)
    .await?;

    Ok(result.count == 0)
}

pub(super) async fn record_character_logout(pool: &PgPool, character_id: u32) -> Result<(), CharacterWriteError> {
    let character_id: i32 = write_checked("character_id", character_id)?;
    sqlx::query!(
        "UPDATE characters SET last_logout = CURRENT_TIMESTAMP WHERE id = $1",
        character_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub(super) async fn start_character_deletion(
    pool: &PgPool,
    owner: CharacterOwner,
    name: &str,
    deletion_minutes: u32,
) -> Result<bool, CharacterWriteError> {
    let deletion_minutes = f64::from(deletion_minutes);
    let result = sqlx::query!(
        "UPDATE characters SET deletion_end = CURRENT_TIMESTAMP + ($4 * INTERVAL '1 minute') WHERE user_id = $1 AND server_id = $2 AND charname = $3",
        owner.user_id,
        i32::from(owner.server_id),
        name,
        deletion_minutes,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() == 1)
}

pub(super) async fn restore_character(
    pool: &PgPool,
    owner: CharacterOwner,
    name: &str,
) -> Result<bool, CharacterWriteError> {
    let result = sqlx::query!(
        "UPDATE characters SET deletion_end = NULL WHERE user_id = $1 AND server_id = $2 AND charname = $3 AND deletion_end > CURRENT_TIMESTAMP",
        owner.user_id,
        i32::from(owner.server_id),
        name,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() == 1)
}

pub(super) async fn create_character(pool: &PgPool, character: NewCharacter) -> Result<(), CharacterWriteError> {
    let reference_id: i32 = write_checked("reference_id", character.reference_id)?;
    let strength: i16 = write_checked("strength", character.strength)?;
    let intelligence: i16 = write_checked("intelligence", character.intelligence)?;
    let stat_points: i16 = write_checked("stat_points", character.stat_points)?;
    let current_hp: i32 = write_checked("current_hp", character.current_hp)?;
    let current_mp: i32 = write_checked("current_mp", character.current_mp)?;
    let gold: i64 = write_checked("gold", character.gold)?;
    let region = i16::from_ne_bytes(character.region.to_ne_bytes());
    let race = DbRace::from(character.race);

    let mut transaction = pool.begin().await?;
    let result = sqlx::query!(
        r#"
        INSERT INTO characters(
            user_id, server_id, charname, race, character_type, scale, level,
            max_level, strength, intelligence, stat_points, current_hp, current_mp,
            x, y, z, region, beginner_mark, gold
        )
        VALUES(
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
            $11, $12, $13, $14, $15, $16, $17, $18, $19
        )
        RETURNING id
        "#,
        character.owner.user_id,
        i32::from(character.owner.server_id),
        &character.name,
        race as DbRace,
        reference_id,
        i16::from(character.scale),
        i16::from(character.level),
        i16::from(character.max_level),
        strength,
        intelligence,
        stat_points,
        current_hp,
        current_mp,
        character.x,
        character.y,
        character.z,
        region,
        character.beginner_mark,
        gold,
    )
    .fetch_one(&mut *transaction)
    .await?;

    for item in character.items {
        let reference_id: i32 = write_checked("item.reference_id", item.reference_id)?;
        let variance: Option<i64> = item
            .variance
            .map(|value| write_checked("item.variance", value))
            .transpose()?;
        let amount: i16 = write_checked("item.amount", item.amount)?;
        sqlx::query!(
            r#"
            INSERT INTO character_items(
                character_id, item_obj_id, upgrade_level, slot, variance, amount
            )
            VALUES($1, $2, $3, $4, $5, $6)
            "#,
            result.id,
            reference_id,
            i16::from(item.upgrade_level),
            i16::from(item.slot),
            variance,
            amount,
        )
        .execute(&mut *transaction)
        .await?;
    }

    transaction.commit().await?;
    Ok(())
}

pub(super) async fn load_character_list(
    pool: &PgPool,
    owner: CharacterOwner,
) -> Result<Vec<ListedCharacter>, CharacterReadError> {
    let mut transaction = pool.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut *transaction)
        .await?;

    let rows = sqlx::query_as!(
        CharacterListRow,
        r#"
        SELECT id, charname, character_type, scale, level, exp, sp,
               strength, intelligence, stat_points, current_hp, current_mp,
               region, deletion_end, last_logout
        FROM characters
        WHERE user_id = $1
          AND server_id = $2
          AND (deletion_end > NOW() OR deletion_end IS NULL)
        ORDER BY id
        "#,
        owner.user_id,
        i32::from(owner.server_id),
    )
    .fetch_all(&mut *transaction)
    .await?;

    let character_ids = rows.iter().map(|row| row.id).collect::<Vec<_>>();
    let item_rows = if character_ids.is_empty() {
        Vec::new()
    } else {
        sqlx::query_as!(
            CharacterListItemRow,
            r#"
            SELECT character_id, item_obj_id, upgrade_level, slot
            FROM character_items
            WHERE character_id = ANY($1)
              AND slot < 13
            ORDER BY character_id, slot
            "#,
            &character_ids,
        )
        .fetch_all(&mut *transaction)
        .await?
    };

    transaction.commit().await?;

    let mut items_by_character: HashMap<i32, Vec<CharacterListItemRow>> = HashMap::new();
    for item in item_rows {
        items_by_character.entry(item.character_id).or_default().push(item);
    }

    rows.into_iter()
        .map(|row| {
            let items = items_by_character.remove(&row.id).unwrap_or_default();
            listed_character(row, items)
        })
        .collect()
}

pub(super) async fn load_world_join(
    pool: &PgPool,
    owner: CharacterOwner,
    name: &str,
) -> Result<WorldJoinCharacter, WorldJoinError> {
    let mut transaction = pool.begin().await.map_err(CharacterReadError::from)?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut *transaction)
        .await
        .map_err(CharacterReadError::from)?;

    let row = sqlx::query_as!(
        WorldJoinRow,
        r#"
        SELECT id, charname, race AS "race!: DbRace", character_type, scale,
               level, max_level, exp, sp, sp_exp, strength, intelligence,
               stat_points, current_hp, current_mp, x, y, z, rotation, region,
               berserk_points, gold, beginner_mark, gm, deletion_end
        FROM characters
        WHERE user_id = $1
          AND server_id = $2
          AND charname = $3
          AND (deletion_end > NOW() OR deletion_end IS NULL)
        "#,
        owner.user_id,
        i32::from(owner.server_id),
        name,
    )
    .fetch_optional(&mut *transaction)
    .await
    .map_err(CharacterReadError::from)?
    .ok_or(WorldJoinError::NotFound)?;

    if row.deletion_end.is_some() {
        return Err(WorldJoinError::PendingDeletion);
    }

    let character_id = row.id;
    let items = load_world_items(&mut transaction, character_id).await?;
    let masteries = load_masteries(&mut transaction, character_id).await?;
    let skills = load_skills(&mut transaction, character_id).await?;
    let hotbar = load_hotbar(&mut transaction, character_id).await?;

    let character = world_join_character(row, items, masteries, skills, hotbar)?;
    transaction.commit().await.map_err(CharacterReadError::from)?;
    Ok(character)
}

async fn load_world_items(
    transaction: &mut Transaction<'_, Postgres>,
    character_id: i32,
) -> Result<Vec<WorldItemRow>, CharacterReadError> {
    Ok(sqlx::query_as!(
        WorldItemRow,
        r#"
        SELECT item_obj_id, upgrade_level, variance, slot, amount
        FROM character_items
        WHERE character_id = $1
        ORDER BY slot
        "#,
        character_id,
    )
    .fetch_all(&mut **transaction)
    .await?)
}

async fn load_masteries(
    transaction: &mut Transaction<'_, Postgres>,
    character_id: i32,
) -> Result<Vec<MasteryRow>, CharacterReadError> {
    Ok(sqlx::query_as!(
        MasteryRow,
        r#"
        SELECT mastery_id, level
        FROM character_masteries
        WHERE character_id = $1
        ORDER BY mastery_id
        "#,
        character_id,
    )
    .fetch_all(&mut **transaction)
    .await?)
}

async fn load_skills(
    transaction: &mut Transaction<'_, Postgres>,
    character_id: i32,
) -> Result<Vec<SkillRow>, CharacterReadError> {
    Ok(sqlx::query_as!(
        SkillRow,
        r#"
        SELECT skill_group_id, level
        FROM character_skills
        WHERE character_id = $1
        ORDER BY skill_group_id
        "#,
        character_id,
    )
    .fetch_all(&mut **transaction)
    .await?)
}

async fn load_hotbar(
    transaction: &mut Transaction<'_, Postgres>,
    character_id: i32,
) -> Result<Vec<HotbarRow>, CharacterReadError> {
    Ok(sqlx::query_as!(
        HotbarRow,
        r#"
        SELECT slot, kind, data
        FROM hotbar_entries
        WHERE character_id = $1
        ORDER BY slot
        "#,
        character_id,
    )
    .fetch_all(&mut **transaction)
    .await?)
}

fn listed_character(
    row: CharacterListRow,
    items: Vec<CharacterListItemRow>,
) -> Result<ListedCharacter, CharacterReadError> {
    let id = row.id;
    Ok(ListedCharacter {
        id: checked(id, "id", row.id)?,
        name: row.charname,
        reference_id: checked(id, "character_type", row.character_type)?,
        scale: checked(id, "scale", row.scale)?,
        level: checked(id, "level", row.level)?,
        experience: checked(id, "exp", row.exp)?,
        skill_points: checked(id, "sp", row.sp)?,
        strength: checked(id, "strength", row.strength)?,
        intelligence: checked(id, "intelligence", row.intelligence)?,
        stat_points: checked(id, "stat_points", row.stat_points)?,
        current_hp: checked(id, "current_hp", row.current_hp)?,
        current_mp: checked(id, "current_mp", row.current_mp)?,
        region: u16::from_ne_bytes(row.region.to_ne_bytes()),
        deletion_end: row.deletion_end,
        last_logout: row.last_logout,
        equipped_items: items
            .into_iter()
            .map(|item| {
                Ok(CharacterListItem {
                    slot: checked(id, "item.slot", item.slot)?,
                    reference_id: checked(id, "item.item_obj_id", item.item_obj_id)?,
                    upgrade_level: checked(id, "item.upgrade_level", item.upgrade_level)?,
                })
            })
            .collect::<Result<_, CharacterReadError>>()?,
    })
}

fn world_join_character(
    row: WorldJoinRow,
    items: Vec<WorldItemRow>,
    masteries: Vec<MasteryRow>,
    skills: Vec<SkillRow>,
    hotbar: Vec<HotbarRow>,
) -> Result<WorldJoinCharacter, CharacterReadError> {
    let id = row.id;
    for (field, value) in [("x", row.x), ("y", row.y), ("z", row.z)] {
        if !value.is_finite() {
            return Err(CharacterReadError::InvalidStoredData {
                character_id: id,
                field,
                reason: "position must be finite".to_owned(),
            });
        }
    }

    Ok(WorldJoinCharacter {
        id: checked(id, "id", row.id)?,
        name: row.charname,
        race: row.race.into(),
        reference_id: checked(id, "character_type", row.character_type)?,
        scale: checked(id, "scale", row.scale)?,
        level: checked(id, "level", row.level)?,
        max_level: checked(id, "max_level", row.max_level)?,
        experience: checked(id, "exp", row.exp)?,
        skill_points: checked(id, "sp", row.sp)?,
        skill_experience: checked(id, "sp_exp", row.sp_exp)?,
        strength: checked(id, "strength", row.strength)?,
        intelligence: checked(id, "intelligence", row.intelligence)?,
        stat_points: checked(id, "stat_points", row.stat_points)?,
        current_hp: checked(id, "current_hp", row.current_hp)?,
        current_mp: checked(id, "current_mp", row.current_mp)?,
        berserk_points: checked(id, "berserk_points", row.berserk_points)?,
        gold: checked(id, "gold", row.gold)?,
        beginner_mark: row.beginner_mark,
        game_master: row.gm,
        location: SavedLocation {
            region: u16::from_ne_bytes(row.region.to_ne_bytes()),
            x: row.x,
            y: row.y,
            z: row.z,
            heading: u16::from_ne_bytes(row.rotation.to_ne_bytes()),
        },
        items: items
            .into_iter()
            .map(|item| {
                Ok(CharacterWorldItem {
                    reference_id: checked(id, "item.item_obj_id", item.item_obj_id)?,
                    upgrade_level: checked(id, "item.upgrade_level", item.upgrade_level)?,
                    variance: item
                        .variance
                        .map(|value| checked(id, "item.variance", value))
                        .transpose()?,
                    slot: checked(id, "item.slot", item.slot)?,
                    amount: checked(id, "item.amount", item.amount)?,
                })
            })
            .collect::<Result<_, CharacterReadError>>()?,
        masteries: masteries
            .into_iter()
            .map(|mastery| {
                Ok((
                    checked(id, "mastery.mastery_id", mastery.mastery_id)?,
                    checked(id, "mastery.level", mastery.level)?,
                ))
            })
            .collect::<Result<_, CharacterReadError>>()?,
        skills: skills
            .into_iter()
            .map(|skill| {
                Ok((
                    checked(id, "skill.skill_group_id", skill.skill_group_id)?,
                    checked(id, "skill.level", skill.level)?,
                ))
            })
            .collect::<Result<_, CharacterReadError>>()?,
        hotbar: hotbar
            .into_iter()
            .map(|entry| {
                Ok((
                    checked(id, "hotbar.slot", entry.slot)?,
                    checked(id, "hotbar.kind", entry.kind)?,
                    checked(id, "hotbar.data", entry.data)?,
                ))
            })
            .collect::<Result<_, CharacterReadError>>()?,
    })
}

fn checked<T, U>(character_id: i32, field: &'static str, value: T) -> Result<U, CharacterReadError>
where
    T: Copy + Display,
    U: TryFrom<T>,
{
    U::try_from(value).map_err(|_| invalid_value(character_id, field, value))
}

fn write_checked<T, U>(field: &'static str, value: T) -> Result<U, CharacterWriteError>
where
    U: TryFrom<T>,
{
    U::try_from(value).map_err(|_| CharacterWriteError::ValueOutOfRange { field })
}
