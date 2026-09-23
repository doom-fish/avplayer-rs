# avplayer

Safe Rust bindings for Apple's [AVFoundation playback stack](https://developer.apple.com/documentation/avfoundation) on macOS: `AVPlayer`, `AVPlayerItem`, `AVPlayerLayer`, `AVQueuePlayer`, `AVPlayerLooper`, `AVAsset`, `AVURLAsset`, and `AVAssetReader`.

> **Status:** `0.8.0` is a soundness release. Observer, delegate and stream
> callbacks keep their Rust context alive until the framework can no longer
> call them, blocking helpers no longer write into the caller's memory after a
> timeout, and documented Objective-C exception preconditions return errors
> instead of aborting the process. Video and caption outputs now deliver the
> actual `CVPixelBuffer`/`CMSampleBuffer` objects. See [`CHANGELOG.md`](CHANGELOG.md)
> and [`COVERAGE.md`](COVERAGE.md).

Requires macOS 13 or later (the Swift bridge's deployment target). Newer APIs
are checked at runtime and return an error on older systems.

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
avplayer = { version = "0.8", features = ["async"] }
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

## Threading and the main run loop

`AVPlayer` only becomes `ReadyToPlay`, starts playing and delivers main-queue
callbacks while the process's main run loop is running. A command-line tool or
test whose main thread blocks (for example in `join()`) sees players stay in
`PlayerStatus::Unknown`. Run the main run loop (`CFRunLoopRun`, an app event
loop) and drive the player from it or from another thread.

Time observers registered without a queue label, and other observers that
deliver on the main queue, fire on the main thread. Observers registered with a
queue label fire on a private serial queue.

## Callbacks and teardown

- Callbacks must be `Fn + Send + Sync`, because the framework can call them
  from several threads at once. Periodic and boundary time observers take
  `FnMut + Send`; their calls are serialized.
- Dropping an observer, delegate or stream stops new callbacks. A callback that
  is already running keeps its closure alive until it returns, so an observer
  can be dropped from inside its own callback.
- Dropping a time observer from another thread waits for a callback that is
  still running on the observer's queue (the main queue when no label was
  given). It never waits when nothing is running, so dropping one in a process
  that doesn't service the main queue doesn't hang.
- Dropping an `AssetDownloadURLSession` calls `finishTasksAndInvalidate()`.
  Downloads already running continue in the background without delivering
  events; call `invalidate_and_cancel()` first to stop them.

## Errors instead of Objective-C exceptions

`AVFoundation` raises Objective-C exceptions, which abort the process, when some
preconditions are violated. These cases return an `AVPlayerError` instead:
seeking to an invalid or indefinite time or with a negative tolerance,
prerolling before `ReadyToPlay`, `PlayerActionAtItemEnd::Advance` on a plain
`Player`, an item that already belongs to another player, duplicate items in
`QueuePlayer::with_items`, a zero-length `PlayerLooper` range,
`setRate(_:time:atHostTime:)` while `automaticallyWaitsToMinimizeStalling` is
on, calling `start_reading` twice, copying samples before reading starts,
invalid `reset_for_reading_time_ranges` input, reader output settings changed
after reading starts, unsupported reader output settings, and CEA-608 native
legible-output subtypes. The state-dependent checks go through a small
Objective-C `@try`/`@catch` shim in `swift-bridge/Sources/AVPlayerObjCBridge`.

## Displaying video

`PlayerLayer::as_ptr()` and `SampleBufferDisplayLayer::as_ptr()` return the
underlying `CALayer` (borrowed, valid while the wrapper is alive) so it can be
added to a layer tree, for example with `coreanimation-rs`. `AVPlayerView`
belongs to `AVKit` and isn't wrapped by this crate.

## Highlights

- `AVAsset` / `AVURLAsset`: async key loading, metadata-group construction helpers, media-selection access, asset variants, fragmented-asset/media-extension helpers, URL inspection, and broader asset/track property coverage.
- `AVPlayer`: play/pause/rate/seek (including tolerance seeks and `setRate(_:time:atHostTime:)`), default rate, audio output device, volume + mute, action-at-item-end, time-control/waiting-state inspection, time observers, status/time-control and rate-change observation, HDR/background/network policy access, and media-selection criteria application.
- `AVPlayerItem`: observation callbacks (including time-jumped / failed-to-end / live-offset changes), forward/reverse playback end times, stepping, a video composition built from an asset's properties, per-track audio-mix volumes, buffering/bit-rate/resolution preferences, variant preferences, protected-content authorization status, custom compositor info, outputs, and per-item logs.
- `AVPlayerLayer`: player attachment, video gravity, video rect inspection, and displayed pixel-buffer access.
- `AVQueuePlayer` / `AVPlayerLooper`: queue mutation, current-item inspection, loop configuration, and loop-state reporting.
- `AVPlayerItemOutput`, `AVPlayerItemVideoOutput`, `AVPlayerItemMetadataOutput`, and `AVPlayerItemLegibleOutput`: attach/detach, base-output timing/suppression helpers, delegate observation (legible output events carry the native `CMSampleBuffer`s), and text-styling/output configuration helpers.
- `AVPlayerVideoOutput` samples carry each tagged buffer's `CVPixelBuffer` or `CMSampleBuffer`, and `AVPlayerItemRenderedLegibleOutput` caption images carry their `CVPixelBuffer`.
- `UrlAssetOptions` covers the `AVURLAsset` option keys: precise timing, MIME-type override, reference restrictions, HTTP cookies and user agent, network-access limits, alias data references, request attribution, primary session identifier and spherical-tag parsing.
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
