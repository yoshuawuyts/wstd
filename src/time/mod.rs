//! Async time interfaces.

pub(crate) mod utils;

mod duration;
mod instant;
pub use duration::Duration;
pub use instant::Instant;

pub mod future;
pub mod stream;
pub mod task;

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use wasi::clocks::{
    monotonic_clock::{subscribe_duration, subscribe_instant},
    wall_clock,
};

use crate::{
    iter::AsyncIterator,
    runtime::{PollableFuture, Reactor},
};

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
        Some(Instant::now())
    }
}

#[derive(Debug)]
pub struct Timer(Option<PollableFuture>);

impl Timer {
    pub fn never() -> Timer {
        Timer(None)
    }
    pub fn at(deadline: Instant) -> Timer {
        Timer(Some(PollableFuture::new(subscribe_instant(*deadline))))
    }
    pub fn after(duration: Duration) -> Timer {
        Timer(Some(PollableFuture::new(subscribe_duration(*duration))))
    }
    pub fn set_after(&mut self, duration: Duration) {
        *self = Self::after(duration);
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
