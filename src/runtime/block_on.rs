use super::{Reactor, REACTOR};

use core::future::Future;
use core::pin::pin;
use core::task::Waker;
use core::task::{Context, Poll};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::task::Wake;

/// Start the event loop
pub fn block_on<Fut>(fut: Fut) -> Fut::Output
where
    Fut: Future,
{
    // Construct the reactor
    let reactor = Reactor::new();
    // Store a copy as a singleton to be used elsewhere:
    let prev = REACTOR.replace(Some(reactor.clone()));
    if prev.is_some() {
        panic!("cannot wstd::runtime::block_on inside an existing block_on!")
    }

    // Pin the future so it can be polled
    let mut fut = pin!(fut);

    // Create a new context to be passed to the future.
    let waker_impl = Arc::new(ReactorWaker::new());
    let waker = Waker::from(Arc::clone(&waker_impl));
    let mut cx = Context::from_waker(&waker);

    // Either the future completes and we return, or some IO is happening
    // and we wait.
    let res = loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(res) => break res,
            Poll::Pending => {
                reactor.block_until(waker_impl.awake());
                waker_impl.set_awake(false);
            }
        }
    };
    // Clear the singleton
    REACTOR.replace(None);
    res
}

struct ReactorWaker {
    awake: AtomicBool,
}

impl ReactorWaker {
    fn new() -> Self {
        Self {
            awake: AtomicBool::new(false),
        }
    }

    #[inline]
    fn set_awake(&self, awake: bool) {
        self.awake.store(awake, Ordering::Relaxed);
    }

    #[inline]
    fn awake(&self) -> bool {
        self.awake.load(Ordering::Relaxed)
    }
}

impl Wake for ReactorWaker {
    fn wake(self: Arc<Self>) {
        self.set_awake(true);
    }
}
