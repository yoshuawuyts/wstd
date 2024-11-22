use super::task::SleepUntil;
use std::future::IntoFuture;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use wasi::clocks::monotonic_clock;

use super::Duration;

/// A measurement of a monotonically nondecreasing clock. Opaque and useful only
/// with Duration.
///
/// This type wraps `std::time::Duration` so we can implement traits on it
/// without coherence issues, just like if we were implementing this in the
/// stdlib.
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Copy)]
pub struct Instant(pub(crate) monotonic_clock::Instant);

impl Instant {
    /// Returns an instant corresponding to "now".
    ///
    /// # Examples
    ///
    /// ```
    /// use futures_time::time::Instant;
    ///
    /// let now = Instant::now();
    /// ```
    #[must_use]
    pub fn now() -> Self {
        Instant(wasi::clocks::monotonic_clock::now())
    }
}

impl Add<Duration> for Instant {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign<Duration> for Instant {
    fn add_assign(&mut self, rhs: Duration) {
        *self = Self(self.0 + rhs.0)
    }
}

impl Sub<Duration> for Instant {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = Self(self.0 - rhs.0)
    }
}

impl std::ops::Deref for Instant {
    type Target = monotonic_clock::Instant;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Instant {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoFuture for Instant {
    type Output = Instant;

    type IntoFuture = SleepUntil;

    fn into_future(self) -> Self::IntoFuture {
        super::task::sleep_until(self)
    }
}
