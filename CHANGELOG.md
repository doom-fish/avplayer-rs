# Changelog

## [0.3.5] - 2026-05-18

- Add one-line docs across the public safe and FFI surfaces, raising public-item rustdoc coverage to 95.0%.

## [0.3.4] - 2026-05-18

- Widen apple-cf version bound to `<0.10` so 0.9.x resolves.

All notable changes to this project will be documented in this file.

## [0.3.3] - 2026-05-18

### Changed

- Added `Debug` derives to the remaining 42 public raw-pointer wrapper structs and observer tokens, including the `Asset` / `Player` / `PlayerItem` families, output wrappers, timeline wrappers, and asset-reader wrappers.
- Retained the existing manual non-exhaustive `Debug` implementations on the 9 async future / accessor structs whose internal state is intentionally opaque.

## [0.3.2] - 2026-05-18

### Changed

- Re-exported the shared `SimpleCallback` and `DropCallback` aliases from `doom-fish-utils::ffi_callbacks` instead of maintaining local duplicate FFI typedefs.

## [0.3.1] - 2026-05-17

### Fixed

- Wrapped all `extern "C"` event-callback trampolines in `catch_cb_panic` to
  prevent panics from unwinding across the FFI boundary (undefined behaviour).
- Added `unsafe impl Send` for all raw-pointer RAII wrappers (`Player`,
  `PlayerItem`, `Asset`, observer tokens, output types, etc.) — the underlying
  Obj-C handles are safe to transfer across thread boundaries.
- Added `// SAFETY:` comments to unsafe blocks in `player.rs` and `async_api.rs`.
- Narrowed `apple-cf` version range from `>=0.4, <0.8` to `>=0.4, <0.6`.
- Set `doom-fish-utils` version constraint to `>=0.1, <0.3`.

## [0.3.0] - 2026-05-17

### Added

- **`async_api` module** (feature-gated behind `async`) providing executor-agnostic
  `Future` newtypes for AVFoundation's async-throws and completion-handler APIs:
  - `AsyncAsset` — `load_properties()`, `load_tracks()`,
    `load_tracks_with_media_type()`, `load_track_with_id()`
    wrapping `AVAsset.load(...)` async properties and
    `AVAsset.loadTracks(withMediaType:)` / `loadTrack(withTrackID:)`.
  - `AsyncPlayerItem` — `seek()` wrapping
    `AVPlayerItem.seek(to:completionHandler:)`.
  - `AsyncPlayer` — `seek()` and `preroll()` wrapping
    `AVPlayer.seek(to:completionHandler:)` and
    `AVPlayer.preroll(atRate:completionHandler:)`.
  - Result types `AssetProperties` and `TrackProperties`.
  - Swift bridge thunks in `swift-bridge/Sources/AVPlayerBridge/Async.swift`.
- New example `examples/21_async_api.rs` demonstrating all async APIs with
  `pollster::block_on`.
- New integration tests `tests/async_api_tests.rs` covering happy paths and
  error / missing-track paths for every future newtype.
- `doom-fish-utils` added as an optional dependency (enabled by `async` feature).
- `pollster = "0.3"` dev-dependency for single-threaded `block_on` in examples
  and tests.

## [0.2.2] - 2026-05-17

### Added

- Added `AVPlayer` rate-change observation (`AVPlayerRateDidChangeNotification`), typed waiting reasons, HDR notification-name access, audiovisual background playback policy, and network resource priority wrappers.
- Added `AVPlayerItem` time-jumped / failed-to-end / live-offset observer events, `VariantPreferences`, protected-content authorization status accessors, and custom video compositor introspection.
- Added abstract `PlayerItemOutput` timing/suppression helpers plus delegate observation for video, metadata, and legible outputs.
- Added `PlayerItemLegibleOutput` text-styling-resolution helpers and typed `PlayerItemTrackVideoFieldMode::DeinterlaceFields` access.

### Changed

- Raised the audited AVFoundation coverage from `70.64%` to `100.00%` in `COVERAGE_AUDIT.md` by closing the remaining 32 gaps.
- Expanded examples and integration tests to exercise the newly wrapped player, player-item, and output APIs.

## [0.2.1] - 2026-05-16

### Added

- Added `AVPlayerVideoOutput`, `AVVideoOutputSpecification`, tag-collection presets, and `AVPlayerVideoOutput.Configuration` wrappers.
- Added `AVPlayerItemRenderedLegibleOutput` plus rendered-caption image decoding and delegate callbacks.
- Added `AVPlayerItemMetadataCollector` / `AVPlayerItemMediaDataCollector` attachment, listing, and delegate observation.
- Added `AVPlayerInterstitialEvent`, `AVPlayerInterstitialEventController`, and `AVPlayerInterstitialEventMonitor` wrappers plus notification decoding.
- Added `AVPlayerItemIntegratedTimeline` snapshot/segment wrappers, periodic and boundary observers, out-of-sync observation, and seek helpers.
- Added numbered examples `16` through `20` and integration tests for the new surfaces.

### Changed

- Raised the audited AVFoundation coverage from `23.85%` to `70.64%` in `COVERAGE_AUDIT.md`.
- Added runtime availability guards for the newer macOS 15+ / 26+ AVFoundation surfaces.

## [0.2.0] - 2026-05-16

### Added

- Expanded `AVPlayer` coverage with action-at-item-end, volume, mute, time-control status, waiting reason, and media-selection-criteria control.
- Added `AVPlayerItem` buffering, bit-rate, maximum-resolution, audio-time-pitch, loaded-range, and output-count APIs.
- Added `AVPlayerLayer`, `AVQueuePlayer`, `AVPlayerLooper`, `AVPlayerItemTrack`, and `AVPlayerMediaSelectionCriteria` wrappers.
- Added `AVPlayerItemAccessLog` and `AVPlayerItemErrorLog` wrappers plus event decoding.
- Added `AVPlayerItemMetadataOutput`, `AVPlayerItemVideoOutput`, and `AVPlayerItemLegibleOutput` wrappers.
- Added `UrlAssetOptions` for `AVURLAsset` construction.
- Added one numbered example and one integration test for every requested player-subsystem area.
- Added `COVERAGE.md` mapping requested AVFoundation areas to Swift bridge files, Rust modules, examples, and tests.

### Changed

- Linked `QuartzCore` / `CoreGraphics` to support `AVPlayerLayer`.
- Updated README documentation to describe the broader `0.2.0` playback surface.

## [0.1.0] - 2026-05-16

### Added

- Initial `AVPlayer` / `AVPlayerItem` / `AVAsset` / `AVURLAsset` surface for macOS.
- Asynchronous asset-key loading and per-key status inspection.
- Track enumeration with media type, dimensions, frame rate, and data-rate readback.
- Metadata listing for assets and player items.
- Player controls: play, pause, rate, current time, duration, seek, status, and error.
- `AVPlayerItem` observer bridge covering status changes, presentation-size updates, and `AVPlayerItemDidPlayToEndTimeNotification`.
- Periodic and boundary time observers with Rust callback trampolines.
- `AVAssetReader`, `AVAssetReaderTrackOutput`, `AVAssetReaderAudioMixOutput`, and `AVAssetReaderVideoCompositionOutput`.
- `VideoOutputSettings` / `AudioOutputSettings` helpers for output conversion dictionaries.
- End-to-end smoke example `examples/01_smoke_surface.rs` writing artifacts into `target/example-artifacts`.
