# Changelog

All notable changes to this project will be documented in this file.

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
