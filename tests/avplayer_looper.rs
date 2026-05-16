mod support;

use avplayer::prelude::*;

#[test]
fn avplayer_looper_reports_state_without_error() -> support::TestResult {
    let queue = QueuePlayer::new()?;
    let template = support::player_item("test-avplayer-looper")?;
    let looper = PlayerLooper::new(&queue, &template)?;

    assert!(looper.error()?.is_none());
    assert!(matches!(
        looper.status()?,
        PlayerLooperStatus::Unknown | PlayerLooperStatus::Ready
    ));
    let _ = looper.loop_count()?;
    let _ = looper.looping_items()?;
    looper.disable_looping();
    Ok(())
}
