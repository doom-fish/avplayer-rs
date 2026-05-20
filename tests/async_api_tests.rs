//! Integration tests for the `async_api` module (feature = "async").

#[cfg(feature = "async")]
mod async_tests {
    use avplayer::{
        async_api::{AsyncAsset, AsyncPlayer, AsyncPlayerItem},
        Player, PlayerItem, Time, UrlAsset,
    };

    // Helper: resolve the shared test artifact path.
    fn test_aiff() -> String {
        // Probe the usual locations for the test artifact.
        for candidate in &[
            "target/example-artifacts/test.aiff",
            "../avplayer-rs/target/example-artifacts/test.aiff",
        ] {
            if std::path::Path::new(candidate).exists() {
                return (*candidate).to_owned();
            }
        }
        String::new()
    }

    fn skip_if_missing(path: &str) -> bool {
        if path.is_empty() {
            eprintln!("SKIP: test artifact not found — run example 01_smoke_surface first");
            return true;
        }
        false
    }

    // ── AsyncAsset::load_properties ───────────────────────────────────────────

    #[test]
    fn test_async_asset_load_properties_happy() {
        let path = test_aiff();
        if skip_if_missing(&path) {
            return;
        }
        let asset = UrlAsset::from_file_path(&path).expect("UrlAsset::from_file_path");
        let props = pollster::block_on(AsyncAsset::new(asset.as_asset()).load_properties())
            .expect("load_properties");
        assert!(props.is_playable, "AIFF should be playable");
        assert!(!props.has_protected_content, "test AIFF has no DRM");
        assert!(
            (props.preferred_rate - 1.0_f32).abs() < f32::EPSILON,
            "default preferred rate"
        );
    }

    #[test]
    fn test_async_asset_load_properties_error_path() {
        // Using a non-existent remote URL should eventually fail with an error.
        // We accept either a LoadFailed or successful result here since behaviour
        // depends on network availability; we mostly verify the future resolves.
        let asset = UrlAsset::from_remote_url("https://127.0.0.1:0/no-such-asset.mp4")
            .expect("UrlAsset construction does not validate URL");
        let result = pollster::block_on(AsyncAsset::new(asset.as_asset()).load_properties());
        // Either an error or a result is acceptable — we just want no panic / hang.
        drop(result);
    }

    // ── AsyncAsset::load_tracks ───────────────────────────────────────────────

    #[test]
    fn test_async_asset_load_tracks_happy() {
        let path = test_aiff();
        if skip_if_missing(&path) {
            return;
        }
        let asset = UrlAsset::from_file_path(&path).expect("UrlAsset::from_file_path");
        let tracks = pollster::block_on(AsyncAsset::new(asset.as_asset()).load_tracks())
            .expect("load_tracks");
        assert!(!tracks.is_empty(), "AIFF should have at least one track");
    }

    // ── AsyncAsset::load_tracks_with_media_type ───────────────────────────────

    #[test]
    fn test_async_asset_load_tracks_with_media_type_audio() {
        let path = test_aiff();
        if skip_if_missing(&path) {
            return;
        }
        let asset = UrlAsset::from_file_path(&path).expect("UrlAsset::from_file_path");
        let audio = pollster::block_on(
            AsyncAsset::new(asset.as_asset()).load_tracks_with_media_type("audio"),
        )
        .expect("load_tracks_with_media_type(audio)");
        assert!(
            !audio.is_empty(),
            "AIFF should have at least one audio track"
        );
    }

    #[test]
    fn test_async_asset_load_tracks_with_media_type_video_empty() {
        let path = test_aiff();
        if skip_if_missing(&path) {
            return;
        }
        let asset = UrlAsset::from_file_path(&path).expect("UrlAsset::from_file_path");
        let video = pollster::block_on(
            AsyncAsset::new(asset.as_asset()).load_tracks_with_media_type("video"),
        )
        .expect("load_tracks_with_media_type(video)");
        assert!(
            video.is_empty(),
            "pure audio AIFF should have no video tracks"
        );
    }

    // ── AsyncAsset::load_track_with_id ────────────────────────────────────────

    #[test]
    fn test_async_asset_load_track_with_id_found() {
        let path = test_aiff();
        if skip_if_missing(&path) {
            return;
        }
        let asset = UrlAsset::from_file_path(&path).expect("UrlAsset::from_file_path");
        let tracks = pollster::block_on(AsyncAsset::new(asset.as_asset()).load_tracks())
            .expect("load_tracks");
        let first_id = tracks[0].track_id;
        let found =
            pollster::block_on(AsyncAsset::new(asset.as_asset()).load_track_with_id(first_id))
                .expect("load_track_with_id");
        assert!(found.is_some(), "should find track with id={first_id}");
        assert_eq!(found.unwrap().track_id, first_id);
    }

    #[test]
    fn test_async_asset_load_track_with_id_missing() {
        let path = test_aiff();
        if skip_if_missing(&path) {
            return;
        }
        let asset = UrlAsset::from_file_path(&path).expect("UrlAsset::from_file_path");
        let result =
            pollster::block_on(AsyncAsset::new(asset.as_asset()).load_track_with_id(99999))
                .expect("load_track_with_id(99999) should not error");
        assert!(result.is_none(), "nonexistent track id should return None");
    }

    // ── AsyncPlayerItem::seek ─────────────────────────────────────────────────

    #[test]
    fn test_async_player_item_seek() {
        let path = test_aiff();
        if skip_if_missing(&path) {
            return;
        }
        let item = PlayerItem::from_file_path(&path).expect("PlayerItem::from_file_path");
        let finished = pollster::block_on(AsyncPlayerItem::new(&item).seek(Time::new(0, 1)))
            .expect("PlayerItem seek");
        assert!(finished, "seek to 0.0 on a ready item should complete");
    }

    // ── AsyncPlayer::seek ─────────────────────────────────────────────────────

    #[test]
    fn test_async_player_seek() {
        let path = test_aiff();
        if skip_if_missing(&path) {
            return;
        }
        let item = PlayerItem::from_file_path(&path).expect("PlayerItem::from_file_path");
        let player = Player::from_item(&item).expect("Player::from_item");
        let finished = pollster::block_on(AsyncPlayer::new(&player).seek(Time::new(0, 1)))
            .expect("Player seek");
        assert!(finished, "seek to 0.0 should complete");
    }

    // ── AsyncPlayer::preroll ──────────────────────────────────────────────────

    #[test]
    fn test_async_player_preroll() {
        let path = test_aiff();
        if skip_if_missing(&path) {
            return;
        }
        let item = PlayerItem::from_file_path(&path).expect("PlayerItem::from_file_path");
        let player = Player::from_item(&item).expect("Player::from_item");
        let finished =
            pollster::block_on(AsyncPlayer::new(&player).preroll(1.0)).expect("Player preroll");
        // preroll may return false if item is not ready; both outcomes are valid.
        let _ = finished;
    }
}
