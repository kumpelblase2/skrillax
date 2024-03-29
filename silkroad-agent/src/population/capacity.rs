use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;
use tracing::trace;

struct Capacity {
    max: u16,
    queued: AtomicU16,
    playing: AtomicU16,
}

pub struct QueueToken {
    inner: Arc<Capacity>,
}

impl Drop for QueueToken {
    fn drop(&mut self) {
        trace!("Queue token expired");
        self.inner.queued.fetch_sub(1, Ordering::Relaxed);
    }
}

pub struct PlayingToken {
    inner: Arc<Capacity>,
}

impl Drop for PlayingToken {
    fn drop(&mut self) {
        trace!("Play token expired");
        self.inner.playing.fetch_sub(1, Ordering::Relaxed);
    }
}

impl Capacity {
    fn new(capacity: u16) -> Self {
        Capacity {
            max: capacity,
            queued: AtomicU16::default(),
            playing: AtomicU16::default(),
        }
    }

    fn current_total(&self) -> u16 {
        self.queued.load(Ordering::Acquire) + self.playing.load(Ordering::Acquire)
    }

    fn available(&self) -> u16 {
        self.max - self.current_total()
    }

    fn usage(&self) -> f32 {
        let total_current = self.current_total() as f32;
        let maximum = self.max as f32;
        total_current / maximum
    }

    fn can_queue(&self) -> bool {
        self.available() > 0
    }
}

#[derive(Clone)]
pub struct CapacityController {
    inner: Arc<Capacity>,
}

impl CapacityController {
    pub fn new(capacity: u16) -> Self {
        let capacity = Capacity::new(capacity);
        CapacityController {
            inner: Arc::new(capacity),
        }
    }

    pub fn usage(&self) -> f32 {
        self.inner.usage()
    }

    pub fn add_queue(&self) -> Option<QueueToken> {
        if !self.inner.can_queue() {
            return None;
        }

        self.inner.queued.fetch_add(1, Ordering::Relaxed);
        Some(QueueToken {
            inner: Arc::clone(&self.inner),
        })
    }

    pub fn add_playing(&self) -> PlayingToken {
        self.inner.playing.fetch_add(1, Ordering::Relaxed);
        PlayingToken {
            inner: Arc::clone(&self.inner),
        }
    }
}
