use bevy_ecs_macros::Component;
use bevy_time::{Timer, TimerMode};
use std::time::Duration;

#[derive(Component)]
pub struct Persistable {
    time_until_next_persist: Timer,
}

impl Persistable {
    pub fn from_seconds(seconds: u32) -> Self {
        Self {
            time_until_next_persist: Timer::from_seconds(seconds as f32, TimerMode::Repeating),
        }
    }

    pub fn should_persist(&mut self, time_passed: Duration) -> bool {
        self.time_until_next_persist.tick(time_passed).just_finished()
    }
}
