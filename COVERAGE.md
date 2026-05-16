# AVPlayer 0.2.0 coverage

This release expands the crate across every logical area requested for the `0.2.0` player-subsystem pass.

| Area | Swift bridge | Rust module | Example | Test | Notes |
| --- | --- | --- | --- | --- | --- |
| `AVPlayer` | `swift-bridge/Sources/AVPlayerBridge/Player.swift` | `src/player.rs`, `src/player_media_selection_criteria.rs` | `examples/04_avplayer.rs` | `tests/avplayer.rs` | Core playback, time observers, volume/mute, time-control status, action-at-item-end. |
| `AVPlayerItem` | `swift-bridge/Sources/AVPlayerBridge/PlayerItem.swift` | `src/player.rs`, `src/player_item.rs` | `examples/05_avplayer_item.rs` | `tests/avplayer_item.rs` | Observation, buffering, bit-rate, resolution, pitch, loaded/seekable ranges. |
| `AVPlayerLayer` | `swift-bridge/Sources/AVPlayerBridge/PlayerLayer.swift` | `src/player_layer.rs` | `examples/06_avplayer_layer.rs` | `tests/avplayer_layer.rs` | Layer/player attachment, gravity, video rect, displayed pixel buffer. |
| `AVQueuePlayer` | `swift-bridge/Sources/AVPlayerBridge/QueuePlayer.swift` | `src/queue_player.rs` | `examples/07_avqueue_player.rs` | `tests/avqueue_player.rs` | Queue mutation, item enumeration, action-at-item-end. |
| `AVPlayerLooper` | `swift-bridge/Sources/AVPlayerBridge/PlayerLooper.swift` | `src/player_looper.rs` | `examples/08_avplayer_looper.rs` | `tests/avplayer_looper.rs` | Loop creation, status, loop count, looping items, disable looping. |
| `AVAsset` | `swift-bridge/Sources/AVPlayerBridge/Asset.swift` | `src/asset.rs` | `examples/02_avasset.rs` | `tests/avasset.rs` | Duration, metadata, tracks, async key loading. |
| `AVURLAsset` | `swift-bridge/Sources/AVPlayerBridge/UrlAsset.swift` | `src/asset.rs`, `src/url_asset.rs` | `examples/03_avurlasset.rs` | `tests/avurlasset.rs` | File/remote URL creation plus `UrlAssetOptions`. |
| `AVPlayerItemAccessLog` | `swift-bridge/Sources/AVPlayerBridge/PlayerItemLogs.swift` | `src/player_item_access_log.rs` | `examples/09_avplayer_item_access_log.rs` | `tests/avplayer_item_access_log.rs` | Access-log presence, extended log, event decoding. |
| `AVPlayerItemErrorLog` | `swift-bridge/Sources/AVPlayerBridge/PlayerItemLogs.swift` | `src/player_item_error_log.rs` | `examples/10_avplayer_item_error_log.rs` | `tests/avplayer_item_error_log.rs` | Error-log presence, extended log, event decoding. |
| `AVPlayerItemMetadataOutput` | `swift-bridge/Sources/AVPlayerBridge/PlayerItemOutputs.swift` | `src/player_item_metadata_output.rs` | `examples/11_avplayer_item_metadata_output.rs` | `tests/avplayer_item_metadata_output.rs` | Output construction, attach/detach, identifiers, advance interval. |
| `AVPlayerItemVideoOutput` | `swift-bridge/Sources/AVPlayerBridge/PlayerItemOutputs.swift` | `src/player_item_video_output.rs` | `examples/12_avplayer_item_video_output.rs` | `tests/avplayer_item_video_output.rs` | Output construction, attach/detach, suppresses-player-rendering, pixel-buffer queries. |
| `AVPlayerItemLegibleOutput` | `swift-bridge/Sources/AVPlayerBridge/PlayerItemOutputs.swift` | `src/player_item_legible_output.rs` | `examples/13_avplayer_item_legible_output.rs` | `tests/avplayer_item_legible_output.rs` | Output construction, attach/detach, native subtype filtering, advance interval. |
| `AVPlayerItemTrack` | `swift-bridge/Sources/AVPlayerBridge/PlayerItemTrack.swift`, `PlayerItem.swift` | `src/player_item_track.rs` | `examples/14_avplayer_item_track.rs` | `tests/avplayer_item_track.rs` | Availability depends on AVFoundation materialization; examples/tests handle zero-track cases on synthesized AIFFs. |
| `AVPlayerMediaSelectionCriteria` | `swift-bridge/Sources/AVPlayerBridge/PlayerMediaSelectionCriteria.swift` | `src/player_media_selection_criteria.rs` | `examples/15_avplayer_media_selection_criteria.rs` | `tests/avplayer_media_selection_criteria.rs` | Preferred languages, preferred/principal characteristics, per-player application. |

## Adjacent APIs intentionally not added in 0.2.0

- `AVPlayerItem` custom media-selection accessors introduced in newer SDKs beyond the requested scope.
- `AVPlayerItemOutput` delegate callback plumbing (`AVPlayerItemOutputPullDelegate`, `AVPlayerItemMetadataOutputPushDelegate`, `AVPlayerItemLegibleOutputPushDelegate`).
- `AVPlayerItemRenderedLegibleOutput`, which was not part of the requested logical-area list.
