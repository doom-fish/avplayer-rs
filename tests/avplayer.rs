mod support;

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
    player.play();
    player.pause();
    assert!(player.rate()? >= 0.0);
    Ok(())
}
