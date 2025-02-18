use crate::io::{AsyncRead, AsyncWrite, Error};
use wasi::io::streams::StreamError;

/// Copy bytes from a reader to a writer.
pub async fn copy<R, W>(mut reader: R, mut writer: W) -> crate::io::Result<()>
where
    R: AsyncRead,
    W: AsyncWrite,
{
    // Optimized path when we have an `AsyncInputStream` and an
    // `AsyncOutputStream`.
    if let Some(reader) = reader.as_async_input_stream() {
        if let Some(writer) = writer.as_async_output_stream() {
            loop {
                match super::splice(reader, writer, u64::MAX).await {
                    Ok(_n) => (),
                    Err(StreamError::Closed) => return Ok(()),
                    Err(StreamError::LastOperationFailed(err)) => {
                        return Err(Error::other(err.to_debug_string()));
                    }
                }
            }
        }
    }

    // Unoptimized case: read the input and then write it.
    let mut buf = [0; 1024];
    'read: loop {
        let bytes_read = reader.read(&mut buf).await?;
        if bytes_read == 0 {
            break 'read Ok(());
        }
        writer.write_all(&buf[0..bytes_read]).await?;
    }
}
