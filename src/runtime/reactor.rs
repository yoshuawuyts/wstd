use super::{
    polling::{EventKey, Poller},
    REACTOR,
};

use core::cell::RefCell;
use core::future::Future;
use core::task::{Context, Poll, Waker};
use std::collections::HashMap;
use std::pin::Pin;
use std::rc::Rc;
use wasi::io::poll::Pollable;

/// Manage async system resources for WASI 0.2
#[derive(Debug, Clone)]
pub struct Reactor {
    inner: Rc<RefCell<InnerReactor>>,
}

/// The private, internal `Reactor` implementation - factored out so we can take
/// a lock of the whole.
#[derive(Debug)]
struct InnerReactor {
    poller: Poller,
    wakers: HashMap<EventKey, Waker>,
}

impl Reactor {
    /// Return a `Reactor` for the currently running `wstd::runtime::block_on`.
    ///
    /// # Panic
    /// This will panic if called outside of `wstd::runtime::block_on`.
    pub fn current() -> Self {
        REACTOR.with(|r| {
            r.borrow()
                .as_ref()
                .expect("Reactor::current must be called within a wstd runtime")
                .clone()
        })
    }

    /// Create a new instance of `Reactor`
    pub(crate) fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(InnerReactor {
                poller: Poller::new(),
                wakers: HashMap::new(),
            })),
        }
    }

    /// Block until new events are ready. Calls the respective wakers once done.
    ///
    /// # On Wakers and single-threaded runtimes
    ///
    /// At first glance it might seem silly that this goes through the motions
    /// of calling the wakers. The main waker we create here is a `noop` waker:
    /// it does nothing. However, it is common and encouraged to use wakers to
    /// distinguish between events. Concurrency primitives may construct their
    /// own wakers to keep track of identity and wake more precisely. We do not
    /// control the wakers construted by other libraries, and it is for this
    /// reason that we have to call all the wakers - even if by default they
    /// will do nothing.
    pub(crate) fn block_until(&self) {
        let mut reactor = self.inner.borrow_mut();
        for key in reactor.poller.block_until() {
            match reactor.wakers.get(&key) {
                Some(waker) => waker.wake_by_ref(),
                None => panic!("tried to wake the waker for non-existent `{:?}`", key),
            }
        }
    }

    /// Wait for the pollable to resolve.
    pub async fn wait_for(&self, pollable: &Pollable) {
        WaitFor::new(self, pollable).await
    }
}

#[must_use = "futures do nothing unless polled or .awaited"]
struct WaitFor<'a> {
    reactor: &'a Reactor,
    key: Option<EventKey>,
    pollable: &'a Pollable,
}

impl<'a> WaitFor<'a> {
    fn new(reactor: &'a Reactor, pollable: &'a Pollable) -> Self {
        WaitFor {
            reactor,
            key: None,
            pollable,
        }
    }
}

impl<'a> Future for WaitFor<'a> {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let mut this = self.as_mut();
        // Start by taking a lock on the reactor. This is single-threaded
        // and short-lived, so it will never be contended.
        let mut reactor = this.reactor.inner.borrow_mut();

        // Schedule interest in the `pollable` on the first iteration. On
        // every iteration, register the waker with the reactor.
        // Safety: caller of insert operation must remove key during lifetime of &Pollable.
        if this.key.is_none() {
            this.key = Some(unsafe { reactor.poller.insert(&this.pollable) });
        }
        let key = this.key.as_ref().unwrap();
        reactor.wakers.insert(*key, cx.waker().clone());

        // Check whether we're ready or need to keep waiting. If we're
        // ready, we clean up after ourselves.
        if this.pollable.ready() {
            let key = this.key.take().unwrap();
            reactor.poller.remove(key);
            reactor.wakers.remove(&key);
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

impl<'a> Drop for WaitFor<'a> {
    fn drop(&mut self) {
        if let Some(key) = self.key {
            let mut reactor = self.reactor.inner.borrow_mut();
            reactor.poller.remove(key);
            reactor.wakers.remove(&key);
        }
    }
}
