
use futures_lite::prelude::*;
use wstd::prelude::*;
use wstd::time::Duration;

#[wstd::main]
async fn main() {
    let interval = Duration::from_millis(100);
    let throttle = Duration::from_millis(300);

    let take = 4;
    let expected = 2;

    let mut counter = 0;
    wstd::stream::interval(interval)
        .take(take)
        .throttle(throttle)
        .for_each(|_| counter += 1)
        .await;

    assert_eq!(counter, expected);
}
