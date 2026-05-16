# Changelog

All notable changes to this project will be documented in this file.

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
