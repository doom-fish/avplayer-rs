#[path = "../tests/support/mod.rs"]
mod support;

use avplayer::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let item = support::player_item("example-avplayer-item-track")?;
    let tracks = item.tracks()?;

    println!("player-item track count: {}", item.track_count()?);
    if let Some(track) = tracks.into_iter().next() {
        let enabled = track.is_enabled()?;
        track.set_enabled(!enabled);
        track.set_enabled(enabled);
        track.set_typed_video_field_mode(Some(
            &PlayerItemTrackVideoFieldMode::DeinterlaceFields,
        ))?;
        println!("track enabled: {}", track.is_enabled()?);
        println!(
            "current video frame rate: {}",
            track.current_video_frame_rate()?
        );
        println!("video field mode: {:?}", track.video_field_mode()?);
        println!("typed video field mode: {:?}", track.typed_video_field_mode()?);
        println!("asset track present: {}", track.asset_track()?.is_some());
    } else {
        println!("AVFoundation has not materialized any AVPlayerItemTrack instances yet");
    }
    Ok(())
}
