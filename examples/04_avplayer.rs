#[path = "../tests/support/mod.rs"]
mod support;

use std::thread;
use std::time::Duration;

use avplayer::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let player = support::player("example-avplayer")?;
    player.set_volume(0.25);
    player.set_muted(false);
    player.set_automatically_waits_to_minimize_stalling(false);
    player.set_action_at_item_end(PlayerActionAtItemEnd::Pause)?;

    println!("status: {:?}", player.status()?);
    println!("time-control: {:?}", player.time_control_status()?);
    println!(
        "waiting reason raw: {:?}",
        player.reason_for_waiting_to_play()?
    );
    println!("waiting reason typed: {:?}", player.waiting_reason()?);
    println!("current item present: {}", player.current_item().is_some());
    println!("volume: {}", player.volume()?);
    println!("muted: {}", player.is_muted()?);

    match player_eligible_for_hdr_playback_did_change_notification() {
        Ok(notification) => {
            println!("HDR notification: {notification}");
            println!(
                "eligible for HDR playback: {}",
                player.eligible_for_hdr_playback()?
            );
        }
        Err(error) => println!("HDR APIs unavailable: {error}"),
    }

    match player.audiovisual_background_playback_policy() {
        Ok(policy) => {
            player.set_audiovisual_background_playback_policy(policy)?;
            println!("background playback policy: {policy:?}");
        }
        Err(error) => println!("background playback policy unavailable: {error}"),
    }

    match player.network_resource_priority() {
        Ok(priority) => {
            player.set_network_resource_priority(priority)?;
            println!("network resource priority: {priority:?}");
        }
        Err(error) => println!("network resource priority unavailable: {error}"),
    }

    match player.observe_rate_changes(Some("examples.avplayer.rate"), |event| {
        println!("rate change: {event:?}");
    }) {
        Ok(_observer) => {
            player.play();
            thread::sleep(Duration::from_millis(100));
            player.pause();
        }
        Err(error) => {
            println!("rate-change observation unavailable: {error}");
            player.play();
            player.pause();
        }
    }
    Ok(())
}
