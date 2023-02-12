use crate::config::SpawnOptions;
use bevy_ecs_macros::Component;
use rand::random;
use std::time::{Duration, Instant};

#[derive(Component)]
pub struct Spawner {
    pub active: bool,
    pub radius: f32,
    pub ref_id: u32,
    pub target_amount: usize,
    pub current_amount: usize,
    last_spawn_check: Instant,
    spawn_check_duration: Duration,
}

impl Spawner {
    pub(crate) fn new(settings: &SpawnOptions, spawned: u32) -> Self {
        Spawner {
            active: false,
            radius: settings.radius,
            target_amount: settings.amount,
            ref_id: spawned,
            current_amount: 0,
            last_spawn_check: Instant::now(),
            spawn_check_duration: Duration::from_secs(1),
        }
    }

    pub fn has_spots_available(&self) -> bool {
        self.current_amount < self.target_amount
    }

    pub fn should_spawn(&mut self, now: Instant) -> bool {
        if now.duration_since(self.last_spawn_check) > self.spawn_check_duration {
            self.last_spawn_check = now;
            return random::<f32>() > 0.5;
        }
        false
    }
}
