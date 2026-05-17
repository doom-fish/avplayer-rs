mod support;

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

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

    match item.variant_preferences() {
        Ok(original) => {
            let updated = original | VariantPreferences::SCALABILITY_TO_LOSSLESS_AUDIO;
            item.set_variant_preferences(updated)?;
            assert!(item
                .variant_preferences()?
                .contains(VariantPreferences::SCALABILITY_TO_LOSSLESS_AUDIO));
            item.set_variant_preferences(original)?;
        }
        Err(error) => assert!(error
            .to_string()
            .contains("AVPlayerItem.variantPreferences requires macOS 11.3+")),
    }

    let _ = item.authorization_required_for_playback()?;
    let _ = item.application_authorized_for_playback()?;
    let _ = item.content_authorized_for_playback()?;
    let _ = item.content_authorization_request_status()?;
    assert!(item.custom_video_compositor()?.is_none());

    let (tx, rx) = mpsc::channel();
    let _observer = item.observe(move |event| {
        let _ = tx.send(event);
    })?;
    let player = Player::from_item(&item)?;
    player.play();
    thread::sleep(Duration::from_millis(150));
    player.seek_to(Time::new(0, 1))?;
    thread::sleep(Duration::from_millis(100));
    player.pause();

    for _ in 0..10 {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(PlayerItemEvent::TimeJumped {
                has_originating_participant,
            }) => {
                let _ = has_originating_participant;
                break;
            }
            Ok(_) | Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}
