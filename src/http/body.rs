use crate::io::AsyncRead;

/// An HTTP body
#[derive(Debug)]
pub struct Body {}

impl AsyncRead for Body {
    async fn read(&mut self, buf: &mut [u8]) -> crate::io::Result<usize> {
        todo!()
    }
}
