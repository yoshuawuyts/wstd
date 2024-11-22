//! Async time interfaces.
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use wasi::clocks::{monotonic_clock::subscribe_instant, wall_clock};
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

impl From<std::time::Duration> for Duration {
    fn from(std_dur: std::time::Duration) -> Duration {
        Self::from_nanos(std_dur.as_nanos().try_into().unwrap_or(u64::MAX))
    }
}

impl From<Duration> for std::time::Duration {
    fn from(dur: Duration) -> std::time::Duration {
        std::time::Duration::from_nanos(dur.0)
    }
}

/// A measurement of a monotonically nondecreasing clock. Opaque and useful only
/// with `Duration`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant(monotonic_clock::Instant);

impl Instant {
    pub fn now() -> Self {
        Self(monotonic_clock::now())
    }
}

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
        Timer::after(self.duration).wait().await;
        Some(Instant::now())
    }
}

#[derive(Debug)]
pub struct Timer(Option<Instant>);

impl Timer {
    pub fn never() -> Timer {
        Timer(None)
    }
    pub fn at(deadline: Instant) -> Timer {
        Timer(Some(deadline))
    }
    pub fn after(duration: Duration) -> Timer {
        Timer(Some(Instant::now() + duration))
    }
    pub fn set_after(&mut self, duration: Duration) {
        *self = Self::after(duration);
    }
    pub async fn wait(&self) {
        match self.0 {
            Some(deadline) => {
                Reactor::current()
                    .wait_for(subscribe_instant(*deadline))
                    .await
            }
            None => std::future::pending().await,
        }
    }
}

impl Future for Timer {
    type Output = Instant;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_ref();
        let pinned = std::pin::pin!(this.wait());
        match pinned.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(()) => Poll::Ready(Instant::now()),
        }
    }
}
