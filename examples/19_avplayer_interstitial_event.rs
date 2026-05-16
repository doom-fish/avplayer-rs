#[path = "../tests/support/mod.rs"]
mod support;

use avplayer::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let player = support::player("example-avplayer-interstitial-event")?;
    let item = player.current_item().expect("player should have a current item");

    let event = PlayerInterstitialEvent::new(&item, Time::new(0, 1))?;
    event.set_identifier("example-interstitial")?;
    event.set_cue(&PlayerInterstitialEventCue::JoinCue)?;
    event.set_will_play_once(true);
    event.set_restrictions(
        PlayerInterstitialEventRestrictions::CONSTRAINS_SEEKING_FORWARD_IN_PRIMARY_CONTENT,
    );

    let controller = PlayerInterstitialEventController::new(&player)?;
    controller.set_events(&[&event])?;

    let monitor = PlayerInterstitialEventMonitor::new(&player)?;
    let _observer = monitor.observe(|event| {
        println!("monitor event: {event:?}");
    })?;

    println!("event info: {:?}", event.info()?);
    println!("controller state: {:?}", controller.state()?);
    println!("monitor state: {:?}", monitor.state()?);
    println!(
        "waiting reason constant: {}",
        player_waiting_during_interstitial_event_reason()?
    );
    Ok(())
}
