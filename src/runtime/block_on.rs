use super::Reactor;

use core::future::Future;
use core::pin::pin;
use core::ptr;
use core::task::Waker;
use core::task::{Context, Poll, RawWaker, RawWakerVTable};

/// Start the event loop
pub fn block_on<F, Fut>(f: F) -> Fut::Output
where
    F: FnOnce(Reactor) -> Fut,
    Fut: Future,
{
    // Construct the reactor
    let reactor = Reactor::new();

    // Create the future and pin it so it can be polled
    let fut = (f)(reactor.clone());
    let mut fut = pin!(fut);

    // Create a new context to be passed to the future.
    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);

    // Either the future completes and we return, or some IO is happening
    // and we wait.
    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(res) => return res,
            Poll::Pending => reactor.block_until(),
        }
    }
}

/// Construct a new no-op waker
// NOTE: we can remove this once <https://github.com/rust-lang/rust/issues/98286> lands
fn noop_waker() -> Waker {
    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        // Cloning just returns a new no-op raw waker
        |_| RAW,
        // `wake` does nothing
        |_| {},
        // `wake_by_ref` does nothing
        |_| {},
        // Dropping does nothing as we don't allocate anything
        |_| {},
    );
    const RAW: RawWaker = RawWaker::new(ptr::null(), &VTABLE);

    // SAFETY: all fields are no-ops, so this is safe
    unsafe { Waker::from_raw(RAW) }
}
