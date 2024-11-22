//! Asynchronous values.
//!
//! # Cancellation
//!
//! Futures can be cancelled by dropping them before they finish executing. This
//! is useful when we're no longer interested in the result of an operation, as
//! it allows us to stop doing needless work. This also means that a future may cancel at any `.await` point, and so just
//! like with `?` we have to be careful to roll back local state if our future
//! halts there.
//!
//! In order to perform a cancellation remotely, you can use the [`channel::bounded`]
//! function to create a sender/receiver pair. When the sender side of
//! this pair emits a message, all receivers are trigered. Receivers can be passed to
//! [`Future::timeout`] or [`Stream::timeout`] to perform a cancellation when
//! the message is received.
//!
//! [`channel::bounded`]: crate::channel::bounded
//! [`Future::timeout`]: crate::future::FutureExt::timeout
//! [`Stream::timeout`]: crate::stream::StreamExt::timeout
//!
//!
//! ```no_run
//! use futures_lite::prelude::*;
//! use wstd::prelude::*;
//! use wstd::channel;
//! use wstd::time::Duration;
//!
//! #[wstd::main]
//! async fn main() {
//!     let (send, mut recv) = channel::bounded::<()>(1); // create a new send/receive pair
//!     let mut counter = 0;
//!     let value = async { "meow" }
//!         .delay(Duration::from_millis(100))
//!         .timeout(recv.next()) // time-out if the sender is dropped.
//!         .await;
//!
//!     assert_eq!(value.unwrap(), "meow");
//! }
//! ```

mod delay;
mod future_ext;
mod park;
mod relative_future;
mod timeout;

pub use delay::Delay;
pub use future_ext::FutureExt;
pub use park::Park;
pub use relative_future::Timer;
pub use std::future::IntoFuture;
pub use timeout::Timeout;
