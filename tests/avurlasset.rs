mod support;

use avplayer::prelude::*;

#[test]
fn avurlasset_supports_options_and_key_status_queries() -> support::TestResult {
    let path = support::audio_path("test-avurlasset")?;
    let options = UrlAssetOptions::new().prefer_precise_duration_and_timing(false);
    let asset = UrlAsset::from_file_path_with_options(&path, options)?;

    let statuses = asset.load_values_asynchronously(["duration", "tracks"])?;
    assert_eq!(statuses.len(), 2);
    assert_eq!(asset.status_of_value("duration")?, KeyValueStatus::Loaded);
    assert!(asset.url()?.ends_with("test-avurlasset.aiff"));
    Ok(())
}
