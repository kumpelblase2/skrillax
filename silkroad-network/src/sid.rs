use std::fmt::{Debug, Formatter};
use std::sync::atomic::{AtomicU64, Ordering};

/// An identifier for a given connection between the server and the client.
/// It is simply a - temporary - unique identifier to reference a target
/// stream for a given packet or where a given packet may have come from.
///
/// It does not attempt to be globally unique nor to be unique for any
/// period of time that may approach infinity. It should be "unique enough"
/// for the length of any connection.
///
/// Internally this uses a `u64` which is increased monotonically for each
/// new stream. This ID should not be exposed externally due to its
/// predictive nature. Assuming there are 1 million new streams every
/// second, it would still take thousands of years to wrap around again
/// and result in a duplicate stream id, which is good enough for us.
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
