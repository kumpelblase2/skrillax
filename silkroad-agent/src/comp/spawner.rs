use crate::config::SpawnOptions;
use bevy::prelude::*;
use rand::random;
use silkroad_data::characterdata::RefCharacterData;
use std::time::Duration;
use tracing::trace;

#[derive(Component)]
pub struct Spawner {
    pub active: bool,
    pub radius: f32,
    pub reference: &'static RefCharacterData,
    pub target_amount: usize,
    current_amount: usize,
    spawn_check_timer: Timer,
}

impl Spawner {
    pub(crate) fn new(settings: &SpawnOptions, spawned: &'static RefCharacterData) -> Self {
        Spawner {
            active: false,
            radius: settings.radius,
            target_amount: settings.amount,
            reference: spawned,
            current_amount: 0,
            spawn_check_timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
        }
    }

    pub fn deactivate(&mut self) {
        self.active = false;
        self.current_amount = 0;
    }

    pub fn has_spots_available(&self) -> bool {
        self.current_amount < self.target_amount
    }

    pub fn available_spots(&self) -> usize {
        self.target_amount - self.current_amount
    }

    pub fn should_spawn(&mut self, delta: Duration) -> bool {
        if self.has_spots_available() && self.spawn_check_timer.tick(delta).just_finished() {
            return random::<f32>() > 0.5;
        }
        false
    }

    pub fn increase_alive(&mut self) {
        self.increase_alive_by(1);
    }

    pub fn increase_alive_by(&mut self, amount: usize) {
        let current = self.current_amount;
        self.current_amount = self.current_amount.saturating_add(amount);
        let new = self.current_amount;
        if self.active {
            trace!(
                "Spawner changed capacity: {}/{} -> {}/{}",
                current,
                self.target_amount,
                new,
                self.target_amount
            );
        }
    }

    pub fn decrease_alive_by(&mut self, amount: usize) {
        let current = self.current_amount;
        self.current_amount = self.current_amount.saturating_sub(amount);
        let new = self.current_amount;
        if self.active {
            trace!(
                "Spawner changed capacity: {}/{} -> {}/{}",
                current,
                self.target_amount,
                new,
                self.target_amount
            );
        }
    }

    pub fn decrease_alive(&mut self) {
        self.decrease_alive_by(1);
    }
}
