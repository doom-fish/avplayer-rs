mod support;

use avplayer::prelude::*;

#[test]
fn avplayer_interstitial_event_surfaces_smoke() -> support::TestResult {
    let player = support::player("test-avplayer-interstitial-event")?;
    let item = player
        .current_item()
        .expect("player should have a current item");

    let event = PlayerInterstitialEvent::new(&item, Time::new(0, 1))?;
    event.set_identifier("smoke-interstitial")?;
    event.set_restrictions(
        PlayerInterstitialEventRestrictions::CONSTRAINS_SEEKING_FORWARD_IN_PRIMARY_CONTENT,
    );
    event.set_resumption_offset(Time::new(1, 1));
    event.set_playout_limit(Time::new(2, 1));
    event.set_aligns_start_with_primary_segment_boundary(true);
    event.set_aligns_resumption_with_primary_segment_boundary(true);
    event.set_cue(&PlayerInterstitialEventCue::JoinCue)?;
    event.set_will_play_once(true);
    event.set_timeline_occupancy(PlayerInterstitialEventTimelineOccupancy::SinglePoint);
    event.set_supplements_primary_content(true);
    event.set_content_may_vary(true);

    let info = event.info()?;
    assert_eq!(info.identifier, "smoke-interstitial");
    assert_eq!(info.cue, Some(PlayerInterstitialEventCue::JoinCue));
    assert!(info.will_play_once);
    assert!(info.has_primary_item);
    assert!(info.restrictions.contains(
        PlayerInterstitialEventRestrictions::CONSTRAINS_SEEKING_FORWARD_IN_PRIMARY_CONTENT,
    ));

    let controller = PlayerInterstitialEventController::new(&player)?;
    controller.set_events(&[&event])?;
    let _controller_state = controller.state()?;

    let monitor = PlayerInterstitialEventMonitor::new(&player)?;
    let _observer = monitor.observe(|_| {})?;
    let _monitor_state = monitor.state()?;

    assert!(!player_waiting_during_interstitial_event_reason()?.is_empty());
    Ok(())
}
