use bevy_core::Time;
use bevy_ecs::system::{Res, ResMut};
use rand::random;
use std::time::Duration;

const MINUTES_IN_A_DAY: f64 = 24.0 * 60.0;

fn get_official_per_day_duration() -> Duration {
    let ingame_seconds_per_second = 49.5;
    let required_real_seconds = (MINUTES_IN_A_DAY * 60.0) / ingame_seconds_per_second;
    Duration::from_secs_f64(required_real_seconds)
}

pub struct DaylightCycle {
    moon: u16,
    full_day: Duration,
    time: Duration,
}

impl DaylightCycle {
    pub fn new(one_day: Duration) -> Self {
        let moon = random::<u16>();
        Self {
            moon,
            full_day: one_day,
            time: Duration::default(),
        }
    }

    pub fn official() -> Self {
        Self::new(get_official_per_day_duration())
    }

    pub fn advance(&mut self, amount: Duration) {
        self.time = self.time + amount;
        if self.time >= self.full_day {
            self.moon = self.moon.wrapping_add(1);
            self.time = self.time - self.full_day;
        }
    }

    pub fn moon(&self) -> u16 {
        self.moon
    }

    pub fn time(&self) -> (u8, u8) {
        let time_progressed = self.time.as_secs_f64() / self.full_day.as_secs_f64();
        let ingame_progress = (time_progressed * MINUTES_IN_A_DAY) as u16;
        let minutes = ingame_progress % 60;
        let hours = ingame_progress / 60;
        (hours as u8, minutes as u8)
    }
}

pub fn advance_daylight(mut cycle: ResMut<DaylightCycle>, time: Res<Time>) {
    cycle.advance(time.delta());
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_moon_increase() {
        let full_day = Duration::from_secs(5);
        let mut cycle = DaylightCycle::new(full_day);
        let moon_start = cycle.moon();
        cycle.advance(full_day);
        let moon_next = cycle.moon();
        assert_ne!(moon_next, moon_start);
    }

    #[test]
    pub fn test_time_convert() {
        let full_day = Duration::from_secs(24 * 60);
        let mut cycle = DaylightCycle::new(full_day);
        assert!(matches!(cycle.time(), (0, 0)));
        cycle.advance(Duration::from_secs(60));
        assert!(matches!(cycle.time(), (1, 0)));
        cycle.advance(Duration::from_secs(60));
        assert!(matches!(cycle.time(), (2, 0)));
        cycle.advance(Duration::from_secs(30));
        assert!(matches!(cycle.time(), (2, 30)));
        cycle.advance(Duration::from_secs(30));
        assert!(matches!(cycle.time(), (3, 0)));
    }

    #[test]
    pub fn test_accurate() {
        let mut cycle = DaylightCycle::official();
        cycle.advance(Duration::from_secs(1042));
        let (hour, minute) = cycle.time();
        assert_eq!(hour, 14);
        assert_eq!(minute, 19);
        cycle.advance(Duration::from_secs(292));
        let (hour, minute) = cycle.time();
        assert_eq!(hour, 18);
        assert_eq!(minute, 20);
    }
}
