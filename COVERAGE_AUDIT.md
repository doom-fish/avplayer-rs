# avplayer-rs coverage audit (vs MacOSX26.2.sdk)

SDK_PUBLIC_SYMBOLS: 109
VERIFIED: 26
GAPS: 83
EXEMPT: 0
COVERAGE_PCT: 23.85%

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
| AVPlayerItemRenderedLegibleOutputPushDelegate | Protocol | AVPlayerItemOutput.h | Delegate protocol callbacks are not exposed. |
| AVPlayerItemRenderedLegibleOutput | Interface | AVPlayerItemOutput.h | Rendered legible output is not wrapped. |
| AVPlayerItemTrackVideoFieldModeDeinterlaceFields | Constant | AVPlayerItemTrack.h | video_field_mode uses raw strings; the named constant is not exported. |
| AVPlayerVideoOutput | Interface | AVPlayerOutput.h | The newer AVPlayerVideoOutput pipeline is not wrapped. |
| CMTagCollectionVideoOutputPreset | Typedef | AVPlayerOutput.h | The newer AVPlayerVideoOutput pipeline is not wrapped. |
| CMTagCollectionCreateWithVideoOutputPreset | Function | AVPlayerOutput.h | The newer AVPlayerVideoOutput pipeline is not wrapped. |
| AVVideoOutputSpecification | Interface | AVPlayerOutput.h | The newer AVPlayerVideoOutput pipeline is not wrapped. |
| AVPlayerVideoOutputConfiguration | Interface | AVPlayerOutput.h | The newer AVPlayerVideoOutput pipeline is not wrapped. |
| AVPlayerInterstitialEventRestrictions | Typedef | AVPlayerInterstitialEventController.h | Interstitial-event/controller APIs are not wrapped. |
| AVPlayerInterstitialEventNoCue | Constant | AVPlayerInterstitialEventController.h | Interstitial-event/controller APIs are not wrapped. |
| AVPlayerInterstitialEventJoinCue | Constant | AVPlayerInterstitialEventController.h | Interstitial-event/controller APIs are not wrapped. |
| AVPlayerInterstitialEventLeaveCue | Constant | AVPlayerInterstitialEventController.h | Interstitial-event/controller APIs are not wrapped. |
| AVPlayerInterstitialEventTimelineOccupancy | Typedef | AVPlayerInterstitialEventController.h | Interstitial-event/controller APIs are not wrapped. |
| AVPlayerInterstitialEvent | Interface | AVPlayerInterstitialEventController.h | Interstitial-event/controller APIs are not wrapped. |
| AVPlayerInterstitialEventAssetListResponseStatus | Typedef | AVPlayerInterstitialEventController.h | Interstitial-event/controller APIs are not wrapped. |
| AVPlayerInterstitialEventSkippableEventState | Typedef | AVPlayerInterstitialEventController.h | Interstitial-event/controller APIs are not wrapped. |
| AVPlayerInterstitialEventMonitor | Interface | AVPlayerInterstitialEventController.h | Interstitial-event/controller APIs are not wrapped. |
| AVPlayerInterstitialEventMonitorEventsDidChangeNotification | Constant | AVPlayerInterstitialEventController.h | Interstitial-event/controller APIs are not wrapped. |
| AVPlayerInterstitialEventMonitorCurrentEventDidChangeNotification | Constant | AVPlayerInterstitialEventController.h | Interstitial-event/controller APIs are not wrapped. |
| AVPlayerInterstitialEventMonitorAssetListResponseStatusDidChangeNotification | Constant | AVPlayerInterstitialEventController.h | Interstitial-event/controller APIs are not wrapped. |
| AVPlayerInterstitialEventMonitorAssetListResponseStatusDidChangeEventKey | Constant | AVPlayerInterstitialEventController.h | Interstitial-event/controller APIs are not wrapped. |
| AVPlayerInterstitialEventMonitorAssetListResponseStatusDidChangeStatusKey | Constant | AVPlayerInterstitialEventController.h | Interstitial-event/controller APIs are not wrapped. |
| AVPlayerInterstitialEventMonitorAssetListResponseStatusDidChangeErrorKey | Constant | AVPlayerInterstitialEventController.h | Interstitial-event/controller APIs are not wrapped. |
| AVPlayerInterstitialEventMonitorCurrentEventSkippableStateDidChangeNotification | Constant | AVPlayerInterstitialEventController.h | Interstitial-event/controller APIs are not wrapped. |
| AVPlayerInterstitialEventMonitorCurrentEventSkippableStateDidChangeEventKey | Constant | AVPlayerInterstitialEventController.h | Interstitial-event/controller APIs are not wrapped. |
| AVPlayerInterstitialEventMonitorCurrentEventSkippableStateDidChangeStateKey | Constant | AVPlayerInterstitialEventController.h | Interstitial-event/controller APIs are not wrapped. |
| AVPlayerInterstitialEventMonitorCurrentEventSkippableStateDidChangeSkipControlLabelKey | Constant | AVPlayerInterstitialEventController.h | Interstitial-event/controller APIs are not wrapped. |
| AVPlayerInterstitialEventMonitorCurrentEventSkippedNotification | Constant | AVPlayerInterstitialEventController.h | Interstitial-event/controller APIs are not wrapped. |
| AVPlayerInterstitialEventMonitorCurrentEventSkippedEventKey | Constant | AVPlayerInterstitialEventController.h | Interstitial-event/controller APIs are not wrapped. |
| AVPlayerInterstitialEventMonitorInterstitialEventWasUnscheduledNotification | Constant | AVPlayerInterstitialEventController.h | Interstitial-event/controller APIs are not wrapped. |
| AVPlayerInterstitialEventMonitorInterstitialEventWasUnscheduledEventKey | Constant | AVPlayerInterstitialEventController.h | Interstitial-event/controller APIs are not wrapped. |
| AVPlayerInterstitialEventMonitorInterstitialEventWasUnscheduledErrorKey | Constant | AVPlayerInterstitialEventController.h | Interstitial-event/controller APIs are not wrapped. |
| AVPlayerInterstitialEventMonitorInterstitialEventDidFinishNotification | Constant | AVPlayerInterstitialEventController.h | Interstitial-event/controller APIs are not wrapped. |
| AVPlayerInterstitialEventMonitorInterstitialEventDidFinishEventKey | Constant | AVPlayerInterstitialEventController.h | Interstitial-event/controller APIs are not wrapped. |
| AVPlayerInterstitialEventMonitorInterstitialEventDidFinishPlayoutTimeKey | Constant | AVPlayerInterstitialEventController.h | Interstitial-event/controller APIs are not wrapped. |
| AVPlayerInterstitialEventMonitorInterstitialEventDidFinishDidPlayEntireEventKey | Constant | AVPlayerInterstitialEventController.h | Interstitial-event/controller APIs are not wrapped. |
| AVPlayerInterstitialEventController | Interface | AVPlayerInterstitialEventController.h | Interstitial-event/controller APIs are not wrapped. |
| AVPlayerWaitingDuringInterstitialEventReason | Constant | AVPlayerInterstitialEventController.h | Interstitial-event/controller APIs are not wrapped. |
| AVPlayerItemSegmentType | Typedef | AVPlayerItemIntegratedTimeline.h | Integrated timeline APIs are not wrapped. |
| AVPlayerItemSegment | Interface | AVPlayerItemIntegratedTimeline.h | Integrated timeline APIs are not wrapped. |
| AVPlayerItemIntegratedTimelineSnapshot | Interface | AVPlayerItemIntegratedTimeline.h | Integrated timeline APIs are not wrapped. |
| AVPlayerItemIntegratedTimeline | Interface | AVPlayerItemIntegratedTimeline.h | Integrated timeline APIs are not wrapped. |
| AVPlayerItemIntegratedTimelineObserver | Protocol | AVPlayerItemIntegratedTimeline.h | Integrated timeline APIs are not wrapped. |
| AVPlayerIntegratedTimelineSnapshotsOutOfSyncNotification | Constant | AVPlayerItemIntegratedTimeline.h | Integrated timeline APIs are not wrapped. |
| AVPlayerIntegratedTimelineSnapshotsOutOfSyncReasonKey | Constant | AVPlayerItemIntegratedTimeline.h | Integrated timeline APIs are not wrapped. |
| AVPlayerIntegratedTimelineSnapshotsOutOfSyncReason | Typedef | AVPlayerItemIntegratedTimeline.h | Integrated timeline APIs are not wrapped. |
| AVPlayerIntegratedTimelineSnapshotsOutOfSyncReasonSegmentsChanged | Constant | AVPlayerItemIntegratedTimeline.h | Integrated timeline APIs are not wrapped. |
| AVPlayerIntegratedTimelineSnapshotsOutOfSyncReasonCurrentSegmentChanged | Constant | AVPlayerItemIntegratedTimeline.h | Integrated timeline APIs are not wrapped. |
| AVPlayerIntegratedTimelineSnapshotsOutOfSyncReasonLoadedTimeRangesChanged | Constant | AVPlayerItemIntegratedTimeline.h | Integrated timeline APIs are not wrapped. |
| AVPlayerItemMediaDataCollector | Interface | AVPlayerItemMediaDataCollector.h | Media-data collectors and delegates are not wrapped. |
| AVPlayerItemMetadataCollectorPushDelegate | Protocol | AVPlayerItemMediaDataCollector.h | Delegate protocol callbacks are not exposed. |
| AVPlayerItemMetadataCollector | Interface | AVPlayerItemMediaDataCollector.h | Media-data collectors and delegates are not wrapped. |
| AVContentAuthorizationStatus | Typedef | AVPlayerItemProtectedContentAdditions.h | Protected-content authorization APIs are not wrapped. |

## ⏭️ EXEMPT
No macOS-available deprecated top-level symbols in the audited header set.

| Symbol | Kind | Header | Reason | SDK attribute |
| --- | --- | --- | --- | --- |
