use crate::io::{AsyncRead, AsyncWrite};

/// Copy bytes from a reader to a writer.
pub async fn copy<R, W>(mut reader: R, mut writer: W) -> crate::io::Result<()>
where
    R: AsyncRead,
    W: AsyncWrite,
{
    let mut buf = vec![0; 1024];
    'read: loop {
        let bytes_read = reader.read(&mut buf).await?;
        if bytes_read == 0 {
            break 'read Ok(());
        }
        let mut slice = &buf[0..bytes_read];

        'write: loop {
            let bytes_written = writer.write(slice).await?;
            slice = &slice[bytes_written..];
            if slice.is_empty() {
                break 'write;
            }
        }
    }
}
