//! Async IO abstractions.

mod copy;
mod read;
mod write;

pub use copy::*;
pub use read::*;
pub use write::*;

/// The error type for I/O operations.
///
pub use std::io::Error;

/// A specialized Result type for I/O operations.
///
pub use std::io::Result;
