use crate::config::SpawnOptions;
use bevy_ecs_macros::Component;
use bevy_time::{Timer, TimerMode};
use rand::random;
use std::time::Duration;

#[derive(Component)]
pub struct Spawner {
    pub active: bool,
    pub radius: f32,
    pub ref_id: u32,
    pub target_amount: usize,
    pub current_amount: usize,
    spawn_check_timer: Timer,
}

impl Spawner {
    pub(crate) fn new(settings: &SpawnOptions, spawned: u32) -> Self {
        Spawner {
            active: false,
            radius: settings.radius,
            target_amount: settings.amount,
            ref_id: spawned,
            current_amount: 0,
            spawn_check_timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
        }
    }

    pub fn has_spots_available(&self) -> bool {
        self.current_amount < self.target_amount
    }

    pub fn should_spawn(&mut self, delta: Duration) -> bool {
        if self.spawn_check_timer.tick(delta).just_finished() {
            return random::<f32>() > 0.5;
        }
        false
    }
}
