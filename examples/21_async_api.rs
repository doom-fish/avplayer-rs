//! Example: Tier-1 async API — asset property loading, track loading, seek,
//! and preroll using `pollster::block_on`.
//!
//! Run on a macOS host:
//! ```
//! cargo run --example 21_async_api --features async
//! ```
//!
//! The example uses a synthetic short AIFF produced by the existing smoke test
//! in `target/example-artifacts/test.aiff`.  If that file does not exist the
//! example exits 0 with a notice (headless CI safe).

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async_main())
}

async fn async_main() -> Result<(), Box<dyn std::error::Error>> {
    #![allow(clippy::future_not_send)]
    use avplayer::Time;
    use avplayer::{
        async_api::{AsyncAsset, AsyncPlayer, AsyncPlayerItem},
        Player, PlayerItem, UrlAsset,
    };

    let path = "target/example-artifacts/test.aiff";
    if !std::path::Path::new(path).exists() {
        println!("NOTE: {path} not found — run example 01_smoke_surface first.");
        println!("Skipping async example (exit 0).");
        return Ok(());
    }

    // ── Asset property loading ────────────────────────────────────────────────
    println!("=== AsyncAsset::load_properties ===");
    let asset = UrlAsset::from_file_path(path)?;
    let props = AsyncAsset::new(asset.as_asset()).load_properties().await?;
    println!("  duration       : {:?}", props.duration);
    println!("  is_playable    : {}", props.is_playable);
    println!("  is_exportable  : {}", props.is_exportable);
    println!("  has_drm        : {}", props.has_protected_content);
    println!("  preferred_rate : {}", props.preferred_rate);
    println!("  metadata items : {}", props.metadata.len());

    // ── All-tracks loading ────────────────────────────────────────────────────
    println!("=== AsyncAsset::load_tracks ===");
    let tracks = AsyncAsset::new(asset.as_asset()).load_tracks().await?;
    println!("  track count: {}", tracks.len());
    for t in &tracks {
        println!(
            "  track {} ({:?})  {}×{}  fps={:?}",
            t.track_id,
            t.media_type,
            t.natural_size.width,
            t.natural_size.height,
            t.nominal_frame_rate,
        );
    }

    // ── Tracks by media type ──────────────────────────────────────────────────
    println!("=== AsyncAsset::load_tracks_with_media_type(\"audio\") ===");
    let audio_tracks = AsyncAsset::new(asset.as_asset())
        .load_tracks_with_media_type("audio")
        .await?;
    println!("  audio track count: {}", audio_tracks.len());

    // ── Track by ID ───────────────────────────────────────────────────────────
    println!("=== AsyncAsset::load_track_with_id ===");
    if let Some(first) = tracks.first() {
        let by_id = AsyncAsset::new(asset.as_asset())
            .load_track_with_id(first.track_id)
            .await?;
        println!(
            "  found track by id={}: {}",
            first.track_id,
            by_id.is_some()
        );
    }
    let missing = AsyncAsset::new(asset.as_asset())
        .load_track_with_id(99999)
        .await?;
    println!("  track with id=99999 found: {}", missing.is_some());

    // ── PlayerItem seek ───────────────────────────────────────────────────────
    println!("=== AsyncPlayerItem::seek ===");
    let item = PlayerItem::from_file_path(path)?;
    let seek_time = Time::new(0, 1);
    let finished = AsyncPlayerItem::new(&item).seek(seek_time).await?;
    println!("  seek finished: {finished}");

    // ── Player seek ───────────────────────────────────────────────────────────
    println!("=== AsyncPlayer::seek ===");
    let player = Player::from_item(&item)?;
    let finished = AsyncPlayer::new(&player).seek(Time::new(0, 1)).await?;
    println!("  player seek finished: {finished}");

    // ── Player preroll ────────────────────────────────────────────────────────
    println!("=== AsyncPlayer::preroll ===");
    let finished = AsyncPlayer::new(&player).preroll(1.0).await?;
    println!("  preroll finished: {finished}");

    println!("All async examples completed successfully.");
    Ok(())
}
