use std::time::Instant;

#[derive(Default)]
pub(crate) struct Ticks(pub u64);

impl Ticks {
    pub(crate) fn increase(&mut self) {
        self.0 += 1;
    }
}

pub(crate) struct CurrentTime(pub Instant);

impl Default for CurrentTime {
    fn default() -> Self {
        CurrentTime(Instant::now())
    }
}

#[derive(Default)]
pub(crate) struct Delta(pub f64);
