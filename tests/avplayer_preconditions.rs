mod support;

use std::sync::mpsc;
use std::time::Duration;

use avplayer::prelude::*;

#[test]
fn seeks_reject_invalid_indefinite_and_negative_tolerance_times() -> support::TestResult {
    let player = support::player("test-preconditions-seek")?;
    for time in [Time::invalid(), Time::indefinite(), Time::new(1, 0)] {
        assert!(matches!(
            player.seek_to(time),
            Err(AVPlayerError::InvalidArgument(_))
        ));
    }
    assert!(matches!(
        player.seek_to_with_tolerance(Time::new(0, 1), Time::new(-1, 600), Time::new(0, 1)),
        Err(AVPlayerError::InvalidArgument(_))
    ));
    assert!(matches!(
        player.seek_to_with_tolerance(Time::new(0, 1), Time::new(0, 1), Time::invalid()),
        Err(AVPlayerError::InvalidArgument(_))
    ));
    player.seek_to_with_tolerance(Time::new(0, 1), Time::new(0, 1), Time::positive_infinity())?;
    player.seek_to(Time::new(0, 1))?;
    Ok(())
}

#[test]
fn advance_at_item_end_is_rejected_on_a_plain_player() -> support::TestResult {
    let player = support::player("test-preconditions-advance")?;
    assert!(matches!(
        player.set_action_at_item_end(PlayerActionAtItemEnd::Advance),
        Err(AVPlayerError::InvalidArgument(_))
    ));
    assert_eq!(player.action_at_item_end()?, PlayerActionAtItemEnd::Pause);

    let queue = QueuePlayer::new()?;
    queue.set_action_at_item_end(PlayerActionAtItemEnd::Advance)?;
    assert_eq!(queue.action_at_item_end()?, PlayerActionAtItemEnd::Advance);
    Ok(())
}

#[test]
fn an_item_owned_by_another_player_is_rejected() -> support::TestResult {
    let item = support::player_item("test-preconditions-owned-item")?;
    let owner = Player::from_item(&item)?;
    assert!(owner.current_item().is_some());

    assert!(matches!(
        Player::from_item(&item),
        Err(AVPlayerError::PlayerCreateFailed(_))
    ));
    assert!(matches!(
        QueuePlayer::with_items(&[&item]),
        Err(AVPlayerError::PlayerCreateFailed(_))
    ));
    let other = support::player("test-preconditions-other-player")?;
    assert!(matches!(
        other.replace_current_item(Some(&item)),
        Err(AVPlayerError::OperationFailed(_))
    ));
    other.replace_current_item(None)?;
    assert!(other.current_item().is_none());
    Ok(())
}

#[test]
fn duplicate_queue_items_are_rejected() -> support::TestResult {
    let item = support::player_item("test-preconditions-duplicate")?;
    let error = QueuePlayer::with_items(&[&item, &item]).unwrap_err();
    assert!(matches!(error, AVPlayerError::PlayerCreateFailed(_)));
    assert!(error.to_string().contains("same item twice"));
    Ok(())
}

#[test]
fn a_zero_duration_loop_range_is_rejected() -> support::TestResult {
    let queue = QueuePlayer::new()?;
    let template = support::player_item("test-preconditions-looper")?;
    let error = PlayerLooper::with_time_range(
        &queue,
        &template,
        TimeRange::new(Time::new(0, 1), Time::new(0, 1)),
    )
    .unwrap_err();
    assert!(error.to_string().contains("positive"));
    assert!(PlayerLooper::with_time_range(
        &queue,
        &template,
        TimeRange::new(Time::new(-1, 1), Time::new(1, 1)),
    )
    .is_err());
    Ok(())
}

#[test]
fn host_time_rate_changes_require_stall_waiting_to_be_disabled() -> support::TestResult {
    let player = support::player("test-preconditions-host-time")?;
    player.set_automatically_waits_to_minimize_stalling(true);
    assert!(matches!(
        player.set_rate_at_host_time(1.0, Time::invalid(), Time::invalid()),
        Err(AVPlayerError::InvalidArgument(_))
    ));
    Ok(())
}

#[test]
fn default_rate_and_output_device_round_trip() -> support::TestResult {
    let player = support::player("test-preconditions-default-rate")?;
    player.set_default_rate(1.5)?;
    assert!((player.default_rate()? - 1.5).abs() < f32::EPSILON);
    assert!(matches!(
        player.set_default_rate(f32::NAN),
        Err(AVPlayerError::InvalidArgument(_))
    ));
    player.set_audio_output_device_unique_id(None)?;
    assert_eq!(player.audio_output_device_unique_id()?, None);
    assert!(matches!(
        player.set_audio_output_device_unique_id(Some("bad\0id")),
        Err(AVPlayerError::InvalidArgument(_))
    ));
    Ok(())
}

#[test]
fn status_observers_report_the_initial_state() -> support::TestResult {
    let player = support::player("test-preconditions-status")?;
    let (tx, rx) = mpsc::channel();
    let observer = player.observe_status(move |event| {
        let _ = tx.send(event);
    })?;
    let mut saw_status = false;
    let mut saw_time_control = false;
    while let Ok(event) = rx.recv_timeout(Duration::from_secs(2)) {
        match event {
            PlayerStatusEvent::StatusChanged { .. } => saw_status = true,
            PlayerStatusEvent::TimeControlStatusChanged {
                time_control_status,
                ..
            } => {
                saw_time_control = true;
                assert_eq!(time_control_status, player.time_control_status()?);
            }
            _ => {}
        }
        if saw_status && saw_time_control {
            break;
        }
    }
    assert!(saw_status && saw_time_control);
    drop(observer);
    assert!(rx.recv_timeout(Duration::from_millis(200)).is_err());
    Ok(())
}

#[test]
fn item_playback_end_times_and_audio_mix_round_trip() -> support::TestResult {
    let item = support::player_item("test-preconditions-item-members")?;
    item.set_forward_playback_end_time(Time::new(1, 2));
    assert_eq!(item.forward_playback_end_time()?, Time::new(1, 2));
    item.set_forward_playback_end_time(Time::invalid());
    assert_eq!(item.forward_playback_end_time()?, Time::invalid());
    item.set_reverse_playback_end_time(Time::new(1, 4));
    assert_eq!(item.reverse_playback_end_time()?, Time::new(1, 4));

    assert!(!item.has_audio_mix()?);
    item.set_audio_mix_volumes(&[AudioMixTrackVolume::new(1, 0.5)])?;
    assert!(item.has_audio_mix()?);
    assert!(matches!(
        item.set_audio_mix_volumes(&[AudioMixTrackVolume::new(1, -1.0)]),
        Err(AVPlayerError::InvalidArgument(_))
    ));
    item.clear_audio_mix();
    assert!(!item.has_audio_mix()?);

    assert!(!item.has_video_composition()?);
    item.clear_video_composition();
    assert!(!item.has_video_composition()?);
    let _ = item.can_step_forward()?;
    let _ = item.can_step_backward()?;
    item.step_by_count(1);
    Ok(())
}

#[test]
fn layers_expose_their_raw_layer_pointer() -> support::TestResult {
    let player = support::player("test-preconditions-layer")?;
    let layer = PlayerLayer::new(Some(&player))?;
    assert!(!layer.as_ptr().is_null());
    let display_layer = SampleBufferDisplayLayer::new()?;
    assert!(!display_layer.as_ptr().is_null());
    assert_ne!(layer.as_ptr(), display_layer.as_ptr());
    Ok(())
}

#[test]
fn zero_capacity_event_streams_are_rejected() -> support::TestResult {
    let identifier = format!("tests.avplayer.zero-capacity.{}", std::process::id());
    assert!(matches!(
        AssetDownloadURLSession::background_events(&identifier, 0),
        Err(AVPlayerError::InvalidArgument(_))
    ));
    let asset = support::loaded_audio_asset("test-preconditions-zero-capacity")?;
    assert!(matches!(
        asset.resource_loader().loading_request_stream(0),
        Err(AVPlayerError::InvalidArgument(_))
    ));
    Ok(())
}
