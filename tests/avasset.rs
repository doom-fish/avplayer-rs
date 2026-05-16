mod support;

use avplayer::prelude::*;

#[test]
fn avasset_exposes_duration_tracks_and_metadata() -> support::TestResult {
    let asset = support::loaded_audio_asset("test-avasset")?;
    let track = support::first_audio_track(&asset)?;

    assert!(asset.duration()?.as_numeric().is_some());
    assert!(asset.url()?.ends_with("test-avasset.aiff"));
    assert_eq!(track.media_type()?, MediaType::Audio);
    assert!(track.track_id()? > 0);
    assert!(!asset.metadata()?.is_empty());
    Ok(())
}
