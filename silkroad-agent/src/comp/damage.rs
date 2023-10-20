use bevy_ecs_macros::Component;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

#[derive(Component, Default)]
pub(crate) struct DamageReceiver {
    damage_counts: HashMap<u32, u64>,
}

impl DamageReceiver {
    pub(crate) fn record_damage(&mut self, source: u32, amount: u64) {
        match self.damage_counts.entry(source) {
            Entry::Occupied(mut previous) => {
                previous.insert(*previous.get() + amount);
            },
            Entry::Vacant(empty) => {
                empty.insert(amount);
            },
        }
    }

    pub(crate) fn total_damage_of(&self, source: u32) -> u64 {
        self.damage_counts.get(&source).copied().unwrap_or(0)
    }

    pub(crate) fn all_attackers(&self) -> impl Iterator<Item = u32> + '_ {
        self.damage_counts.keys().copied()
    }
}

#[derive(Component, Default)]
pub(crate) struct Invincible {
    by_command: bool,
}

impl Invincible {
    pub(crate) fn from_command() -> Self {
        Invincible { by_command: true }
    }
}
