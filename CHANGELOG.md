# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.8.0] - 2026-09-24

### Security

- Periodic and boundary time observers freed their callback while a block that was already running could still use it, so dropping an observer during a callback was a use-after-free. The Swift observer box now owns the `CallbackContext` reference. Dropping closes the callback gate, calls `removeTimeObserver`, waits with `dispatch_sync` on the observer's queue (the main queue when no label was given) while a callback is running, and releases the context only after the last callback has returned. It never waits on the queue it is running on, and never waits when no callback is running, so dropping an observer from its own callback, or in a process that doesn't service the main queue, doesn't deadlock.
- The blocking helper behind `Asset::duration`, `metadata` and `url`, `AssetTrack` properties and `AssetImageGenerator::copy_image_at_time` kept its task running after the 30-second timeout, and a late failure wrote an error string into the caller's stack frame. Results now go into lock-protected heap state, only the caller writes the out-pointers, and a timeout cancels the task and returns an error.
- Every delegate, notification and KVO box (player item, rate, time, legible, metadata, video and rendered-legible outputs, metadata collector, interstitial monitor, integrated timeline, caption validation, resource loader, content-key session and asset download) freed its callback context right after unregistering and guarded delivery with an unsynchronized `disposed` flag, so a callback racing a drop read freed memory. Contexts are now `doom_fish_utils::callback_context::CallbackContext`s. The Swift box holds a reference behind a lock-protected gate that no new callback can enter after teardown starts, and whichever finishes last, teardown or the last running callback, releases it. The Rust handle deactivates the context before unregistering.
- `AssetDownloadURLSession` never invalidated its `URLSession`, so the session could call a delegate whose context had been freed, and the session, delegate and background identifier leaked. Dropping it now closes the delegate and calls `finishTasksAndInvalidate()`.
- Content-key session events carried `+1` key-request and content-key pointers inside the event JSON, which leaked whenever the event wasn't delivered. The pointers are now borrowed for the callback and retained by Rust when it builds the event.

### Fixed

- Documented Objective-C exception preconditions return an `AVPlayerError` instead of aborting the process: `AsyncPlayer::preroll` before `ReadyToPlay`, seeks to invalid or indefinite times or with invalid or negative tolerances (sync, async and integrated-timeline seeks), `PlayerActionAtItemEnd::Advance` on a plain `Player`, an item that belongs to another player (`Player::from_item`, `QueuePlayer::with_items`, `QueuePlayer` insertion, `replace_current_item`), duplicate `QueuePlayer::with_items` items, a zero-length or negative `PlayerLooper` range, `start_reading` called twice, copying samples or metadata/caption groups before reading starts, invalid `reset_for_reading_time_ranges` input or state, reader output settings changed after reading starts, reader outputs and adaptors that AVFoundation rejects, `setRate(_:time:atHostTime:)` while `automaticallyWaitsToMinimizeStalling` is on, a video composition AVFoundation rejects, and CEA-608 native legible-output subtypes. State-dependent checks go through a new Objective-C `@try`/`@catch` target, `AVPlayerObjCBridge`.
- `AVPlayerVideoOutput` samples and rendered caption images only reported tags and sizes and dropped the actual buffers; they now carry retained `CVPixelBuffer`/`CMSampleBuffer` values. Legible-output events also carry their native `CMSampleBuffer`s.
- `AsyncAsset::load_tracks_with_media_type` turned a media type containing a NUL byte into an empty string; the future now resolves to `InvalidArgument`.
- `Asset::tracks` used the deprecated synchronous `AVAsset.tracks`, which blocks on remote assets, and could index past a track list that changed between the count and the copy. It now loads `.tracks` asynchronously once.
- `ContentKeySession::observe_events`, `AssetDownloadURLSession::background_with_events`, `AssetResourceLoader::observe_loading_request_events` and `AssetReaderOutputCaptionAdaptor::observe_validation_events` panicked for a capacity of 0; they return `InvalidArgument`.
- A content-key event pointer that didn't fit in `usize` panicked inside the `extern "C"` trampoline.
- Null entries in reader track or queue-player item pointer arrays no longer force-unwrap in Swift.
- Resource-loader delegates registered without a queue label get a private serial queue instead of a nil delegate queue.
- Periodic time observers require a positive interval and boundary observers at least one numeric time.
- Tests that asserted nothing now check their results, and the URL-asset test no longer relies on `AVURLAssetPreferPreciseDurationAndTimingKey = false`, which makes AIFF duration loading fail on macOS 27.

### Changed

- **BREAKING:** observer, delegate and event-stream callbacks must be `Fn + Send + Sync` (they were `Fn + Send`), because nil-queue observers can be called from several threads at once. Periodic and boundary time observers still take `FnMut + Send`; their calls are serialized.
- **BREAKING:** `copy_next_sample_buffer` and `copy_next_video_pixel_buffer` on the reader outputs return `Result<Option<_>, AVPlayerError>`, and `set_always_copies_sample_data` and `AssetReaderOutput::set_supports_random_access` return `Result<(), AVPlayerError>`.
- **BREAKING:** `RenderedCaptionImage` has a `pixel_buffer` field, `PlayerVideoTaggedBuffer` a `buffer` field and `PlayerItemLegibleOutputEvent::AttributedStrings` a `native_sample_buffers` field.
- **BREAKING:** `UrlAssetOptions` is no longer `Copy`, `UrlAsset::from_file_path_with_options` and `from_remote_url_with_options` take `&UrlAssetOptions`, and `prefers_precise_duration_and_timing` takes `&self`.
- **BREAKING:** raw FFI: `av_player_replace_current_item`, `av_reader_output_copy_next_sample_buffer`, `av_reader_output_copy_next_video_pixel_buffer`, `av_reader_output_set_always_copies_sample_data`, `av_reader_output_set_supports_random_access` and `av_player_video_output_sample_json` take an extra out-parameter, and the legible and rendered-legible observer callbacks receive the buffer objects.
- `AVPlayerError` has a `TimedOut` variant, used when loading an asset's tracks times out.
- Async seek, preroll and track-loading futures resolve immediately with `InvalidArgument` or `OperationFailed` for rejected arguments.
- `rust-version` is 1.82 (was 1.76). Requires `apple-cf` `>=0.11, <0.12` (now by path as well) and `doom-fish-utils` `>=0.4.1, <0.5`.
- The retain/release `Drop` boilerplate of the wrapper types moved into a `retain_release_wrapper!` macro, with no behavior change.

### Added

- `Player::seek_to_with_tolerance`, `AsyncPlayer::seek_with_tolerance` and `AsyncPlayerItem::seek_with_tolerance`.
- `Player::set_rate_at_host_time`, `default_rate`, `set_default_rate`, `audio_output_device_unique_id`, `set_audio_output_device_unique_id` and `replace_current_item`.
- `Player::observe_status`, which reports `status` and `timeControlStatus` changes as `PlayerStatusEvent`s through a `PlayerStatusObserver`.
- `PlayerItem::forward_playback_end_time`, `set_forward_playback_end_time`, `reverse_playback_end_time`, `set_reverse_playback_end_time`, `can_step_forward`, `can_step_backward` and `step_by_count`.
- `PlayerItem::has_video_composition`, `set_video_composition_from_asset`, `clear_video_composition`, `has_audio_mix`, `set_audio_mix_volumes` (with `AudioMixTrackVolume`) and `clear_audio_mix`.
- `PlayerLayer::as_ptr` and `SampleBufferDisplayLayer::as_ptr`, the underlying `CALayer` for attaching to a layer tree. `AVPlayerView` (AVKit) remains out of scope.
- `PlayerVideoTaggedBufferData`.
- `UrlAssetOptions` builders for the MIME-type override, reference restrictions, HTTP cookies (`UrlAssetHttpCookie`), cellular, expensive and constrained network access, alias data references, request attribution (`UrlRequestAttribution`), HTTP user agent, primary session identifier and spherical-tag parsing.

### Removed

- Raw FFI `av_asset_track_count` and `av_asset_copy_track_at_index`, replaced by `av_asset_load_tracks`.

## [0.7.0] - 2026-05-20

### Added

- Bounded async stream wrappers for `AVAssetDownloadURLSession` background delegate events, `AVAssetResourceLoader` loading-request observation, and `AVAssetReaderOutputCaptionAdaptor` caption-validation observation, complementing the existing `AVContentKeySession` event stream support.
- Async stream smoke coverage for the new asset-download and resource-loader helpers plus compile coverage for caption-validation streams.

### Notes

- Phase 32 completeness + async sweep.

## [0.6.1] - 2026-05-20

- Added in-`src/` unit tests across `time`, `url_asset`, `player_interstitial_event`, and `player_media_selection_criteria` (Tier 2 quality polish), providing fast `cargo test --lib` fail-fast signal alongside the existing integration tests under `tests/`.

## [0.6.0] - 2026-05-19

### Added

- Completed the `AVContentKey` request/response flow with wrappers for `AVContentKeyRequest`, `AVPersistableContentKeyRequest`, `AVContentKeyResponse`, `AVContentKeySessionDelegate`, `AVContentKeyRequestRetryReason`, `AVContentKeyRequestStatus`, `AVContentKeySpecifier`, and `AVContentKey` plus FairPlay response-data helpers.
- Added `ContentKeySessionEventStream`-backed delegate observation so content-key session events surface as typed Rust enums.
- Added `tests/content_key_smoke.rs` covering synthetic clear-key request/response completion and content-key specifier round-tripping.

### Changed

- Bumped the crate version to `0.6.0` and widened the local `doom-fish-utils` compatibility bound to include the shared 0.3.x async-stream utilities used by the new content-key event stream.

## [0.5.1] - 2026-05-19

- Bump MSRV from 1.70 to 1.76 to match fleet baseline.

## [0.5.0] - 2026-05-19

### Added

- Added shared AVFoundation wrappers for `AVAssetImageGenerator`, `AVAssetCache`, and `AVAssetPlaybackAssistant`.
- Added the asset-resource-loading family: `AVAssetResourceLoader`, `AVAssetResourceLoaderDelegate`, `AVAssetResourceLoadingRequest`, `AVAssetResourceLoadingContentInformationRequest`, `AVAssetResourceLoadingDataRequest`, `AVAssetResourceLoadingRequestor`, and `AVAssetResourceRenewalRequest`.
- Added the HLS/offline-download family: `AVAssetDownloadTask`, `AVAggregateAssetDownloadTask`, `AVAssetDownloadURLSession`, `AVAssetDownloadConfiguration`, `AVAssetDownloadContentConfiguration`, `AVAssetDownloadStorageManager`, `AVAssetDownloadStorageManagementPolicy`, and `AVAssetDownloadDelegate`.
- Added `AVContentKeySession` basics plus `AVContentKeyRecipient` eligibility through `AVURLAsset`.

### Changed

- Bumped the crate version to `0.5.0` for the shared AVFoundation expansion and updated the coverage audit to document the new wrapped families plus the remaining `AVContentKey` request/response gap.

## [0.4.0] - 2026-05-19

### Added

- Expanded the AVFoundation wrapper surface with `AVMetadataGroup` / timed/date-range metadata groups, `AVMutableMetadataItem`, and `AVMetadataItemFilter`.
- Added media-selection wrappers for `AVMediaSelection`, `AVMutableMediaSelection`, `AVMediaSelectionGroup`, `AVMediaSelectionOption`, and availability-guarded presentation/custom-scheme types.
- Added `AVAssetVariant`, nested video/audio attribute wrappers, and `AVAssetVariantQualifier` accessors.
- Added `AVFragmentedAsset`, `AVFragmentedAssetTrack`, `AVFragmentedAssetMinder`, and `AVMediaExtensionProperties` wrappers.
- Added `AVAssetReaderOutput` base access, `AVAssetReaderSampleReferenceOutput`, `AVAssetReaderOutputMetadataAdaptor`, and `AVAssetReaderOutputCaptionAdaptor` wrappers.
- Added `AVSampleBufferDisplayLayer`, borrowed `AVPlayerItemMediaDataCollector` access, and `PlayerItemSegment` / `PlayerItemSegmentInfo` re-exports.
- Expanded `AVAsset` / `AVAssetTrack` property coverage with preferred-rate/volume, fragment/playability flags, chapter/metadata format access, language/timing flags, and sample-cursor helpers.
- Added integration coverage for the new asset, fragmented-asset, media-selection, metadata-group, reader-output, and sample-buffer-display-layer surfaces.

### Changed

- Reconciled the coverage documentation with the broader 0.4.0 audit scope and documented the supplemental AVFoundation families now wrapped beyond the earlier 109-symbol playback-only audit.
- Reworked `AVMutableDateRangeMetadataGroup` mutation to rebuild the underlying object instead of calling the current macOS SDK's crashing setters directly.
- Changed optional asset-variant bitrate, frame-rate, and channel-count accessors to return `Option` values instead of undocumented sentinel numbers.

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
