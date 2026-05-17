# avplayer

Safe Rust bindings for Apple's [AVFoundation playback stack](https://developer.apple.com/documentation/avfoundation) on macOS: `AVPlayer`, `AVPlayerItem`, `AVPlayerLayer`, `AVQueuePlayer`, `AVPlayerLooper`, `AVAsset`, `AVURLAsset`, and `AVAssetReader`.

> **Status:** `0.2.2` closes the remaining audited AVPlayer / AVPlayerItem gaps and brings the crate to 100% coverage of the audited macOS playback surface, including rate-change notifications, HDR/background/network policy, variant preferences, protected-content authorization, and abstract/delegate player-item outputs. See [`COVERAGE.md`](COVERAGE.md) for the per-area map.

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

## Highlights

- `AVAsset` / `AVURLAsset`: async key loading, URL inspection, metadata, track enumeration, and `UrlAssetOptions`.
- `AVPlayer`: play/pause/rate/seek, volume + mute, action-at-item-end, time-control/waiting-state inspection, time observers, rate-change observation, HDR/background/network policy access, and media-selection criteria application.
- `AVPlayerItem`: observation callbacks (including time-jumped / failed-to-end / live-offset changes), buffering/bit-rate/resolution preferences, variant preferences, protected-content authorization status, custom compositor info, outputs, and per-item logs.
- `AVPlayerLayer`: player attachment, video gravity, video rect inspection, and displayed pixel-buffer access.
- `AVQueuePlayer` / `AVPlayerLooper`: queue mutation, current-item inspection, loop configuration, and loop-state reporting.
- `AVPlayerItemOutput`, `AVPlayerItemVideoOutput`, `AVPlayerItemMetadataOutput`, and `AVPlayerItemLegibleOutput`: attach/detach, base-output timing/suppression helpers, delegate observation, and text-styling/output configuration helpers.
- `AVPlayerItemTrack`, `AVPlayerItemAccessLog`, `AVPlayerItemErrorLog`, and `AVPlayerMediaSelectionCriteria` wrappers.
- `AVAssetReader`, `AssetReaderTrackOutput`, `AssetReaderAudioMixOutput`, and `AssetReaderVideoCompositionOutput` remain available for frame/sample extraction.

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
