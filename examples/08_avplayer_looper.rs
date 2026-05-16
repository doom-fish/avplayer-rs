#[path = "../tests/support/mod.rs"]
mod support;

use avplayer::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let queue = QueuePlayer::new()?;
    let template = support::player_item("example-avplayer-looper")?;
    let looper = PlayerLooper::with_time_range_and_ordering(
        &queue,
        &template,
        Some(TimeRange::new(Time::new(0, 1), Time::new(1, 1))),
        PlayerLooperItemOrdering::LoopingItemsFollowExistingItems,
    )?;

    println!("status: {:?}", looper.status()?);
    println!("error: {:?}", looper.error()?);
    println!("loop count: {}", looper.loop_count()?);
    println!("looping items: {}", looper.looping_items()?.len());
    looper.disable_looping();
    Ok(())
}
