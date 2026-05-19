# avplayer-rs coverage audit (vs MacOSX26.2.sdk)

SDK_PUBLIC_SYMBOLS: 109
VERIFIED: 109
GAPS: 0
EXEMPT: 0
COVERAGE_PCT: 100.00%

> Filtered out macOS-unavailable top-level symbols (`AVPlayerHDRMode`, `AVPlayerAvailableHDRModesDidChangeNotification`). No deprecated-but-macOS-available top-level symbols remained, so EXEMPT is 0. All 109 remaining macOS-available top-level symbols in the original playback-focused audit are wrapped.

## Supplemental 0.4.0 expansion set

The original 109-symbol audit was narrower than the follow-up re-audit that identified additional AVFoundation types worth wrapping around assets, metadata, media selection, variants, fragmented assets, reader adaptors, and display layers. Release `0.4.0` adds wrappers for that supplemental set, including:

- `AVMetadataGroup`, `AVTimedMetadataGroup`, `AVMutableTimedMetadataGroup`, `AVDateRangeMetadataGroup`, `AVMutableDateRangeMetadataGroup`, `AVMutableMetadataItem`, `AVMetadataItemFilter`
- `AVMediaSelection`, `AVMutableMediaSelection`, `AVMediaSelectionGroup`, `AVMediaSelectionOption`, `AVCustomMediaSelectionScheme`, `AVMediaPresentationSelector`, `AVMediaPresentationSetting`
- `AVAssetVariant`, `AVAssetVariant.VideoAttributes`, `AVAssetVariant.VideoAttributes.LayoutAttributes`, `AVAssetVariant.AudioAttributes`, `AVAssetVariant.AudioAttributes.RenditionSpecificAttributes`, `AVAssetVariantQualifier`
- `AVFragmentedAsset`, `AVFragmentedAssetTrack`, `AVFragmentedAssetMinder`, `AVMediaExtensionProperties`
- `AVAssetReaderOutput`, `AVAssetReaderSampleReferenceOutput`, `AVAssetReaderOutputMetadataAdaptor`, `AVAssetReaderOutputCaptionAdaptor`, `AVAssetReaderCaptionValidationHandling`, `AVSampleBufferDisplayLayer`
- Supplemental `AVAsset` / `AVAssetTrack` property coverage plus borrowed `AVPlayerItemMediaDataCollector` access and `PlayerItemSegment` re-exports

See `COVERAGE.md` for the 0.4.0 file-by-file map and integration-test coverage for these additions.

## Supplemental 0.5.0 shared AVFoundation expansion set

Release `0.5.0` adds a new shared-AVFoundation expansion layer in `avplayer-rs`, prioritizing the highest-value asset and loader APIs first while keeping the crate buildable and safe.

Implemented in this release:

- `AVAssetImageGenerator`, including configurable tolerances / aperture / dynamic-range policy plus retained image snapshots
- `AVAssetCache` and `AVAssetPlaybackAssistant`
- `AVAssetResourceLoader`, `AVAssetResourceLoaderDelegate`, `AVAssetResourceLoadingRequest`, `AVAssetResourceLoadingContentInformationRequest`, `AVAssetResourceLoadingDataRequest`, `AVAssetResourceLoadingRequestor`, and `AVAssetResourceRenewalRequest`
- `AVAssetDownloadTask`, `AVAssetDownloadURLSession`, `AVAssetDownloadConfiguration`, `AVAssetDownloadContentConfiguration`, `AVAssetDownloadStorageManager`, `AVAssetDownloadStorageManagementPolicy`, `AVAssetDownloadDelegate`, and `AVAggregateAssetDownloadTask`
- `AVContentKeySession` basics plus `AVContentKeyRecipient` eligibility surfaced through `AVURLAsset`

## Supplemental 0.6.0 content-key flow completion

Release `0.6.0` closes the remaining content-key gap with wrappers for:

- `AVContentKeyRequest`, `AVPersistableContentKeyRequest`, `AVContentKeyResponse`, `AVContentKeySessionDelegate`, `AVContentKeyRequestRetryReason`, `AVContentKeyRequestStatus`, `AVContentKeySpecifier`, and `AVContentKey`
- `ContentKeySessionEventStream`-backed delegate observation plus FairPlay / Clear Key / authorization-token response helpers for synthetic and playback-driven key fulfilment

## 🟢 VERIFIED
| Symbol | Kind | Header | Wrapped by |
| --- | --- | --- | --- |
| AVPlayerStatus | Typedef | AVPlayer.h | PlayerStatus |
| AVPlayer | Interface | AVPlayer.h | Player |
| AVPlayerTimeControlStatus | Typedef | AVPlayer.h | PlayerTimeControlStatus |
| AVPlayerWaitingReason | Typedef | AVPlayer.h | Player::reason_for_waiting_to_play() -> Option<String> |
| AVPlayerActionAtItemEnd | Typedef | AVPlayer.h | PlayerActionAtItemEnd |
| AVQueuePlayer | Interface | AVPlayer.h | QueuePlayer |
| AVPlayerItemDidPlayToEndTimeNotification | Constant | AVPlayerItem.h | PlayerItem::observe() / PlayerItemEvent::DidPlayToEnd |
| AVPlayerItemPlaybackStalledNotification | Constant | AVPlayerItem.h | PlayerItem::observe() / PlayerItemEvent::PlaybackStalled |
| AVPlayerItemNewAccessLogEntryNotification | Constant | AVPlayerItem.h | PlayerItem::observe() / PlayerItemEvent::NewAccessLogEntry |
| AVPlayerItemNewErrorLogEntryNotification | Constant | AVPlayerItem.h | PlayerItem::observe() / PlayerItemEvent::NewErrorLogEntry |
| AVPlayerItemMediaSelectionDidChangeNotification | Constant | AVPlayerItem.h | PlayerItem::observe() / PlayerItemEvent::MediaSelectionChanged |
| AVPlayerItemStatus | Typedef | AVPlayerItem.h | PlayerItemStatus |
| AVPlayerItem | Interface | AVPlayerItem.h | PlayerItem |
| AVPlayerItemAccessLog | Interface | AVPlayerItem.h | PlayerItemAccessLog |
| AVPlayerItemErrorLog | Interface | AVPlayerItem.h | PlayerItemErrorLog |
| AVPlayerItemAccessLogEvent | Interface | AVPlayerItem.h | PlayerItemAccessLogEvent |
| AVPlayerItemErrorLogEvent | Interface | AVPlayerItem.h | PlayerItemErrorLogEvent |
| AVPlayerItemVideoOutput | Interface | AVPlayerItemOutput.h | PlayerItemVideoOutput |
| AVPlayerItemLegibleOutput | Interface | AVPlayerItemOutput.h | PlayerItemLegibleOutput |
| AVPlayerItemMetadataOutput | Interface | AVPlayerItemOutput.h | PlayerItemMetadataOutput |
| AVPlayerItemTrack | Interface | AVPlayerItemTrack.h | PlayerItemTrack |
| AVPlayerLayer | Interface | AVPlayerLayer.h | PlayerLayer |
| AVPlayerLooperStatus | Typedef | AVPlayerLooper.h | PlayerLooperStatus |
| AVPlayerLooperItemOrdering | Typedef | AVPlayerLooper.h | PlayerLooperItemOrdering |
| AVPlayerLooper | Interface | AVPlayerLooper.h | PlayerLooper |
| AVPlayerMediaSelectionCriteria | Interface | AVPlayerMediaSelectionCriteria.h | PlayerMediaSelectionCriteria |
| AVPlayerItemRenderedLegibleOutputPushDelegate | Protocol | AVPlayerItemOutput.h | PlayerItemRenderedLegibleOutput::observe() / RenderedLegibleOutputEvent |
| AVPlayerItemRenderedLegibleOutput | Interface | AVPlayerItemOutput.h | PlayerItemRenderedLegibleOutput |
| AVPlayerVideoOutput | Interface | AVPlayerOutput.h | PlayerVideoOutput + Player::set_video_output() |
| CMTagCollectionVideoOutputPreset | Typedef | AVPlayerOutput.h | PlayerVideoOutputTagCollectionPreset |
| CMTagCollectionCreateWithVideoOutputPreset | Function | AVPlayerOutput.h | PlayerVideoOutputTagCollection::from_preset() |
| AVVideoOutputSpecification | Interface | AVPlayerOutput.h | VideoOutputSpecification |
| AVPlayerVideoOutputConfiguration | Interface | AVPlayerOutput.h | PlayerVideoOutputConfiguration |
| AVPlayerInterstitialEventRestrictions | Typedef | AVPlayerInterstitialEventController.h | PlayerInterstitialEventRestrictions |
| AVPlayerInterstitialEventNoCue | Constant | AVPlayerInterstitialEventController.h | PlayerInterstitialEventCue::NoCue |
| AVPlayerInterstitialEventJoinCue | Constant | AVPlayerInterstitialEventController.h | PlayerInterstitialEventCue::JoinCue |
| AVPlayerInterstitialEventLeaveCue | Constant | AVPlayerInterstitialEventController.h | PlayerInterstitialEventCue::LeaveCue |
| AVPlayerInterstitialEventTimelineOccupancy | Typedef | AVPlayerInterstitialEventController.h | PlayerInterstitialEventTimelineOccupancy |
| AVPlayerInterstitialEvent | Interface | AVPlayerInterstitialEventController.h | PlayerInterstitialEvent |
| AVPlayerInterstitialEventAssetListResponseStatus | Typedef | AVPlayerInterstitialEventController.h | PlayerInterstitialEventAssetListResponseStatus |
| AVPlayerInterstitialEventSkippableEventState | Typedef | AVPlayerInterstitialEventController.h | PlayerInterstitialEventSkippableEventState |
| AVPlayerInterstitialEventMonitor | Interface | AVPlayerInterstitialEventController.h | PlayerInterstitialEventMonitor |
| AVPlayerInterstitialEventMonitorEventsDidChangeNotification | Constant | AVPlayerInterstitialEventController.h | PlayerInterstitialEventMonitor::observe() / PlayerInterstitialEventMonitorEvent::EventsDidChange |
| AVPlayerInterstitialEventMonitorCurrentEventDidChangeNotification | Constant | AVPlayerInterstitialEventController.h | PlayerInterstitialEventMonitor::observe() / PlayerInterstitialEventMonitorEvent::CurrentEventDidChange |
| AVPlayerInterstitialEventMonitorAssetListResponseStatusDidChangeNotification | Constant | AVPlayerInterstitialEventController.h | PlayerInterstitialEventMonitor::observe() / PlayerInterstitialEventMonitorEvent::AssetListResponseStatusDidChange |
| AVPlayerInterstitialEventMonitorAssetListResponseStatusDidChangeEventKey | Constant | AVPlayerInterstitialEventController.h | PlayerInterstitialEventMonitorEvent::AssetListResponseStatusDidChange::interstitial_event |
| AVPlayerInterstitialEventMonitorAssetListResponseStatusDidChangeStatusKey | Constant | AVPlayerInterstitialEventController.h | PlayerInterstitialEventMonitorEvent::AssetListResponseStatusDidChange::status |
| AVPlayerInterstitialEventMonitorAssetListResponseStatusDidChangeErrorKey | Constant | AVPlayerInterstitialEventController.h | PlayerInterstitialEventMonitorEvent::AssetListResponseStatusDidChange::error_message |
| AVPlayerInterstitialEventMonitorCurrentEventSkippableStateDidChangeNotification | Constant | AVPlayerInterstitialEventController.h | PlayerInterstitialEventMonitor::observe() / PlayerInterstitialEventMonitorEvent::CurrentEventSkippableStateDidChange |
| AVPlayerInterstitialEventMonitorCurrentEventSkippableStateDidChangeEventKey | Constant | AVPlayerInterstitialEventController.h | PlayerInterstitialEventMonitorEvent::CurrentEventSkippableStateDidChange::interstitial_event |
| AVPlayerInterstitialEventMonitorCurrentEventSkippableStateDidChangeStateKey | Constant | AVPlayerInterstitialEventController.h | PlayerInterstitialEventMonitorEvent::CurrentEventSkippableStateDidChange::state |
| AVPlayerInterstitialEventMonitorCurrentEventSkippableStateDidChangeSkipControlLabelKey | Constant | AVPlayerInterstitialEventController.h | PlayerInterstitialEventMonitorEvent::CurrentEventSkippableStateDidChange::skip_control_label |
| AVPlayerInterstitialEventMonitorCurrentEventSkippedNotification | Constant | AVPlayerInterstitialEventController.h | PlayerInterstitialEventMonitor::observe() / PlayerInterstitialEventMonitorEvent::CurrentEventSkipped |
| AVPlayerInterstitialEventMonitorCurrentEventSkippedEventKey | Constant | AVPlayerInterstitialEventController.h | PlayerInterstitialEventMonitorEvent::CurrentEventSkipped::interstitial_event |
| AVPlayerInterstitialEventMonitorInterstitialEventWasUnscheduledNotification | Constant | AVPlayerInterstitialEventController.h | PlayerInterstitialEventMonitor::observe() / PlayerInterstitialEventMonitorEvent::InterstitialEventWasUnscheduled |
| AVPlayerInterstitialEventMonitorInterstitialEventWasUnscheduledEventKey | Constant | AVPlayerInterstitialEventController.h | PlayerInterstitialEventMonitorEvent::InterstitialEventWasUnscheduled::interstitial_event |
| AVPlayerInterstitialEventMonitorInterstitialEventWasUnscheduledErrorKey | Constant | AVPlayerInterstitialEventController.h | PlayerInterstitialEventMonitorEvent::InterstitialEventWasUnscheduled::error_message |
| AVPlayerInterstitialEventMonitorInterstitialEventDidFinishNotification | Constant | AVPlayerInterstitialEventController.h | PlayerInterstitialEventMonitor::observe() / PlayerInterstitialEventMonitorEvent::InterstitialEventDidFinish |
| AVPlayerInterstitialEventMonitorInterstitialEventDidFinishEventKey | Constant | AVPlayerInterstitialEventController.h | PlayerInterstitialEventMonitorEvent::InterstitialEventDidFinish::interstitial_event |
| AVPlayerInterstitialEventMonitorInterstitialEventDidFinishPlayoutTimeKey | Constant | AVPlayerInterstitialEventController.h | PlayerInterstitialEventMonitorEvent::InterstitialEventDidFinish::playout_time |
| AVPlayerInterstitialEventMonitorInterstitialEventDidFinishDidPlayEntireEventKey | Constant | AVPlayerInterstitialEventController.h | PlayerInterstitialEventMonitorEvent::InterstitialEventDidFinish::did_play_entire_event |
| AVPlayerInterstitialEventController | Interface | AVPlayerInterstitialEventController.h | PlayerInterstitialEventController |
| AVPlayerWaitingDuringInterstitialEventReason | Constant | AVPlayerInterstitialEventController.h | player_waiting_during_interstitial_event_reason() |
| AVPlayerItemSegmentType | Typedef | AVPlayerItemIntegratedTimeline.h | PlayerItemSegmentType |
| AVPlayerItemSegment | Interface | AVPlayerItemIntegratedTimeline.h | PlayerItemIntegratedTimelineSegment |
| AVPlayerItemIntegratedTimelineSnapshot | Interface | AVPlayerItemIntegratedTimeline.h | PlayerItemIntegratedTimelineSnapshot |
| AVPlayerItemIntegratedTimeline | Interface | AVPlayerItemIntegratedTimeline.h | PlayerItemIntegratedTimeline |
| AVPlayerItemIntegratedTimelineObserver | Protocol | AVPlayerItemIntegratedTimeline.h | PlayerItemIntegratedTimelineObserver |
| AVPlayerIntegratedTimelineSnapshotsOutOfSyncNotification | Constant | AVPlayerItemIntegratedTimeline.h | player_integrated_timeline_snapshots_out_of_sync_notification() / PlayerItemIntegratedTimeline::observe_snapshots_out_of_sync() |
| AVPlayerIntegratedTimelineSnapshotsOutOfSyncReasonKey | Constant | AVPlayerItemIntegratedTimeline.h | player_integrated_timeline_snapshots_out_of_sync_reason_key() / PlayerIntegratedTimelineOutOfSyncEvent::reason |
| AVPlayerIntegratedTimelineSnapshotsOutOfSyncReason | Typedef | AVPlayerItemIntegratedTimeline.h | PlayerIntegratedTimelineSnapshotsOutOfSyncReason |
| AVPlayerIntegratedTimelineSnapshotsOutOfSyncReasonSegmentsChanged | Constant | AVPlayerItemIntegratedTimeline.h | player_integrated_timeline_snapshots_out_of_sync_reason_segments_changed() |
| AVPlayerIntegratedTimelineSnapshotsOutOfSyncReasonCurrentSegmentChanged | Constant | AVPlayerItemIntegratedTimeline.h | player_integrated_timeline_snapshots_out_of_sync_reason_current_segment_changed() |
| AVPlayerIntegratedTimelineSnapshotsOutOfSyncReasonLoadedTimeRangesChanged | Constant | AVPlayerItemIntegratedTimeline.h | player_integrated_timeline_snapshots_out_of_sync_reason_loaded_time_ranges_changed() |
| AVPlayerItemMediaDataCollector | Interface | AVPlayerItemMediaDataCollector.h | PlayerItem::media_data_collectors() / PlayerItemMediaDataCollectorInfo |
| AVPlayerItemMetadataCollectorPushDelegate | Protocol | AVPlayerItemMediaDataCollector.h | PlayerItemMetadataCollector::observe() / MetadataCollectorEvent |
| AVPlayerItemMetadataCollector | Interface | AVPlayerItemMediaDataCollector.h | PlayerItemMetadataCollector |
| AVPlayerRateDidChangeNotification | Constant | AVPlayer.h | Player::observe_rate_changes() / PlayerRateDidChangeEvent |
| AVPlayerRateDidChangeReasonKey | Constant | AVPlayer.h | Player::observe_rate_changes() / PlayerRateDidChangeEvent::reason |
| AVPlayerRateDidChangeOriginatingParticipantKey | Constant | AVPlayer.h | Player::observe_rate_changes() / PlayerRateDidChangeEvent::has_originating_participant |
| AVPlayerRateDidChangeReason | Typedef | AVPlayer.h | PlayerRateDidChangeReason |
| AVPlayerRateDidChangeReasonSetRateCalled | Constant | AVPlayer.h | PlayerRateDidChangeReason::SetRateCalled |
| AVPlayerRateDidChangeReasonSetRateFailed | Constant | AVPlayer.h | PlayerRateDidChangeReason::SetRateFailed |
| AVPlayerRateDidChangeReasonAudioSessionInterrupted | Constant | AVPlayer.h | PlayerRateDidChangeReason::AudioSessionInterrupted |
| AVPlayerRateDidChangeReasonAppBackgrounded | Constant | AVPlayer.h | PlayerRateDidChangeReason::AppBackgrounded |
| AVPlayerWaitingToMinimizeStallsReason | Constant | AVPlayer.h | PlayerWaitingReason::ToMinimizeStalls |
| AVPlayerWaitingWhileEvaluatingBufferingRateReason | Constant | AVPlayer.h | PlayerWaitingReason::WhileEvaluatingBufferingRate |
| AVPlayerWaitingWithNoItemToPlayReason | Constant | AVPlayer.h | PlayerWaitingReason::WithNoItemToPlay |
| AVPlayerWaitingForCoordinatedPlaybackReason | Constant | AVPlayer.h | PlayerWaitingReason::ForCoordinatedPlayback |
| AVPlayerEligibleForHDRPlaybackDidChangeNotification | Constant | AVPlayer.h | player_eligible_for_hdr_playback_did_change_notification() |
| AVPlayerAudiovisualBackgroundPlaybackPolicy | Typedef | AVPlayer.h | PlayerAudiovisualBackgroundPlaybackPolicy |
| AVPlayerNetworkResourcePriority | Typedef | AVPlayer.h | PlayerNetworkResourcePriority |
| AVPlayerItemTimeJumpedNotification | Constant | AVPlayerItem.h | PlayerItem::observe() / PlayerItemEvent::TimeJumped |
| AVPlayerItemFailedToPlayToEndTimeNotification | Constant | AVPlayerItem.h | PlayerItem::observe() / PlayerItemEvent::FailedToPlayToEnd |
| AVPlayerItemRecommendedTimeOffsetFromLiveDidChangeNotification | Constant | AVPlayerItem.h | PlayerItem::observe() / PlayerItemEvent::RecommendedTimeOffsetFromLiveDidChange |
| AVPlayerItemFailedToPlayToEndTimeErrorKey | Constant | AVPlayerItem.h | PlayerItemEvent::FailedToPlayToEnd::error_message |
| AVPlayerItemTimeJumpedOriginatingParticipantKey | Constant | AVPlayerItem.h | PlayerItemEvent::TimeJumped::has_originating_participant |
| AVVideoCompositing | Protocol | AVPlayerItem.h | PlayerItem::custom_video_compositor() / PlayerItemVideoCompositorInfo |
| AVVariantPreferences | Typedef | AVPlayerItem.h | VariantPreferences |
| AVPlayerItemOutput | Interface | AVPlayerItemOutput.h | PlayerItemOutput<'_> |
| AVPlayerItemOutputPullDelegate | Protocol | AVPlayerItemOutput.h | PlayerItemVideoOutput::observe() / PlayerItemVideoOutputEvent |
| AVPlayerItemLegibleOutputPushDelegate | Protocol | AVPlayerItemOutput.h | PlayerItemLegibleOutput::observe() / PlayerItemLegibleOutputEvent |
| AVPlayerItemLegibleOutputTextStylingResolution | Typedef | AVPlayerItemOutput.h | PlayerItemLegibleOutputTextStylingResolution |
| AVPlayerItemLegibleOutputTextStylingResolutionDefault | Constant | AVPlayerItemOutput.h | PlayerItemLegibleOutputTextStylingResolution::Default |
| AVPlayerItemLegibleOutputTextStylingResolutionSourceAndRulesOnly | Constant | AVPlayerItemOutput.h | PlayerItemLegibleOutputTextStylingResolution::SourceAndRulesOnly |
| AVPlayerItemOutputPushDelegate | Protocol | AVPlayerItemOutput.h | PlayerItemMetadataOutput::observe() + PlayerItemLegibleOutput::observe() |
| AVPlayerItemMetadataOutputPushDelegate | Protocol | AVPlayerItemOutput.h | PlayerItemMetadataOutput::observe() / MetadataOutputEvent |
| AVPlayerItemTrackVideoFieldModeDeinterlaceFields | Constant | AVPlayerItemTrack.h | PlayerItemTrackVideoFieldMode::DeinterlaceFields |
| AVContentAuthorizationStatus | Typedef | AVPlayerItemProtectedContentAdditions.h | ContentAuthorizationStatus |

## 🔴 GAPS
No macOS-available gaps remain in the audited header set.

## ⏭️ EXEMPT
No macOS-available deprecated top-level symbols in the audited header set.

| Symbol | Kind | Header | Reason | SDK attribute |
| --- | --- | --- | --- | --- |
