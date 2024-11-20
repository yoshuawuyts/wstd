use crate::io::{AsyncRead, AsyncWrite};

/// Copy bytes from a reader to a writer.
pub async fn copy<R, W>(mut reader: R, mut writer: W) -> crate::io::Result<()>
where
    R: AsyncRead,
    W: AsyncWrite,
{
    let mut buf = [0; 1024];
    'read: loop {
        let bytes_read = reader.read(&mut buf).await?;
        if bytes_read == 0 {
            break 'read Ok(());
        }
        writer.write_all(&buf[0..bytes_read]).await?;
    }
}
