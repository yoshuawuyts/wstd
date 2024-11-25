use std::error::Error;
use wstd::task::sleep;
use wstd::time::Duration;

#[wstd::test]
async fn just_sleep() -> Result<(), Box<dyn Error>> {
    sleep(Duration::from_secs(1)).await;
    Ok(())
}
