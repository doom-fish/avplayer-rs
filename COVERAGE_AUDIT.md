# avplayer-rs coverage audit (vs MacOSX26.2.sdk)

SDK_PUBLIC_SYMBOLS: 109
VERIFIED: 77
GAPS: 32
EXEMPT: 0
COVERAGE_PCT: 70.64%

> Filtered out macOS-unavailable top-level symbols (`AVPlayerHDRMode`, `AVPlayerAvailableHDRModesDidChangeNotification`). No deprecated-but-macOS-available top-level symbols remained, so EXEMPT is 0.

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

## 🔴 GAPS
| Symbol | Kind | Header | Notes |
| --- | --- | --- | --- |
| AVPlayerRateDidChangeNotification | Constant | AVPlayer.h | No public AVPlayer rate-change notification/userInfo wrapper. |
| AVPlayerRateDidChangeReasonKey | Constant | AVPlayer.h | No public AVPlayer rate-change notification/userInfo wrapper. |
| AVPlayerRateDidChangeOriginatingParticipantKey | Constant | AVPlayer.h | No public AVPlayer rate-change notification/userInfo wrapper. |
| AVPlayerRateDidChangeReason | Typedef | AVPlayer.h | No public AVPlayer rate-change notification/userInfo wrapper. |
| AVPlayerRateDidChangeReasonSetRateCalled | Constant | AVPlayer.h | No public AVPlayer rate-change notification/userInfo wrapper. |
| AVPlayerRateDidChangeReasonSetRateFailed | Constant | AVPlayer.h | No public AVPlayer rate-change notification/userInfo wrapper. |
| AVPlayerRateDidChangeReasonAudioSessionInterrupted | Constant | AVPlayer.h | No public AVPlayer rate-change notification/userInfo wrapper. |
| AVPlayerRateDidChangeReasonAppBackgrounded | Constant | AVPlayer.h | No public AVPlayer rate-change notification/userInfo wrapper. |
| AVPlayerWaitingToMinimizeStallsReason | Constant | AVPlayer.h | Waiting reasons are returned as raw strings only; named constants are not exported. |
| AVPlayerWaitingWhileEvaluatingBufferingRateReason | Constant | AVPlayer.h | Waiting reasons are returned as raw strings only; named constants are not exported. |
| AVPlayerWaitingWithNoItemToPlayReason | Constant | AVPlayer.h | Waiting reasons are returned as raw strings only; named constants are not exported. |
| AVPlayerWaitingForCoordinatedPlaybackReason | Constant | AVPlayer.h | Waiting reasons are returned as raw strings only; named constants are not exported. |
| AVPlayerEligibleForHDRPlaybackDidChangeNotification | Constant | AVPlayer.h | No HDR-playback notification surface. |
| AVPlayerAudiovisualBackgroundPlaybackPolicy | Typedef | AVPlayer.h | Playback-policy / network-priority APIs are not wrapped. |
| AVPlayerNetworkResourcePriority | Typedef | AVPlayer.h | Playback-policy / network-priority APIs are not wrapped. |
| AVPlayerItemTimeJumpedNotification | Constant | AVPlayerItem.h | PlayerItem::observe() does not surface this notification or key. |
| AVPlayerItemFailedToPlayToEndTimeNotification | Constant | AVPlayerItem.h | PlayerItem::observe() does not surface this notification or key. |
| AVPlayerItemRecommendedTimeOffsetFromLiveDidChangeNotification | Constant | AVPlayerItem.h | PlayerItem::observe() does not surface this notification or key. |
| AVPlayerItemFailedToPlayToEndTimeErrorKey | Constant | AVPlayerItem.h | PlayerItem::observe() does not surface this notification or key. |
| AVPlayerItemTimeJumpedOriginatingParticipantKey | Constant | AVPlayerItem.h | PlayerItem::observe() does not surface this notification or key. |
| AVVideoCompositing | Protocol | AVPlayerItem.h | No video-compositing / custom compositor binding. |
| AVVariantPreferences | Typedef | AVPlayerItem.h | Variant-preferences flags are not wrapped. |
| AVPlayerItemOutput | Interface | AVPlayerItemOutput.h | Only concrete output subclasses are wrapped; the abstract base class is not exposed. |
| AVPlayerItemOutputPullDelegate | Protocol | AVPlayerItemOutput.h | Delegate protocol callbacks are not exposed. |
| AVPlayerItemLegibleOutputPushDelegate | Protocol | AVPlayerItemOutput.h | Delegate protocol callbacks are not exposed. |
| AVPlayerItemLegibleOutputTextStylingResolution | Typedef | AVPlayerItemOutput.h | Legible-output text-styling-resolution APIs are not exposed. |
| AVPlayerItemLegibleOutputTextStylingResolutionDefault | Constant | AVPlayerItemOutput.h | Legible-output text-styling-resolution APIs are not exposed. |
| AVPlayerItemLegibleOutputTextStylingResolutionSourceAndRulesOnly | Constant | AVPlayerItemOutput.h | Legible-output text-styling-resolution APIs are not exposed. |
| AVPlayerItemOutputPushDelegate | Protocol | AVPlayerItemOutput.h | Delegate protocol callbacks are not exposed. |
| AVPlayerItemMetadataOutputPushDelegate | Protocol | AVPlayerItemOutput.h | Delegate protocol callbacks are not exposed. |
| AVPlayerItemTrackVideoFieldModeDeinterlaceFields | Constant | AVPlayerItemTrack.h | video_field_mode uses raw strings; the named constant is not exported. |
| AVContentAuthorizationStatus | Typedef | AVPlayerItemProtectedContentAdditions.h | Protected-content authorization APIs are not wrapped. |

## ⏭️ EXEMPT
No macOS-available deprecated top-level symbols in the audited header set.

| Symbol | Kind | Header | Reason | SDK attribute |
| --- | --- | --- | --- | --- |
