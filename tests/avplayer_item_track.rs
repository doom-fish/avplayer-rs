mod support;

use avplayer::prelude::*;

#[test]
fn avplayer_item_track_exposes_bridge_state() -> support::TestResult {
    let item = support::player_item("test-avplayer-item-track")?;
    let tracks = item.tracks()?;

    assert_eq!(item.track_count()?, tracks.len());
    if let Some(track) = tracks.into_iter().next() {
        let was_enabled = track.is_enabled()?;
        track.set_enabled(!was_enabled);
        assert_eq!(track.is_enabled()?, !was_enabled);
        track.set_enabled(was_enabled);
        let _ = track.video_field_mode()?;
        track.set_typed_video_field_mode(Some(
            &PlayerItemTrackVideoFieldMode::DeinterlaceFields,
        ))?;
        let _ = track.typed_video_field_mode()?;
        track.set_typed_video_field_mode(None)?;
        if let Some(asset_track) = track.asset_track()? {
            assert_eq!(asset_track.media_type()?, MediaType::Audio);
        }
    }
    Ok(())
}
