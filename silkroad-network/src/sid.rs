use std::fmt::{Debug, Formatter};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct StreamId(u64);

static ID_POOL: AtomicU64 = AtomicU64::new(0);

impl StreamId {
    pub fn new() -> Self {
        let new_id = ID_POOL.fetch_add(1, Ordering::Relaxed);
        StreamId(new_id)
    }
}

impl Default for StreamId {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for StreamId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
