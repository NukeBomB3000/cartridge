//! The audio thread's clock while the device is let go: a period at a time, to a deadline, so the
//! ring drains at the device's rate and muted play keeps its frame rate. slot's (flo333).

use std::time::{Duration, Instant};

const RESYNC_AFTER: u32 = 4;

pub struct Pacer {
    span: Duration,
    next: Option<Instant>,
}

impl Pacer {
    pub fn new(span: Duration) -> Self {
        Pacer { span, next: None }
    }

    pub fn wait(&mut self, now: Instant) -> Duration {
        let from = self
            .next
            .filter(|&due| now < due + self.span * RESYNC_AFTER)
            .unwrap_or(now);
        let due = from + self.span;
        self.next = Some(due);
        due.saturating_duration_since(now)
    }

    pub fn reset(&mut self) {
        self.next = None;
    }
}
