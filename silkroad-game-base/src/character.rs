use crate::{Race, SpawningState, Stats};
use silkroad_data::masterydata::RefMasteryData;
use std::cmp::max;

pub struct Character {
    pub id: u32,
    pub name: String,
    pub race: Race,
    pub scale: u8,
    pub level: u8,
    pub max_level: u8,
    pub exp: u64,
    pub sp: u32,
    pub sp_exp: u32,
    pub stats: Stats,
    pub stat_points: u16,
    pub current_hp: u32,
    pub current_mp: u32,
    pub berserk_points: u8,
    pub gold: u64,
    pub beginner_mark: bool,
    pub gm: bool,
    pub state: SpawningState,
    pub masteries: Vec<(&'static RefMasteryData, u8)>,
}

impl Character {
    pub fn max_hp(&self) -> u32 {
        self.stats.max_health(self.level)
    }

    pub fn max_mp(&self) -> u32 {
        self.stats.max_mana(self.level)
    }

    pub fn increase_level(&mut self) {
        self.level += 1;
        self.max_level = max(self.max_level, self.level);
        self.stats.increase_strength(1);
        self.stats.increase_intelligence(1);
    }
}
