use futures_lite::prelude::*;
use wstd::prelude::*;
use wstd::time::Duration;

#[wstd::main]
async fn main() {
    let interval = Duration::from_millis(10);
    let debounce = Duration::from_millis(20);

    let mut counter = 0;
    wstd::stream::interval(interval)
        .take(10)
        .debounce(debounce)
        .for_each(|_| counter += 1)
        .await;

    assert_eq!(counter, 1);
}
