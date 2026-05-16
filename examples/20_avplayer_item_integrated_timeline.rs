#[path = "../tests/support/mod.rs"]
mod support;

use avplayer::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let item = support::player_item("example-avplayer-item-integrated-timeline")?;
    let _player = Player::from_item(&item)?;
    let timeline = item.integrated_timeline()?;
    let snapshot = timeline.current_snapshot()?;
    let snapshot_info = snapshot.info()?;

    let _periodic = timeline.observe_periodic_times(Time::new(1, 10), |time| {
        println!("periodic integrated time: {time:?}");
    })?;
    let _out_of_sync = timeline.observe_snapshots_out_of_sync(|event| {
        println!("timeline out-of-sync: {event:?}");
    })?;

    println!("timeline info: {:?}", timeline.info()?);
    println!("snapshot info: {snapshot_info:?}");
    println!(
        "seek-to-zero result: {:?}",
        timeline.seek_to_time(Time::new(0, 1), Time::new(0, 1), Time::new(0, 1))
    );
    println!(
        "out-of-sync notification: {}",
        player_integrated_timeline_snapshots_out_of_sync_notification()?
    );
    if let Some(segment) = snapshot.segment_at_index(0) {
        println!("first segment: {:?}", segment.info()?);
    }
    Ok(())
}
