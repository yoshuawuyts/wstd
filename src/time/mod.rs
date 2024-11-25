//! Async time interfaces.

pub(crate) mod utils;

mod duration;
mod instant;
pub use duration::Duration;
pub use instant::Instant;

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use wasi::clocks::{monotonic_clock::subscribe_instant, wall_clock};

use crate::{iter::AsyncIterator, runtime::Reactor};

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
#[derive(Debug)]
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
                    .wait_for(&subscribe_instant(*deadline))
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
