#![cfg(feature = "async")]

mod support;

use avplayer::async_api::{AsyncAsset, AsyncPlayer, AsyncPlayerItem};
use avplayer::prelude::*;

#[test]
fn async_seeks_reject_invalid_times_without_calling_the_framework() -> support::TestResult {
    let player = support::player("test-async-seek-invalid")?;
    let async_player = AsyncPlayer::new(&player);
    for time in [Time::invalid(), Time::indefinite(), Time::new(3, 0)] {
        assert!(matches!(
            pollster::block_on(async_player.seek(time)),
            Err(AVPlayerError::InvalidArgument(_))
        ));
    }
    assert!(matches!(
        pollster::block_on(async_player.seek_with_tolerance(
            Time::new(0, 1),
            Time::negative_infinity(),
            Time::new(0, 1),
        )),
        Err(AVPlayerError::InvalidArgument(_))
    ));

    let item = support::player_item("test-async-item-seek-invalid")?;
    let async_item = AsyncPlayerItem::new(&item);
    assert!(matches!(
        pollster::block_on(async_item.seek(Time::indefinite())),
        Err(AVPlayerError::InvalidArgument(_))
    ));
    assert!(matches!(
        pollster::block_on(async_item.seek_with_tolerance(
            Time::new(0, 1),
            Time::new(0, 1),
            Time::new(-5, 10),
        )),
        Err(AVPlayerError::InvalidArgument(_))
    ));
    Ok(())
}

#[test]
fn tolerance_seeks_complete() -> support::TestResult {
    let item = support::player_item("test-async-tolerance-seek")?;
    let player = Player::from_item(&item)?;
    for _ in 0..50 {
        if item.status()? == PlayerItemStatus::ReadyToPlay {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    let finished = pollster::block_on(AsyncPlayer::new(&player).seek_with_tolerance(
        Time::new(0, 1),
        Time::new(0, 1),
        Time::new(0, 1),
    ))?;
    let item_finished = pollster::block_on(AsyncPlayerItem::new(&item).seek_with_tolerance(
        Time::new(0, 1),
        Time::positive_infinity(),
        Time::positive_infinity(),
    ))?;
    assert!(finished || item.status()? != PlayerItemStatus::ReadyToPlay);
    assert!(item_finished || item.status()? != PlayerItemStatus::ReadyToPlay);
    Ok(())
}

#[test]
fn preroll_before_ready_to_play_is_an_error() -> support::TestResult {
    let player = Player::from_remote_url("https://127.0.0.1:9/never-ready.m3u8")?;
    assert_ne!(player.status()?, PlayerStatus::ReadyToPlay);
    assert!(matches!(
        pollster::block_on(AsyncPlayer::new(&player).preroll(1.0)),
        Err(AVPlayerError::OperationFailed(_))
    ));
    Ok(())
}

#[test]
fn media_types_with_nul_bytes_are_rejected() -> support::TestResult {
    let asset = support::loaded_audio_asset("test-async-media-type-nul")?;
    assert!(matches!(
        pollster::block_on(AsyncAsset::new(asset.as_asset()).load_tracks_with_media_type("so\0un")),
        Err(AVPlayerError::InvalidArgument(_))
    ));
    let tracks =
        pollster::block_on(AsyncAsset::new(asset.as_asset()).load_tracks_with_media_type("soun"))?;
    assert!(tracks
        .iter()
        .all(|track| track.media_type == MediaType::Audio));
    assert!(!tracks.is_empty());
    Ok(())
}
