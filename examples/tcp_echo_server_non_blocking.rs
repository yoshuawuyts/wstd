use futures_concurrency::future::FutureGroup;
use futures_lite::{FutureExt, StreamExt};
use std::{
    cell::RefCell,
    future::Future,
    pin::{pin, Pin},
    rc::Rc,
    task::Poll,
};
use wstd::io;
use wstd::iter::AsyncIterator;
use wstd::net::TcpListener;

type StreamTasks = Rc<RefCell<FutureGroup<Pin<Box<dyn Future<Output = io::Result<()>>>>>>>;

#[wstd::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Listening on {}", listener.local_addr()?);
    println!("type `nc localhost 8080` to create a TCP client");

    let stream_tasks: StreamTasks = StreamTasks::default();
    let mut listening_task = pin!(start_listening(listener, stream_tasks.clone()));

    futures_lite::future::poll_fn(|cx| {
        if let Poll::Ready(_) = listening_task.as_mut().poll(cx) {
            return Poll::Ready(());
        };

        let mut stream_tasks_ref = stream_tasks.borrow_mut();
        if let Poll::Ready(Some(res)) = stream_tasks_ref.poll_next(cx) {
            println!("Task finished: {:?}", res);
            println!("Tasks len: {}", stream_tasks_ref.len());
        }

        Poll::Pending
    })
    .await;
    Ok(())
}

async fn start_listening(listener: TcpListener, stream_tasks: StreamTasks) -> io::Result<()> {
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        println!("Accepted from: {}", stream.peer_addr()?);

        let stream_task = async move { io::copy(&stream, &stream).await }.boxed_local();

        stream_tasks.borrow_mut().insert(stream_task);
        println!("Task added");
        println!("Tasks len: {}", stream_tasks.borrow().len());
    }
    Ok(())
}
