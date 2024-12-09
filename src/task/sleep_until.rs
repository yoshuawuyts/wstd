use crate::time::{Instant, Timer, Wait};

/// Sleeps until the specified instant.
pub fn sleep_until(deadline: Instant) -> Wait {
    Timer::at(deadline).wait()
}
