mod support;

use avplayer::prelude::*;

#[test]
fn avasset_variant_enumeration_is_queryable() -> support::TestResult {
    let asset = support::loaded_audio_asset("test-avasset-variant")?;
    let variants = asset.variants()?;

    for variant in &variants {
        let _ = variant.peak_bit_rate()?;
        let _ = variant.average_bit_rate()?;
        let _ = variant.url()?;
        let _ = AssetVariantQualifier::from_variant(variant)?;

        if let Some(video) = variant.video_attributes() {
            let _ = video.video_range()?;
            let _ = video.codec_types()?;
            let _ = video.presentation_size()?;
            let _ = video.nominal_frame_rate()?;
            for layout in video.video_layout_attributes()? {
                let _ = layout.stereo_view_components()?;
                let _ = layout.projection_type()?;
            }
        }

        if let Some(audio) = variant.audio_attributes() {
            let _ = audio.format_ids()?;
        }
    }
    Ok(())
}
