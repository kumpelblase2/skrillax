use crate::{Race, SpawningState, Stats};
use silkroad_data::masterydata::RefMasteryData;
use silkroad_data::skilldata::RefSkillData;

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
    pub skills: Vec<&'static RefSkillData>,
}
