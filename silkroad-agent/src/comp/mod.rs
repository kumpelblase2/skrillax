pub(crate) mod damage;
pub(crate) mod drop;
pub(crate) mod exp;
pub(crate) mod gold;
pub(crate) mod inventory;
pub(crate) mod mastery;
pub(crate) mod monster;
pub(crate) mod net;
pub(crate) mod npc;
pub(crate) mod player;
pub(crate) mod pos;
pub(crate) mod skill;
pub(crate) mod spawner;
pub(crate) mod visibility;

use crate::db::user::ServerUser;
use crate::population::capacity::PlayingToken;
use crate::sync::Reset;
use bevy::prelude::*;
use std::time::Duration;

#[derive(Component, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct GameEntity {
    pub unique_id: u32,
    pub ref_id: u32,
}

#[derive(Component)]
pub(crate) struct Playing(pub(crate) ServerUser, pub(crate) PlayingToken);

#[derive(Component, Copy, Clone)]
pub(crate) struct Health {
    pub current_health: u32,
    pub max_health: u32,
    pub change: Option<i32>,
}

impl Reset for Health {
    fn reset(&mut self) {
        self.change = None;
    }
}

impl Health {
    pub fn new(max_health: u32) -> Self {
        Self {
            current_health: max_health,
            max_health,
            change: None,
        }
    }

    pub fn reduce(&mut self, amount: u32) {
        let before = self.current_health;
        self.current_health = self.current_health.saturating_sub(amount);
        self.add_change(-(before as i32))
    }

    pub fn regenerate(&mut self, amount: u32) {
        let before = self.current_health;
        self.current_health = self.current_health.saturating_add(amount);
        self.add_change(before as i32)
    }

    fn add_change(&mut self, amount: i32) {
        self.change = match self.change {
            Some(previous_change) => Some(previous_change + amount),
            None => Some(amount),
        }
    }

    pub fn is_dead(&self) -> bool {
        self.current_health == 0
    }

    pub fn upgrade(&mut self, new_max: u32) {
        let diff = new_max - self.current_health;
        self.max_health = new_max;
        self.increase_max(new_max);
        self.add_change(diff as i32)
    }

    pub fn increase_max(&mut self, new_max: u32) {
        self.max_health = new_max;
    }

    pub fn collect_change(&self) -> Option<i32> {
        self.change.as_ref().copied()
    }
}

#[derive(Component)]
pub(crate) struct Mana {
    pub current_mana: u32,
    pub max_mana: u32,
    pub change: Option<i32>,
}

impl Reset for Mana {
    fn reset(&mut self) {
        self.change = None;
    }
}

impl Mana {
    pub fn with_max(max: u32) -> Self {
        Mana {
            current_mana: max,
            max_mana: max,
            change: None,
        }
    }

    fn add_change(&mut self, amount: i32) {
        self.change = match self.change {
            Some(previous_change) => Some(previous_change + amount),
            None => Some(amount),
        }
    }

    pub fn upgrade(&mut self, new_max: u32) {
        let diff = new_max - self.current_mana;
        self.max_mana = new_max;
        self.increase_max(new_max);
        self.add_change(diff as i32)
    }

    pub fn spend(&mut self, amount: u32) {
        self.current_mana = self.max_mana.saturating_sub(amount);
        self.add_change(-(amount as i32));
    }

    pub fn increase_max(&mut self, new_max: u32) {
        self.current_mana = new_max;
    }

    pub fn collect_change(&self) -> Option<i32> {
        self.change.as_ref().copied()
    }
}

#[derive(Hash, Copy, Clone, Eq, PartialEq)]
pub struct EntityReference(pub Entity, pub(crate) GameEntity);

#[derive(Clone, Component)]
pub struct Despawn(pub Timer);

impl Despawn {
    pub fn despawn_after_seconds(seconds: u64) -> Despawn {
        Despawn(Timer::from_seconds(seconds as f32, TimerMode::Once))
    }
}

impl From<Duration> for Despawn {
    fn from(duration: Duration) -> Self {
        Despawn(Timer::new(duration, TimerMode::Once))
    }
}
