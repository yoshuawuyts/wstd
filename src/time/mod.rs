//! Async time interfaces.
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use wasi::clocks::{
    monotonic_clock::{self, subscribe_duration, subscribe_instant},
    wall_clock,
};

use crate::{
    iter::AsyncIterator,
    runtime::{PollableFuture, Reactor},
};

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
        Timer::after(self.duration).await;
        Some(Instant(wasi::clocks::monotonic_clock::now()))
    }
}

#[derive(Debug)]
pub struct Timer(Option<PollableFuture>);

impl Timer {
    pub fn never() -> Timer {
        Timer(None)
    }
    pub fn at(deadline: Instant) -> Timer {
        Timer(Some(PollableFuture::new(subscribe_instant(deadline.0))))
    }
    pub fn after(duration: Duration) -> Timer {
        Timer(Some(PollableFuture::new(subscribe_duration(duration.0))))
    }
}

impl Future for Timer {
    type Output = Instant;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match &mut self.as_mut().0 {
            None => Poll::Pending,
            Some(pollable) => match pollable.poll(&Reactor::current(), cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(()) => Poll::Ready(Instant::now()),
            },
        }
    }
}
