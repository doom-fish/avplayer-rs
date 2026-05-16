# avplayer

Safe Rust bindings for Apple's [AVPlayer](https://developer.apple.com/documentation/avfoundation/avplayer), `AVPlayerItem`, `AVAsset`, `AVURLAsset`, and `AVAssetReader` on macOS.

> **Status:** `0.1.0` covers practical playback + inspection workflows: URL/file assets, asynchronous key loading, track + metadata listing, basic AVPlayer control, `AVPlayerItem` observation, time observers, and frame/sample reading through `AVAssetReader` outputs.

## Quick start

```rust,no_run
use avplayer::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let asset = UrlAsset::from_file_path("target/example-artifacts/test.aiff")?;
    asset.load_values_asynchronously(["duration", "tracks", "metadata"])?;

    println!("duration: {:?}", asset.duration()?);
    println!("tracks: {}", asset.tracks()?.len());

    let player = Player::from_asset(asset.as_asset())?;
    println!("status: {:?}", player.status()?);
    player.play();
    player.pause();
    Ok(())
}
```

## Highlights

- `UrlAsset::from_file_path` / `UrlAsset::from_remote_url`
- `Asset::load_values_asynchronously`, `status_of_value`, `duration`, `tracks`, `metadata`
- `Player::from_url`, `Player::from_asset`, `play`, `pause`, `rate`, `seek_to`, `current_time`, `duration`
- `PlayerItem::observe` for status / presentation-size / end-of-playback events
- `Player::add_periodic_time_observer` / `add_boundary_time_observer`
- `AssetReader`, `AssetReaderTrackOutput`, `AssetReaderAudioMixOutput`, `AssetReaderVideoCompositionOutput`
- `VideoOutputSettings` + `AudioOutputSettings` helpers for `AVAssetReader` conversion dictionaries
- `apple-cf` interop for `CMSampleBuffer` and `CVPixelBuffer`

## Smoke example

```bash
cargo run --all-features --example 01_smoke_surface
```

The smoke example synthesizes a short AIFF under `target/example-artifacts/`, loads it as an `AVURLAsset`, inspects metadata/tracks, reads the first sample buffers through `AVAssetReader`, and exercises `AVPlayer` control + observer registration.

## Notes

- `AVPlayerLayer` is intentionally out of scope for this crate; it belongs to AppKit/UIKit presentation layers.
- The current macOS SDK used for this release does not expose an `AVPlayerItem.externalMetadata` property, so `PlayerItem::metadata()` returns the underlying asset metadata instead.

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
