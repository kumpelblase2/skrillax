use crate::comp::exp::{Experienced, Leveled, SP};
use crate::comp::gold::GoldChange;
use crate::comp::player::StatPoints;
use crate::comp::pos::Position;
use crate::comp::{Health, Mana};
use axum::async_trait;
use silkroad_game_base::{ChangeProvided, GlobalPosition, Heading, Stats};
use sqlx::PgPool;

#[async_trait]
pub trait ApplyToDatabase: Send + Sync {
    async fn apply(&self, character_id: u32, pool: &PgPool);
}

pub struct PositionChange(GlobalPosition, Heading);

impl ChangeProvided for Position {
    type Change = PositionChange;

    fn as_change(&self) -> Self::Change {
        PositionChange(self.position(), self.rotation())
    }
}

#[async_trait]
impl ApplyToDatabase for PositionChange {
    async fn apply(&self, character_id: u32, pool: &PgPool) {
        let location = self.0.to_local();
        sqlx::query!(
            "UPDATE characters SET x = $1, y = $2, z = $3, region = $4, rotation = $5 WHERE id = $6",
            location.1.x,
            location.1.y,
            location.1.z,
            location.0.id() as i16,
            Into::<u16>::into(self.1) as i16,
            character_id as i32
        )
        .execute(pool)
        .await
        .expect("Should be able to update");
    }
}

pub struct HealthChange(u32);
pub struct ManaChange(u32);
pub struct LevelChange(u16, u16);
pub struct StatsChange(Stats, u16);
pub struct ExperienceChange(u64, u64);
pub struct SpChange(u32);

impl ChangeProvided for Health {
    type Change = HealthChange;

    fn as_change(&self) -> Self::Change {
        HealthChange(self.current_health)
    }
}

impl ChangeProvided for Mana {
    type Change = ManaChange;

    fn as_change(&self) -> Self::Change {
        ManaChange(self.current_mana)
    }
}

impl ChangeProvided for Leveled {
    type Change = LevelChange;

    fn as_change(&self) -> Self::Change {
        LevelChange(self.current_level().into(), self.max_level_reached().into())
    }
}

impl ChangeProvided for StatPoints {
    type Change = StatsChange;

    fn as_change(&self) -> Self::Change {
        StatsChange(self.stats(), self.remaining_points())
    }
}

impl ChangeProvided for Experienced {
    type Change = ExperienceChange;

    fn as_change(&self) -> Self::Change {
        ExperienceChange(self.experience(), self.sp_experience())
    }
}

impl ChangeProvided for SP {
    type Change = SpChange;

    fn as_change(&self) -> Self::Change {
        SpChange(self.current())
    }
}

#[async_trait]
impl ApplyToDatabase for HealthChange {
    async fn apply(&self, character_id: u32, pool: &PgPool) {
        sqlx::query!(
            "UPDATE characters SET current_hp = $1 WHERE id = $2",
            self.0 as i32,
            character_id as i32
        )
        .execute(pool)
        .await
        .expect("Should be able to update");
    }
}

#[async_trait]
impl ApplyToDatabase for ManaChange {
    async fn apply(&self, character_id: u32, pool: &PgPool) {
        sqlx::query!(
            "UPDATE characters SET current_mp = $1 WHERE id = $2",
            self.0 as i32,
            character_id as i32
        )
        .execute(pool)
        .await
        .expect("Should be able to update");
    }
}

#[async_trait]
impl ApplyToDatabase for LevelChange {
    async fn apply(&self, character_id: u32, pool: &PgPool) {
        sqlx::query!(
            "UPDATE characters SET level = $1, max_level = $2 WHERE id = $3",
            self.0 as i16,
            self.1 as i16,
            character_id as i32
        )
        .execute(pool)
        .await
        .expect("Should be able to update");
    }
}

#[async_trait]
impl ApplyToDatabase for StatsChange {
    async fn apply(&self, character_id: u32, pool: &PgPool) {
        sqlx::query!(
            "UPDATE characters SET strength = $1, intelligence = $2, stat_points = $3 WHERE id = $4",
            self.0.strength() as i16,
            self.0.intelligence() as i16,
            self.1 as i16,
            character_id as i32
        )
        .execute(pool)
        .await
        .expect("Should be able to update");
    }
}

#[async_trait]
impl ApplyToDatabase for ExperienceChange {
    async fn apply(&self, character_id: u32, pool: &PgPool) {
        sqlx::query!(
            "UPDATE characters SET exp = $1, sp_exp = $2 WHERE id = $3",
            self.0 as i64,
            self.1 as i64,
            character_id as i32
        )
        .execute(pool)
        .await
        .expect("Should be able to update");
    }
}

#[async_trait]
impl ApplyToDatabase for SpChange {
    async fn apply(&self, character_id: u32, pool: &PgPool) {
        sqlx::query!(
            "UPDATE characters SET sp = $1 WHERE id = $2",
            self.0 as i32,
            character_id as i32
        )
        .execute(pool)
        .await
        .expect("Should be able to update");
    }
}

#[async_trait]
impl ApplyToDatabase for GoldChange {
    async fn apply(&self, character_id: u32, pool: &PgPool) {
        sqlx::query!(
            "UPDATE characters SET gold = $1 WHERE id = $2",
            self.0 as i32,
            character_id as i32
        )
        .execute(pool)
        .await
        .expect("Should be able to update");
    }
}
