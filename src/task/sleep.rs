use crate::time::{Duration, Timer, Wait};

/// Sleeps for the specified amount of time.
///
/// This future can be `push_deadline` to be moved
pub fn sleep(dur: Duration) -> Wait {
    Timer::after(dur).wait()
}
