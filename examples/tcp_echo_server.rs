use wstd::io;
use wstd::iter::AsyncIterator;
use wstd::net::TcpListener;
use wstd::runtime::block_on;

fn main() -> io::Result<()> {
    block_on(async move {
        let listener = TcpListener::bind("127.0.0.1:8080").await?;
        println!("Listening on {}", listener.local_addr()?);
        println!("type `nc localhost 8080` to create a TCP client");

        let mut incoming = listener.incoming();
        while let Some(stream) = incoming.next().await {
            let stream = stream?;
            println!("Accepted from: {}", stream.peer_addr()?);
            io::copy(&stream, &stream).await?;
        }
        Ok(())
    })
}
