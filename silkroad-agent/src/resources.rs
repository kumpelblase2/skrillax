#[derive(Default)]
pub(crate) struct Ticks(pub u64);

impl Ticks {
    pub(crate) fn increase(&mut self) {
        self.0 += 1;
    }
}
