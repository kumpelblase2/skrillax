use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;
use tracing::trace;

struct Capacity {
    max: u16,
    admitted: AtomicU16,
}

pub struct QueueToken {
    inner: Arc<Capacity>,
}

impl Drop for QueueToken {
    fn drop(&mut self) {
        trace!("Queue token expired");
        self.inner.admitted.fetch_sub(1, Ordering::AcqRel);
    }
}

pub struct PlayingToken {
    inner: Arc<Capacity>,
}

impl Drop for PlayingToken {
    fn drop(&mut self) {
        trace!("Play token expired");
        self.inner.admitted.fetch_sub(1, Ordering::AcqRel);
    }
}

impl Capacity {
    fn new(capacity: u16) -> Self {
        Capacity {
            max: capacity,
            admitted: AtomicU16::default(),
        }
    }

    fn current_total(&self) -> u16 {
        self.admitted.load(Ordering::Acquire)
    }

    fn usage(&self) -> f32 {
        if self.max == 0 {
            return 0.0;
        }
        self.current_total() as f32 / self.max as f32
    }

    fn try_admit(&self) -> bool {
        self.admitted
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                (current < self.max).then_some(current + 1)
            })
            .is_ok()
    }
}

#[derive(Clone)]
pub struct CapacityController {
    inner: Arc<Capacity>,
}

impl CapacityController {
    pub fn new(capacity: u16) -> Self {
        CapacityController {
            inner: Arc::new(Capacity::new(capacity)),
        }
    }

    pub fn usage(&self) -> f32 {
        self.inner.usage()
    }

    pub fn add_queue(&self) -> Option<QueueToken> {
        self.inner.try_admit().then(|| QueueToken {
            inner: Arc::clone(&self.inner),
        })
    }

    pub fn start_playing(&self, queue_token: QueueToken) -> PlayingToken {
        let inner = Arc::clone(&queue_token.inner);
        std::mem::forget(queue_token);
        PlayingToken { inner }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use std::thread;

    #[test]
    fn concurrent_admission_never_exceeds_capacity() {
        const MAX: usize = 4;
        let controller = CapacityController::new(MAX as u16);
        let barrier = Arc::new(Barrier::new(64));
        let handles: Vec<_> = (0..64)
            .map(|_| {
                let controller = controller.clone();
                let barrier = barrier.clone();
                thread::spawn(move || {
                    barrier.wait();
                    controller.add_queue()
                })
            })
            .collect();
        let tokens: Vec<_> = handles.into_iter().filter_map(|h| h.join().unwrap()).collect();
        assert_eq!(tokens.len(), MAX);
        assert_eq!(controller.usage(), 1.0);
        drop(tokens);
        assert!(controller.add_queue().is_some());
    }

    #[test]
    fn transferring_queue_token_does_not_change_usage() {
        let controller = CapacityController::new(1);
        let queued = controller.add_queue().unwrap();
        let playing = controller.start_playing(queued);
        assert_eq!(controller.usage(), 1.0);
        assert!(controller.add_queue().is_none());
        drop(playing);
        assert!(controller.add_queue().is_some());
    }
}
