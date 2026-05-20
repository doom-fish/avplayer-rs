# avplayer

Safe Rust bindings for Apple's [AVFoundation playback stack](https://developer.apple.com/documentation/avfoundation) on macOS: `AVPlayer`, `AVPlayerItem`, `AVPlayerLayer`, `AVQueuePlayer`, `AVPlayerLooper`, `AVAsset`, `AVURLAsset`, and `AVAssetReader`.

> **Status:** `0.7.0` expands the crate beyond the original playback-only audit
> with metadata-group, media-selection, asset-variant, fragmented-asset,
> reader-adaptor, and sample-buffer-display-layer wrappers while retaining the
> Tier-1 `async_api` module plus Tier-2 delegate/event streams for content-key,
> asset-download, resource-loader, and caption-validation observation. See [`COVERAGE.md`](COVERAGE.md).

## Quick start

```rust,no_run
use avplayer::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let asset = UrlAsset::from_file_path("target/example-artifacts/test.aiff")?;
    asset.load_values_asynchronously(["duration", "tracks", "metadata"])?;

    let player = Player::from_asset(asset.as_asset())?;
    player.set_action_at_item_end(PlayerActionAtItemEnd::Pause)?;

    println!("duration: {:?}", asset.duration()?);
    println!("tracks: {}", asset.tracks()?.len());
    println!("time control: {:?}", player.time_control_status()?);

    player.play();
    player.pause();
    Ok(())
}
```

## Async API

Enable the `async` Cargo feature for executor-agnostic `Future` wrappers around
`AVFoundation`'s `async throws` and completion-handler APIs:

```toml
avplayer = { version = "0.7", features = ["async"] }
```

```rust,no_run
use avplayer::{UrlAsset, async_api::AsyncAsset};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async {
        let asset = UrlAsset::from_file_path("my.mp4")?;
        let props = AsyncAsset::new(asset.as_asset()).load_properties().await?;
        println!("playable={} duration={:?}", props.is_playable, props.duration);
        let tracks = AsyncAsset::new(asset.as_asset()).load_tracks().await?;
        println!("{} tracks", tracks.len());
        Ok(())
    })
}
```

| Type | API wrapped |
|------|-------------|
| `async_api::AsyncAsset` | `AVAsset.load(...)`, `loadTracks(withMediaType:)`, `loadTrack(withTrackID:)` |
| `async_api::AsyncPlayerItem` | `AVPlayerItem.seek(to:completionHandler:)` |
| `async_api::AsyncPlayer` | `AVPlayer.seek(to:completionHandler:)`, `preroll(atRate:completionHandler:)` |

The Tier-1 `async_api` module covers one-shot futures. Multi-fire delegate
streams live alongside their owning types: use `ContentKeySession::observe_events`
/ `event_stream`, `AssetDownloadURLSession::background_with_events` /
`background_events`, `AssetResourceLoader::observe_loading_request_events` /
`loading_request_stream`, and `AssetReaderOutputCaptionAdaptor::observe_validation_events`
/ `validation_event_stream`.

## Highlights

- `AVAsset` / `AVURLAsset`: async key loading, metadata-group construction helpers, media-selection access, asset variants, fragmented-asset/media-extension helpers, URL inspection, and broader asset/track property coverage.
- `AVPlayer`: play/pause/rate/seek, volume + mute, action-at-item-end, time-control/waiting-state inspection, time observers, rate-change observation, HDR/background/network policy access, and media-selection criteria application.
- `AVPlayerItem`: observation callbacks (including time-jumped / failed-to-end / live-offset changes), buffering/bit-rate/resolution preferences, variant preferences, protected-content authorization status, custom compositor info, outputs, and per-item logs.
- `AVPlayerLayer`: player attachment, video gravity, video rect inspection, and displayed pixel-buffer access.
- `AVQueuePlayer` / `AVPlayerLooper`: queue mutation, current-item inspection, loop configuration, and loop-state reporting.
- `AVPlayerItemOutput`, `AVPlayerItemVideoOutput`, `AVPlayerItemMetadataOutput`, and `AVPlayerItemLegibleOutput`: attach/detach, base-output timing/suppression helpers, delegate observation, and text-styling/output configuration helpers.
- `AVPlayerItemTrack`, `AVPlayerItemAccessLog`, `AVPlayerItemErrorLog`, and `AVPlayerMediaSelectionCriteria` wrappers.
- `AVAssetReader`, `AssetReaderOutput`, `AssetReaderTrackOutput`, `AssetReaderSampleReferenceOutput`, metadata/caption adaptors, and `AVSampleBufferDisplayLayer` are available for frame/sample extraction and display.

## Examples

Every requested subsystem area now has a numbered example:

- `01_smoke_surface`
- `02_avasset`
- `03_avurlasset`
- `04_avplayer`
- `05_avplayer_item`
- `06_avplayer_layer`
- `07_avqueue_player`
- `08_avplayer_looper`
- `09_avplayer_item_access_log`
- `10_avplayer_item_error_log`
- `11_avplayer_item_metadata_output`
- `12_avplayer_item_video_output`
- `13_avplayer_item_legible_output`
- `14_avplayer_item_track`
- `15_avplayer_media_selection_criteria`

Run any example with:

```bash
cargo run --example 15_avplayer_media_selection_criteria
```

Examples write synthesized media into `target/example-artifacts/` and avoid `/tmp`.

## Notes

- `AVPlayerItemTrack` materialization is media- and readiness-dependent. On synthesized `AIFF`s, `AVFoundation` may legitimately report zero `AVPlayerItemTrack` instances until it fully prepares the item.
- The current macOS SDK used for this release does not expose `AVPlayerItem.externalMetadata`, so `PlayerItem::metadata()` continues to surface the underlying asset metadata.

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
