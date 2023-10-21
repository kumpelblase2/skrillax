use crate::comp::EntityReference;
use crate::sync::Reset;
use bevy_ecs_macros::Component;
use derive_more::Constructor;

const EXP_PER_SP: u64 = 400;

pub(crate) struct ExperienceGained {
    pub(crate) exp: u64,
    pub(crate) sp_exp: u64,
    pub(crate) trigged_level_up: bool,
    pub(crate) from: Option<EntityReference>,
}

#[derive(Component, Default)]
pub(crate) struct Experienced {
    experience: u64,
    sp_exp: u64,
    experience_received: Vec<ExperienceGained>,
}

impl Reset for Experienced {
    fn reset(&mut self) {
        self.experience_received = Vec::new();
    }
}

impl Experienced {
    pub fn new(experience: u64, sp_exp: u64) -> Self {
        Self {
            experience,
            sp_exp,
            experience_received: Vec::new(),
        }
    }

    pub(crate) fn receive(&mut self, exp: u64, sp_exp: u64, from: Option<EntityReference>) {
        self.experience += exp;
        self.sp_exp += sp_exp;
        self.experience_received.push(ExperienceGained {
            exp,
            sp_exp,
            from,
            trigged_level_up: false,
        });
    }

    pub(crate) fn experience(&self) -> u64 {
        self.experience
    }

    pub(crate) fn try_level_up(&mut self, required: u64) -> bool {
        if self.experience >= required {
            self.experience -= required;
            if let Some(last) = self.experience_received.last_mut() {
                last.trigged_level_up = true;
            }
            true
        } else {
            false
        }
    }

    pub(crate) fn convert_sp(&mut self) -> u32 {
        let result = (self.sp_exp / EXP_PER_SP) as u32;
        self.sp_exp %= EXP_PER_SP;
        result
    }

    pub(crate) fn experience_gains(&self) -> &[ExperienceGained] {
        &self.experience_received
    }
}

#[derive(Component, Constructor, Default)]
pub(crate) struct SP {
    sp: u32,
}

impl SP {
    pub(crate) fn gain(&mut self, amount: u32) {
        self.sp += amount;
    }

    pub(crate) fn current(&self) -> u32 {
        self.sp
    }
}

#[derive(Component, Default)]
pub(crate) struct Leveled {
    level: u8,
    leveled_up: i8,
}

impl Reset for Leveled {
    fn reset(&mut self) {
        self.leveled_up = 0;
    }
}

impl Leveled {
    pub(crate) fn current_level(&self) -> u8 {
        self.level
    }

    pub(crate) fn level_up(&mut self) {
        self.level = self.level.saturating_add(1);
        self.leveled_up += 1;
    }

    pub(crate) fn delevel(&mut self) {
        self.level = self.level.saturating_sub(1);
        self.leveled_up -= 1;
    }

    pub(crate) fn did_level(&self) -> bool {
        self.leveled_up > 0
    }

    pub fn new(level: u8) -> Self {
        Self { level, leveled_up: 0 }
    }
}
