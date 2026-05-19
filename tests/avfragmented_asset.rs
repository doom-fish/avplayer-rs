mod support;

use avplayer::prelude::*;

#[test]
fn avfragmented_asset_exposes_tracks_minder_and_extensions() -> support::TestResult {
    let path = support::audio_path("test-avfragmented-asset")?;
    let asset = FragmentedAsset::from_file_path(&path)?;
    asset
        .as_asset()
        .load_values_asynchronously(["tracks", "duration"])?;

    let tracks = asset.tracks()?;
    assert!(!tracks.is_empty());
    let track = &tracks[0];
    assert!(track.track_id()? > 0);
    assert!(track.segment_count()? >= 1);
    assert!(asset.track_with_id(track.track_id()?).is_some());
    assert!(!asset.url()?.unwrap_or_default().is_empty());
    let _ = asset
        .media_extension_properties()
        .map(|properties| properties.info())
        .transpose()?;
    let _ = FragmentedAsset::expects_property_revised_notifications();
    let _ = FragmentedAsset::is_playable_extended_mime_type("audio/aiff")?;

    assert!(!asset.is_associated_with_minder());
    let minder = FragmentedAssetMinder::new(&asset, 1.0)?;
    assert!(minder.minding_interval()? >= 0.0);
    assert!(minder.asset_count()? >= 1);
    assert_eq!(minder.assets()?.len(), minder.asset_count()?);
    assert!(asset.is_associated_with_minder());
    Ok(())
}
