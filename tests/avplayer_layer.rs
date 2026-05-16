mod support;

use avplayer::prelude::*;

#[test]
fn avplayer_layer_tracks_player_and_video_gravity() -> support::TestResult {
    let player = support::player("test-avplayer-layer")?;
    let layer = PlayerLayer::new(Some(&player))?;

    assert!(layer.has_player()?);
    layer.set_video_gravity(VideoGravity::Resize)?;
    assert_eq!(layer.video_gravity()?, VideoGravity::Resize);
    assert!(layer.video_rect()?.width >= 0.0);
    assert!(layer.copy_displayed_pixel_buffer().is_none());
    layer.set_player(None);
    assert!(!layer.has_player()?);
    Ok(())
}
