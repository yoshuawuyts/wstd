#![allow(async_fn_in_trait)]

//! An async standard library for Wasm Components and WASI 0.2
//!
//! This is a minimal async standard library written exclusively to support Wasm
//! Components. It exists primarily to enable people to write async-based
//! applications in Rust before async-std, smol, or tokio land support for Wasm
//! Components and WASI 0.2. Once those runtimes land support, it is recommended
//! users switch to use those instead.
//!
//! # Examples
//!
//! **TCP echo server**
//!
//! ```rust
//! use wstd::io;
//! use wstd::iter::AsyncIterator;
//! use wstd::net::TcpListener;
//! use wstd::runtime::block_on;
//!
//! fn main() -> io::Result<()> {
//!     block_on(|reactor| async move {
//!         let listener = TcpListener::bind(&reactor, "127.0.0.1:8080").await?;
//!         println!("Listening on {}", listener.local_addr()?);
//!         println!("type `nc localhost 8080` to create a TCP client");
//!
//!         let mut incoming = listener.incoming();
//!         while let Some(stream) = incoming.next().await {
//!             let stream = stream?;
//!             println!("Accepted from: {}", stream.peer_addr()?);
//!             io::copy(&stream, &stream).await?;
//!         }
//!         Ok(())
//!     })
//! }
//! ```
//!
//! # Design Decisions
//!
//! This library is entirely self-contained. This means that it does not share
//! any traits or types with any other async runtimes. This means we're trading
//! in some compatibility for ease of maintenance. Because this library is not
//! intended to be maintained in the long term, this seems like the right
//! tradeoff to make.
//!
//! WASI 0.2 does not yet support multi-threading. For that reason this library
//! does not provide any multi-threaded primitives, and is free to make liberal
//! use of Async Functions in Traits since no `Send` bounds are required. This
//! makes for a simpler end-user experience, again at the cost of some
//! compatibility. Though ultimately we do believe that using Async Functions is
//! the right foundation for the standard library abstractions - meaning we may
//! be trading in backward-compatibility for forward-compatibility.
//!
//! This library also supports slightly more interfaces than the stdlib does.
//! For example `wstd::rand` is a new module that provides access to random
//! bytes. And `wstd::runtime` provides access to async runtime primitives.
//! These are unique capabilities provided by WASI 0.2, and because this library
//! is specific to that are exposed from here.
//!
//! Finally, this library does not implicitly thread through a
//! [`Reactor`][runtime::Reactor] handle. Rather than using a `thread_local!`
//! async resource APIs in `wstd` will borrow an instance of `Reactor`. This is
//! a little more verbose, but in turn is a little simpler to implement,
//! maintain, and extend.

pub mod http;
pub mod io;
pub mod iter;
pub mod net;
pub mod rand;
pub mod runtime;
pub mod time;
