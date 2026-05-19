mod support;

use avplayer::prelude::*;

#[test]
fn avasset_reader_output_exposes_random_access_surface() -> support::TestResult {
    let asset = support::loaded_audio_asset("test-avreader-extras")?;
    let track = support::first_audio_track(&asset)?;
    let reader = AssetReader::new(asset.as_asset())?;
    let output = AssetReaderTrackOutput::passthrough(&track)?;
    let borrowed = output.as_output();

    borrowed.set_always_copies_sample_data(false);
    borrowed.set_supports_random_access(true);
    assert!(borrowed.supports_random_access());
    assert_eq!(borrowed.media_type()?, MediaType::Audio);
    assert!(reader.can_add_track_output(&output));
    reader.add_track_output(&output)?;
    Ok(())
}
