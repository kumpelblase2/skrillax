use crate::persistence::ApplyToDatabase;
use crate::sync::Reset;
use axum::async_trait;
use bevy_ecs_macros::Component;
use silkroad_game_base::ChangeProvided;
use sqlx::{PgPool, QueryBuilder};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::mem;

#[derive(Component, Clone)]
pub(crate) struct MasteryKnowledge {
    masteries: HashMap<u32, u8>,
    leveled_masteries: Vec<u32>,
}

impl Reset for MasteryKnowledge {
    fn reset(&mut self) {
        mem::take(&mut self.leveled_masteries);
    }
}

impl MasteryKnowledge {
    pub(crate) fn new(values: &[(u32, u8)]) -> Self {
        MasteryKnowledge {
            masteries: values.iter().copied().collect(),
            leveled_masteries: vec![],
        }
    }

    pub(crate) fn level_mastery_by(&mut self, ref_id: u32, amount: u8) {
        match self.masteries.entry(ref_id) {
            Entry::Occupied(mut entry) => {
                entry.insert(*entry.get() + amount);
            },
            Entry::Vacant(entry) => {
                entry.insert(amount);
            },
        }
        self.leveled_masteries.push(ref_id);
    }

    pub(crate) fn level_of(&self, ref_id: u32) -> Option<u8> {
        self.masteries.get(&ref_id).copied()
    }

    pub(crate) fn total(&self) -> u16 {
        self.masteries.values().map(|v| u16::from(*v)).sum()
    }

    pub(crate) fn updated(&self) -> &[u32] {
        &self.leveled_masteries
    }
}

pub(crate) struct MasteryChange(HashMap<u32, u8>);

impl ChangeProvided for MasteryKnowledge {
    type Change = MasteryChange;

    fn as_change(&self) -> Self::Change {
        MasteryChange(self.masteries.clone())
    }
}

#[async_trait]
impl ApplyToDatabase for MasteryChange {
    async fn apply(&self, character_id: u32, pool: &PgPool) {
        if self.0.is_empty() {
            return;
        }

        let mut query = QueryBuilder::new("INSERT INTO character_masteries(character_id, mastery_id, level)");
        let rows = self
            .0
            .iter()
            .map(|(key, value)| (character_id, *key, *value))
            .collect::<Vec<_>>();
        query.push_values(&rows, |mut bind, (char_id, ref_id, level)| {
            bind.push_bind(*char_id as i32)
                .push_bind(*ref_id as i32)
                .push_bind(*level as i16);
        });

        query
            .push(" ON CONFLICT(character_id, mastery_id) DO UPDATE SET level = EXCLUDED.level")
            .build()
            .execute(pool)
            .await
            .expect("Should be able to update masteries.");
    }
}
