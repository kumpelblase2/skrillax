use crate::persistence::ApplyToDatabase;
use axum::async_trait;
use bevy_ecs_macros::Component;
use silkroad_data::skilldata::RefSkillData;
use silkroad_game_base::{Change, ChangeTracked, MergeResult};
use sqlx::PgPool;
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
        return skill
            .required_skills
            .iter()
            .filter(|skill| skill.group != 0)
            .all(|required_skill| {
                self.skills.get(&required_skill.group).copied().unwrap_or(0) >= required_skill.level
            });
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
