use crate::Game;
use std::ops::{Div, Sub};
use std::thread::sleep;
use std::time::{Duration, Instant};
use tracing::{trace_span, warn};

pub(crate) struct Executor {
    game: Game,
    desired_ticks: u32,
}

impl Executor {
    pub(crate) fn new(game: Game, desired_ticks: u32) -> Self {
        Executor { game, desired_ticks }
    }

    pub(crate) fn run(&mut self) {
        let time_per_frame = Duration::from_secs(1).div(self.desired_ticks);
        let mut last_tick = Instant::now();
        loop {
            let span = trace_span!("tick");
            let guard = span.enter();
            let start_time = Instant::now();
            self.game.tick();
            let end_time = Instant::now();
            drop(guard);
            let work_duration = end_time.duration_since(start_time);
            if work_duration > time_per_frame {
                let overload = work_duration.sub(time_per_frame);
                warn!(?overload, "Can't keep up!");
            } else {
                let sleep_time = time_per_frame.sub(work_duration);
                sleep(sleep_time);
            }
            last_tick = start_time;
        }
    }
}
