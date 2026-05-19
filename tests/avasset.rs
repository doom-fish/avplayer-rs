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

#[test]
fn avasset_exposes_extended_asset_and_track_surface() -> support::TestResult {
    let asset = support::loaded_audio_asset("test-avasset-extras")?;
    let track = support::first_audio_track(&asset)?;

    assert!(asset.preferred_rate()? >= 0.0);
    assert!(asset.preferred_volume()? >= 0.0);
    assert!(asset.overall_duration_hint()?.as_numeric().is_some());
    assert!(!asset.available_metadata_formats()?.is_empty());
    let _ = asset.common_metadata()?;
    assert!(asset.is_playable()?);
    assert!(asset.is_readable()?);
    assert!(!asset.contains_fragments()?);
    let _ = asset.available_chapter_locales()?;
    let _ = asset.creation_date()?;
    let _ = asset.lyrics()?;
    let _ = asset.has_protected_content()?;
    let _ = asset.can_contain_fragments()?;
    let _ = asset.is_exportable()?;
    let _ = asset.is_composable()?;
    let _ = asset.is_compatible_with_air_play_video()?;
    let _ = asset
        .media_extension_properties()
        .map(|properties| properties.info())
        .transpose()?;
    asset.cancel_loading();

    assert!(track.time_range()?.duration.as_numeric().is_some());
    assert!(track.natural_time_scale()? > 0);
    assert!(track.preferred_volume()? >= 0.0);
    assert!(track.is_enabled()?);
    assert!(track.is_playable()?);
    assert!(track.is_decodable()?);
    assert!(track.is_self_contained()?);
    let _ = track.total_sample_data_length()?;
    assert!(track.is_audible()?);
    assert!(!track.is_visual()?);
    assert!(!track.is_legible()?);
    let _ = track.language_code()?;
    let _ = track.extended_language_tag()?;
    let _ = track.available_metadata_formats()?;
    let _ = track.available_track_association_types()?;
    let _ = track.can_provide_sample_cursors()?;
    let _ = track.min_frame_duration()?;
    let _ = track.requires_frame_reordering()?;
    Ok(())
}
