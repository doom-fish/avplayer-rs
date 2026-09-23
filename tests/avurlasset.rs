mod support;

use avplayer::prelude::*;

#[test]
fn avurlasset_supports_options_and_key_status_queries() -> support::TestResult {
    let path = support::audio_path("test-avurlasset")?;
    let options = UrlAssetOptions::new()
        .allows_cellular_access(false)
        .allows_expensive_network_access(false)
        .allows_constrained_network_access(false)
        .http_user_agent("avplayer-tests")
        .url_request_attribution(UrlRequestAttribution::User);
    let asset = UrlAsset::from_file_path_with_options(&path, &options)?;

    let statuses = asset.load_values_asynchronously(["duration", "tracks"])?;
    assert_eq!(statuses.len(), 2);
    assert_eq!(asset.status_of_value("duration")?, KeyValueStatus::Loaded);
    assert!(asset.url()?.ends_with("test-avurlasset.aiff"));
    Ok(())
}

#[test]
fn avurlasset_rejects_invalid_option_values() -> support::TestResult {
    let path = support::audio_path("test-avurlasset-invalid-options")?;
    let options = UrlAssetOptions::new().primary_session_identifier("not-a-uuid");
    assert!(matches!(
        UrlAsset::from_file_path_with_options(&path, &options),
        Err(AVPlayerError::AssetCreateFailed(_))
    ));
    let options = UrlAssetOptions::new()
        .primary_session_identifier("6F9619FF-8B86-D011-B42D-00C04FC964FF")
        .http_cookies([UrlAssetHttpCookie::new("session", "abc", "example.com", "/").secure(true)])
        .reference_restrictions(0);
    let asset = UrlAsset::from_file_path_with_options(&path, &options)?;
    assert!(asset
        .url()?
        .ends_with("test-avurlasset-invalid-options.aiff"));
    Ok(())
}
