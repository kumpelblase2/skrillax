use bevy_ecs::prelude::*;
use derive_more::{Deref, DerefMut};
use silkroad_data::characterdata::RefCharacterData;
use silkroad_game_base::MovementSpeed;

#[derive(Deref, DerefMut, Component, Default)]
pub(crate) struct MovementState(MovementSpeed);

#[derive(Component)]
pub(crate) struct Agent {
    pub(crate) running_speed: f32,
    pub(crate) walking_speed: f32,
    pub(crate) berserk_speed: f32,
}

impl Default for Agent {
    fn default() -> Self {
        Self {
            running_speed: 50.0,
            walking_speed: 16.0,
            berserk_speed: 100.0,
        }
    }
}

impl Agent {
    pub(crate) fn new(walking_speed: f32, running_speed: f32, berserk_speed: f32) -> Self {
        Self {
            running_speed,
            walking_speed,
            berserk_speed,
        }
    }

    pub(crate) fn from_character_data(character_data: &RefCharacterData) -> Self {
        Self {
            running_speed: character_data.run_speed as f32,
            walking_speed: character_data.walk_speed as f32,
            berserk_speed: character_data.berserk_speed as f32,
        }
    }

    pub(crate) fn get_speed_value(&self, speed: MovementSpeed) -> f32 {
        match speed {
            MovementSpeed::Running => self.running_speed,
            MovementSpeed::Walking => self.walking_speed,
            MovementSpeed::Berserk => self.berserk_speed,
        }
    }
}
