use crate::persistence::ApplyToDatabase;
use axum::async_trait;
use bevy::prelude::*;
use silkroad_data::skilldata::RefSkillData;
use silkroad_game_base::{Change, ChangeTracked, MergeResult};
use silkroad_protocol::skill::HotbarItem;
use sqlx::{PgPool, QueryBuilder};
use std::collections::HashMap;
use std::mem;

#[derive(Component)]
pub(crate) struct SkillBook {
    skills: HashMap<u32, u8>,
    new_skills: Vec<(u32, u8)>, // u32 should be more like RefId(u32)
}

impl SkillBook {
    pub(crate) fn new(skills: &[(u32, u8)]) -> Self {
        Self {
            skills: skills.iter().copied().collect(),
            new_skills: Vec::new(),
        }
    }

    pub(crate) fn learn_skill(&mut self, skill: &'static RefSkillData) {
        self.skills.insert(skill.group, skill.level);
        self.new_skills.push((skill.group, skill.level));
    }

    pub(crate) fn has_required_skills_for(&self, skill: &RefSkillData) -> bool {
        skill
            .required_skills
            .iter()
            .filter(|skill| skill.group != 0)
            .all(|required_skill| self.skills.get(&required_skill.group).copied().unwrap_or(0) >= required_skill.level)
    }
}

pub(crate) struct LearnedSkill(u32, u8);

impl Change for LearnedSkill {
    fn merge(self, other: Self) -> MergeResult<Self> {
        if other.0 == self.0 {
            MergeResult::Merged(other)
        } else {
            MergeResult::Unchanged(self, other)
        }
    }
}

impl ChangeTracked for SkillBook {
    type ChangeItem = LearnedSkill;

    fn changes(&mut self) -> Vec<Self::ChangeItem> {
        mem::take(&mut self.new_skills)
            .into_iter()
            .map(|s| LearnedSkill(s.0, s.1))
            .collect::<Vec<_>>()
    }
}

#[async_trait]
impl ApplyToDatabase for LearnedSkill {
    async fn apply(&self, character_id: u32, pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO character_skills(character_id, skill_group_id, level) VALUES($1, $2, $3) ON CONFLICT(skill_group_id, character_id) DO UPDATE SET level = EXCLUDED.level",
            character_id as i32,
            self.0 as i32,
            self.1 as i16
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

#[derive(Component)]
pub(crate) struct Hotbar {
    rows: [Option<(u8, u32)>; 101],
    changed: bool,
}

impl Default for Hotbar {
    fn default() -> Self {
        Hotbar {
            rows: [None; 101],
            changed: false,
        }
    }
}

impl Hotbar {
    pub fn from_list(list: &[(u8, u8, u32)]) -> Self {
        let mut hotbar = Self::default();

        for (slot, kind, data) in list {
            hotbar.rows[*slot as usize] = Some((*kind, *data));
        }

        hotbar
    }

    pub fn used_entries(&self) -> impl Iterator<Item = (u8, u8, u32)> + use<'_> {
        self.rows
            .iter()
            .enumerate()
            .filter_map(|(i, entry)| entry.map(|(kind, data)| (i as u8, kind, data)))
    }

    pub fn as_list(&self) -> Vec<(u8, u8, u32)> {
        self.used_entries().collect()
    }

    pub fn update_entries(&mut self, slots: &[HotbarItem]) {
        self.changed = true;
        for i in 0..self.rows.len() {
            self.rows[i] = None;
        }
        for HotbarItem {
            slot,
            action_flag,
            action_data,
        } in slots
        {
            self.rows[*slot as usize] = Some((*action_flag, *action_data));
        }
    }
}

pub enum HotbarChange {
    None,
    NewContent(Vec<(u8, u8, u32)>),
}

impl ChangeTracked for Hotbar {
    type ChangeItem = HotbarChange;

    fn changes(&mut self) -> Vec<Self::ChangeItem> {
        let changed = self.changed;
        self.changed = false;
        if changed {
            vec![HotbarChange::NewContent(self.as_list())]
        } else {
            vec![HotbarChange::None]
        }
    }
}

pub struct HotbarContent(Vec<(u8, u8, u32)>);

impl Change for HotbarChange {
    fn merge(self, other: Self) -> MergeResult<Self> {
        let result = match (self, other) {
            (_, HotbarChange::NewContent(content)) => HotbarChange::NewContent(content),
            (HotbarChange::NewContent(content), HotbarChange::None) => HotbarChange::NewContent(content),
            (HotbarChange::None, HotbarChange::None) => HotbarChange::None,
        };
        MergeResult::Merged(result)
    }
}

#[async_trait]
impl ApplyToDatabase for HotbarChange {
    async fn apply(&self, character_id: u32, pool: &PgPool) -> Result<(), sqlx::Error> {
        let HotbarChange::NewContent(changes) = self else {
            return Ok(());
        };

        sqlx::query!(
            "DELETE FROM hotbar_entries WHERE character_id = $1",
            character_id as i32,
        )
        .execute(pool)
        .await?;

        let mut query_builder = QueryBuilder::new("INSERT INTO hotbar_entries(character_id, slot, kind, data) ");
        query_builder.push_values(changes.iter(), |mut builder, (slot, kind, data)| {
            builder
                .push_bind(character_id as i32)
                .push_bind(*slot as i16)
                .push_bind(*kind as i16)
                .push_bind(*data as i32);
        });
        query_builder.build().execute(pool).await?;
        Ok(())
    }
}
