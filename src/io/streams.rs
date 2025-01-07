use super::{AsyncPollable, AsyncRead, AsyncWrite};
use std::cell::RefCell;
use std::io::Result;
use wasi::io::streams::{InputStream, OutputStream, StreamError};

#[derive(Debug)]
pub struct AsyncInputStream {
    // Lazily initialized pollable, used for lifetime of stream to check readiness.
    // Field ordering matters: this child must be dropped before stream
    subscription: RefCell<Option<AsyncPollable>>,
    stream: InputStream,
}

impl AsyncInputStream {
    pub fn new(stream: InputStream) -> Self {
        Self {
            subscription: RefCell::new(None),
            stream,
        }
    }
    async fn ready(&self) {
        // Lazily initialize the AsyncPollable
        if self.subscription.borrow().is_none() {
            self.subscription
                .replace(Some(AsyncPollable::new(self.stream.subscribe())));
        }
        // Wait on readiness
        self.subscription
            .borrow()
            .as_ref()
            .expect("populated refcell")
            .wait_for()
            .await;
    }
    /// Like [`AsyncRead::read`], but doesn't require a `&mut self`.
    pub async fn read(&self, buf: &mut [u8]) -> Result<usize> {
        self.ready().await;
        // Ideally, the ABI would be able to read directly into buf. However, with the default
        // generated bindings, it returns a newly allocated vec, which we need to copy into buf.
        let read = match self.stream.read(buf.len() as u64) {
            // We don't need to special-case 0 here: a value of 0 bytes from
            // WASI's `read` doesn't mean end-of-stream as it does in Rust,
            // however since we called `self.ready()`, we'll always get at
            // least one byte.
            Ok(r) => r,
            // 0 bytes from Rust's `read` means end-of-stream.
            Err(StreamError::Closed) => return Ok(0),
            Err(StreamError::LastOperationFailed(err)) => {
                return Err(std::io::Error::other(err.to_debug_string()))
            }
        };
        let len = read.len();
        buf[0..len].copy_from_slice(&read);
        Ok(len)
    }
}

impl AsyncRead for AsyncInputStream {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        Self::read(self, buf).await
    }

    #[inline]
    fn as_async_input_stream(&self) -> Option<&AsyncInputStream> {
        Some(self)
    }
}

#[derive(Debug)]
pub struct AsyncOutputStream {
    // Lazily initialized pollable, used for lifetime of stream to check readiness.
    // Field ordering matters: this child must be dropped before stream
    subscription: RefCell<Option<AsyncPollable>>,
    stream: OutputStream,
}

impl AsyncOutputStream {
    pub fn new(stream: OutputStream) -> Self {
        Self {
            subscription: RefCell::new(None),
            stream,
        }
    }
    async fn ready(&self) {
        // Lazily initialize the AsyncPollable
        if self.subscription.borrow().is_none() {
            self.subscription
                .replace(Some(AsyncPollable::new(self.stream.subscribe())));
        }
        // Wait on readiness
        self.subscription
            .borrow()
            .as_ref()
            .expect("populated refcell")
            .wait_for()
            .await;
    }
    /// Like [`AsyncWrite::write`], but doesn't require a `&mut self`.
    pub async fn write(&self, buf: &[u8]) -> Result<usize> {
        // Loops at most twice.
        loop {
            match self.stream.check_write() {
                Ok(0) => {
                    self.ready().await;
                    // Next loop guaranteed to have nonzero check_write, or error.
                    continue;
                }
                Ok(some) => {
                    let writable = some.try_into().unwrap_or(usize::MAX).min(buf.len());
                    match self.stream.write(&buf[0..writable]) {
                        Ok(()) => return Ok(writable),
                        Err(StreamError::Closed) => {
                            return Err(std::io::Error::from(std::io::ErrorKind::ConnectionReset))
                        }
                        Err(StreamError::LastOperationFailed(err)) => {
                            return Err(std::io::Error::other(err.to_debug_string()))
                        }
                    }
                }
                Err(StreamError::Closed) => {
                    return Err(std::io::Error::from(std::io::ErrorKind::ConnectionReset))
                }
                Err(StreamError::LastOperationFailed(err)) => {
                    return Err(std::io::Error::other(err.to_debug_string()))
                }
            }
        }
    }
    /// Like [`AsyncWrite::flush`], but doesn't require a `&mut self`.
    pub async fn flush(&self) -> Result<()> {
        match self.stream.flush() {
            Ok(()) => {
                self.ready().await;
                Ok(())
            }
            Err(StreamError::Closed) => {
                Err(std::io::Error::from(std::io::ErrorKind::ConnectionReset))
            }
            Err(StreamError::LastOperationFailed(err)) => {
                Err(std::io::Error::other(err.to_debug_string()))
            }
        }
    }
}
impl AsyncWrite for AsyncOutputStream {
    // Required methods
    async fn write(&mut self, buf: &[u8]) -> Result<usize> {
        Self::write(self, buf).await
    }
    async fn flush(&mut self) -> Result<()> {
        Self::flush(self).await
    }

    #[inline]
    fn as_async_output_stream(&self) -> Option<&AsyncOutputStream> {
        Some(self)
    }
}

/// Wait for both streams to be ready and then do a WASI splice.
pub(crate) async fn splice(
    reader: &AsyncInputStream,
    writer: &AsyncOutputStream,
    len: u64,
) -> core::result::Result<u64, StreamError> {
    // Wait for both streams to be ready.
    let r = reader.ready();
    writer.ready().await;
    r.await;

    writer.stream.splice(&reader.stream, len)
}
