mod support;

use avplayer::prelude::*;

#[test]
fn avplayer_item_buffering_preferences_round_trip() -> support::TestResult {
    let item = support::player_item("test-avplayer-item")?;

    item.set_can_use_network_resources_for_live_streaming_while_paused(true);
    assert!(item.can_use_network_resources_for_live_streaming_while_paused()?);
    item.set_preferred_forward_buffer_duration(1.5);
    assert!((item.preferred_forward_buffer_duration()? - 1.5).abs() < 1e-9);
    item.set_preferred_peak_bit_rate(12_345.0);
    assert!((item.preferred_peak_bit_rate()? - 12_345.0).abs() < 1e-6);
    item.set_preferred_peak_bit_rate_for_expensive_networks(6_789.0);
    assert!((item.preferred_peak_bit_rate_for_expensive_networks()? - 6_789.0).abs() < 1e-6);
    item.set_audio_time_pitch_algorithm(&AudioTimePitchAlgorithm::Spectral)?;
    assert_eq!(
        item.audio_time_pitch_algorithm()?,
        AudioTimePitchAlgorithm::Spectral
    );
    assert_eq!(item.track_count()?, item.tracks()?.len());
    assert_eq!(item.output_count()?, 0);
    Ok(())
}
