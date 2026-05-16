#[path = "../tests/support/mod.rs"]
mod support;

use avplayer::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let player = support::player("example-avplayer")?;
    player.set_volume(0.25);
    player.set_muted(false);
    player.set_automatically_waits_to_minimize_stalling(false);
    player.set_action_at_item_end(PlayerActionAtItemEnd::Pause)?;

    println!("status: {:?}", player.status()?);
    println!("time-control: {:?}", player.time_control_status()?);
    println!("current item present: {}", player.current_item().is_some());
    println!("volume: {}", player.volume()?);
    println!("muted: {}", player.is_muted()?);
    player.play();
    player.pause();
    Ok(())
}
