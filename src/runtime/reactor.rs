use super::{
    polling::{EventKey, Poller},
    REACTOR,
};

use core::cell::RefCell;
use core::future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use std::collections::HashMap;
use std::rc::Rc;
use wasi::io::poll::Pollable;

#[derive(Debug)]
struct Registration {
    key: EventKey,
}

impl Drop for Registration {
    fn drop(&mut self) {
        Reactor::current().deregister_event(self.key)
    }
}

#[derive(Debug, Clone)]
pub struct AsyncPollable(Rc<Registration>);

impl AsyncPollable {
    pub fn wait_for(&self) -> WaitFor {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        let key = self.0.key;
        WaitFor {
            waitee: Waitee { key, unique },
            needs_deregistration: false,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct Waitee {
    key: EventKey,
    unique: usize,
}

#[must_use = "futures do nothing unless polled or .awaited"]
#[derive(Debug)]
pub struct WaitFor {
    waitee: Waitee,
    needs_deregistration: bool,
}
impl future::Future for WaitFor {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let reactor = Reactor::current();
        if reactor.ready(&self.as_ref().waitee, cx.waker()) {
            Poll::Ready(())
        } else {
            self.as_mut().needs_deregistration = true;
            Poll::Pending
        }
    }
}
impl Drop for WaitFor {
    fn drop(&mut self) {
        println!("dropping {:?}", self);
        if self.needs_deregistration {
            Reactor::current().deregister_waitee(&self.waitee)
        }
    }
}

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
    wakers: HashMap<Waitee, Waker>,
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
            for (waitee, waker) in reactor.wakers.iter() {
                if waitee.key == key {
                    waker.wake_by_ref()
                }
            }
        }
    }

    /// Turn a wasi [`Pollable`] into an [`AsyncPollable`]
    pub fn schedule(&self, pollable: Pollable) -> AsyncPollable {
        let mut reactor = self.inner.borrow_mut();
        let key = reactor.poller.insert(pollable);
        println!("schedule pollable as {key:?}");
        AsyncPollable(Rc::new(Registration { key }))
    }

    fn deregister_event(&self, key: EventKey) {
        let mut reactor = self.inner.borrow_mut();
        println!("deregister {key:?}",);
        reactor.poller.remove(key);
    }

    fn deregister_waitee(&self, waitee: &Waitee) {
        let mut reactor = self.inner.borrow_mut();
        println!("deregister waker for {waitee:?}",);
        reactor.wakers.remove(waitee);
    }

    fn ready(&self, waitee: &Waitee, waker: &Waker) -> bool {
        let mut reactor = self.inner.borrow_mut();
        let ready = reactor
            .poller
            .get(&waitee.key)
            .expect("only live EventKey can be checked for readiness")
            .ready();
        if !ready {
            println!("register waker for {waitee:?}");
            reactor.wakers.insert(waitee.clone(), waker.clone());
        }
        println!("ready {ready} {waitee:?}");
        ready
    }

    /// Wait for the pollable to resolve.
    pub async fn wait_for(&self, pollable: Pollable) {
        let p = self.schedule(pollable);
        p.wait_for().await
    }
}
