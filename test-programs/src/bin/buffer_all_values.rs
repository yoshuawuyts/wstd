use futures_lite::prelude::*;
use wstd::prelude::*;
use wstd::time::Duration;

#[wstd::main]
async fn main() {
    let interval = Duration::from_millis(5);
    let buffer = Duration::from_millis(20);

    let mut counter = 0;
    wstd::stream::interval(interval)
        .take(10)
        .buffer(buffer)
        .for_each(|buf| counter += buf.len())
        .await;

    assert_eq!(counter, 10);
}
