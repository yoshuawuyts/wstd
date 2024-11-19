//! Async time interfaces.

use wasi::clocks::{
    monotonic_clock::{self, subscribe_duration, subscribe_instant},
    wall_clock,
};

use crate::{iter::AsyncIterator, runtime::Reactor};

/// A Duration type to represent a span of time, typically used for system
/// timeouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration(monotonic_clock::Duration);

impl Duration {
    pub fn from_nanos(nanos: u64) -> Self {
        Self(nanos)
    }
}

/// A measurement of a monotonically nondecreasing clock. Opaque and useful only
/// with `Duration`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant(monotonic_clock::Instant);

/// A measurement of the system clock, useful for talking to external entities
/// like the file system or other processes.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct SystemTime(wall_clock::Datetime);

impl SystemTime {
    pub fn now() -> Self {
        Self(wall_clock::now())
    }
}

/// An async iterator representing notifications at fixed interval.
pub fn interval(duration: Duration) -> Interval {
    Interval { duration }
}

/// An async iterator representing notifications at fixed interval.
///
/// See the [`interval`] function for more.
pub struct Interval {
    duration: Duration,
}
impl AsyncIterator for Interval {
    type Item = Instant;

    async fn next(&mut self) -> Option<Self::Item> {
        wait_for(self.duration).await;
        Some(Instant(wasi::clocks::monotonic_clock::now()))
    }
}

/// Wait until the passed duration has elapsed.
pub async fn wait_for(duration: Duration) {
    Reactor::current()
        .wait_for(subscribe_duration(duration.0))
        .await;
}

/// Wait until the passed instant.
pub async fn wait_until(deadline: Instant) {
    Reactor::current()
        .wait_for(subscribe_instant(deadline.0))
        .await;
}
