mod support;

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use avplayer::prelude::*;

#[test]
fn avplayer_controls_and_state_round_trip() -> support::TestResult {
    let player = support::player("test-avplayer")?;

    assert!(player.current_item().is_some());
    player.set_volume(0.25);
    assert!((player.volume()? - 0.25).abs() < f32::EPSILON);
    player.set_muted(true);
    assert!(player.is_muted()?);
    player.set_automatically_waits_to_minimize_stalling(false);
    assert!(!player.automatically_waits_to_minimize_stalling()?);
    player.set_action_at_item_end(PlayerActionAtItemEnd::None)?;
    assert_eq!(player.action_at_item_end()?, PlayerActionAtItemEnd::None);

    let raw_waiting_reason = player.reason_for_waiting_to_play()?;
    let typed_waiting_reason = player.waiting_reason()?;
    assert_eq!(typed_waiting_reason.is_some(), raw_waiting_reason.is_some());

    match player_eligible_for_hdr_playback_did_change_notification() {
        Ok(notification) => {
            assert_eq!(
                notification,
                "AVPlayerEligibleForHDRPlaybackDidChangeNotification"
            );
            let _ = player.eligible_for_hdr_playback()?;
        }
        Err(error) => assert!(error
            .to_string()
            .contains("AVPlayerEligibleForHDRPlaybackDidChangeNotification requires macOS 10.15+")),
    }

    match player.audiovisual_background_playback_policy() {
        Ok(policy) => {
            player.set_audiovisual_background_playback_policy(policy)?;
            assert_eq!(player.audiovisual_background_playback_policy()?, policy);
        }
        Err(error) => assert!(error
            .to_string()
            .contains("AVPlayer.audiovisualBackgroundPlaybackPolicy requires macOS 12.0+")),
    }

    match player.network_resource_priority() {
        Ok(priority) => {
            player.set_network_resource_priority(priority)?;
            assert_eq!(player.network_resource_priority()?, priority);
        }
        Err(error) => assert!(error
            .to_string()
            .contains("AVPlayer.networkResourcePriority requires macOS 26.0+")),
    }

    let (tx, rx) = mpsc::channel();
    match player.observe_rate_changes(Some("tests.avplayer.rate"), move |event| {
        let _ = tx.send(event);
    }) {
        Ok(_observer) => {
            player.play();
            thread::sleep(Duration::from_millis(100));
            player.pause();

            let event = rx.recv_timeout(Duration::from_secs(2))?;
            assert!(event.rate >= 0.0);
            if let Some(reason) = event.reason {
                match reason {
                    PlayerRateDidChangeReason::SetRateCalled
                    | PlayerRateDidChangeReason::SetRateFailed
                    | PlayerRateDidChangeReason::AudioSessionInterrupted
                    | PlayerRateDidChangeReason::AppBackgrounded
                    | PlayerRateDidChangeReason::Unknown(_)
                    | _ => {}
                }
            }
        }
        Err(error) => assert!(error
            .to_string()
            .contains("AVPlayerRateDidChangeNotification requires macOS 12.0+")),
    }

    player.play();
    player.pause();
    assert!(player.rate()? >= 0.0);
    Ok(())
}
