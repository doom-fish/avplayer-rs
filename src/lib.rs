#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod asset;
mod asset_cache;
mod asset_download;
mod asset_extras;
mod asset_image_generator;
mod asset_playback_assistant;
mod asset_variant;
/// Groups `AVPlayer` framework constants for `async_api`.
#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub mod async_api;
mod content_key_session;
mod error;
/// Groups `AVPlayer` framework constants for `ffi`.
pub mod ffi;
mod fragmented_asset;
mod media_selection;
mod metadata;
mod metadata_groups;
mod player;
mod player_interstitial_event;
mod player_item;
mod player_item_access_log;
mod player_item_error_log;
mod player_item_integrated_timeline;
mod player_item_legible_output;
mod player_item_media_data_collector;
mod player_item_metadata_collector;
mod player_item_metadata_output;
mod player_item_output;
mod player_item_rendered_legible_output;
mod player_item_track;
mod player_item_video_output;
mod player_layer;
mod player_looper;
mod player_media_selection_criteria;
mod player_video_output;
mod queue_player;
mod reader;
mod reader_extras;
mod resource_loader;
mod sample_buffer_display_layer;
mod time;
mod url_asset;
mod util;

/// Re-exports the `AVPlayer` framework surface for this item.
pub use asset::{Asset, AssetTrack, KeyLoadStatus, KeyValueStatus, MediaType, Size, UrlAsset};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use asset_cache::AssetCache;
/// Re-exports the `AVPlayer` framework surface for this item.
pub use asset_download::{
    AggregateAssetDownloadTask, AssetDownloadConfiguration, AssetDownloadContentConfiguration,
    AssetDownloadDelegateEvent, AssetDownloadStorageManagementPolicy,
    AssetDownloadStorageManager, AssetDownloadTask, AssetDownloadTaskState,
    AssetDownloadURLSession, AssetDownloadedAssetEvictionPriority,
};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use asset_image_generator::{
    AssetImage, AssetImageGenerator, AssetImageGeneratorApertureMode,
    AssetImageGeneratorDynamicRangePolicy, GeneratedAssetImage,
};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use asset_playback_assistant::{AssetPlaybackAssistant, AssetPlaybackConfigurationOption};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use asset_variant::{
    AssetVariant, AssetVariantAudioAttributes, AssetVariantAudioRenditionSpecificAttributes,
    AssetVariantQualifier, AssetVariantVideoAttributes, AssetVariantVideoLayoutAttributes,
};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use content_key_session::{
    ContentKey, ContentKeyEvent, ContentKeyIdentifier, ContentKeyRequest,
    ContentKeyRequestOptions, ContentKeyRequestRetryReason, ContentKeyRequestStatus,
    ContentKeyResponse, ContentKeySession, ContentKeySessionEvent, ContentKeySessionEventStream,
    ContentKeySessionObserver, ContentKeySpecifier, ContentKeySystem,
    ExternalContentProtectionStatus, PersistableContentKeyRequest,
};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use error::AVPlayerError;
/// Re-exports the `AVPlayer` framework surface for this item.
pub use fragmented_asset::{
    FragmentedAsset, FragmentedAssetMinder, FragmentedAssetTrack, MediaExtensionProperties,
    MediaExtensionPropertiesInfo,
};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use media_selection::{
    CustomMediaSelectionScheme, MediaPresentationSelector, MediaPresentationSetting,
    MediaSelection, MediaSelectionGroup, MediaSelectionOption, MutableMediaSelection,
};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use metadata::MetadataItem;
/// Re-exports the `AVPlayer` framework surface for this item.
pub use metadata_groups::{
    DateRangeMetadataGroupHandle, MetadataGroup, MetadataItemFilter, MutableDateRangeMetadataGroup,
    MutableMetadataItem, MutableTimedMetadataGroup, TimedMetadataGroupHandle,
};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use player::{
    BoundaryTimeObserver, PeriodicTimeObserver, Player, PlayerItem, PlayerItemEvent,
    PlayerItemObserver, PlayerItemStatus, PlayerStatus,
};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use player_interstitial_event::{
    player_waiting_during_interstitial_event_reason, PlayerInterstitialEvent,
    PlayerInterstitialEventAssetListResponseStatus, PlayerInterstitialEventController,
    PlayerInterstitialEventCue, PlayerInterstitialEventInfo, PlayerInterstitialEventMonitor,
    PlayerInterstitialEventMonitorEvent, PlayerInterstitialEventMonitorObserver,
    PlayerInterstitialEventMonitorState, PlayerInterstitialEventRestrictions,
    PlayerInterstitialEventSkippableEventState, PlayerInterstitialEventTimelineOccupancy,
};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use player_item::{
    AudioTimePitchAlgorithm, ContentAuthorizationStatus, PlayerItemVideoCompositorInfo,
    VariantPreferences,
};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use player_item_access_log::{PlayerItemAccessLog, PlayerItemAccessLogEvent};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use player_item_error_log::{PlayerItemErrorLog, PlayerItemErrorLogEvent};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use player_item_integrated_timeline::{
    player_integrated_timeline_snapshots_out_of_sync_notification,
    player_integrated_timeline_snapshots_out_of_sync_reason_current_segment_changed,
    player_integrated_timeline_snapshots_out_of_sync_reason_key,
    player_integrated_timeline_snapshots_out_of_sync_reason_loaded_time_ranges_changed,
    player_integrated_timeline_snapshots_out_of_sync_reason_segments_changed,
    PlayerIntegratedTimelineOutOfSyncEvent, PlayerIntegratedTimelineSnapshotsOutOfSyncReason,
    PlayerItemIntegratedTimeline, PlayerItemIntegratedTimelineInfo,
    PlayerItemIntegratedTimelineObserver, PlayerItemIntegratedTimelineSegment,
    PlayerItemIntegratedTimelineSegmentInfo, PlayerItemIntegratedTimelineSnapshot,
    PlayerItemIntegratedTimelineSnapshotInfo, PlayerItemSegmentType,
};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use player_item_integrated_timeline::{
    PlayerItemIntegratedTimelineSegment as PlayerItemSegment,
    PlayerItemIntegratedTimelineSegmentInfo as PlayerItemSegmentInfo,
};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use player_item_legible_output::{
    PlayerItemLegibleOutput, PlayerItemLegibleOutputEvent, PlayerItemLegibleOutputObserver,
    PlayerItemLegibleOutputTextStylingResolution,
};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use player_item_media_data_collector::PlayerItemMediaDataCollector;
/// Re-exports the `AVPlayer` framework surface for this item.
pub use player_item_metadata_collector::{
    DateRangeMetadataGroup, MetadataCollectorEvent, MetadataCollectorObserver,
    PlayerItemMediaDataCollectorInfo, PlayerItemMediaDataCollectorKind,
    PlayerItemMetadataCollector,
};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use player_item_metadata_output::{
    MetadataOutputEvent, MetadataOutputObserver, PlayerItemMetadataOutput, TimedMetadataGroup,
};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use player_item_output::PlayerItemOutput;
/// Re-exports the `AVPlayer` framework surface for this item.
pub use player_item_rendered_legible_output::{
    PlayerItemRenderedLegibleOutput, RenderedCaptionImage, RenderedLegibleOutputEvent,
    RenderedLegibleOutputObserver,
};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use player_item_track::{PlayerItemTrack, PlayerItemTrackVideoFieldMode};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use player_item_video_output::{
    PlayerItemVideoOutput, PlayerItemVideoOutputEvent, PlayerItemVideoOutputObserver,
    PlayerItemVideoOutputSettings,
};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use player_layer::{PlayerLayer, Rect, VideoGravity};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use player_looper::{PlayerLooper, PlayerLooperItemOrdering, PlayerLooperStatus};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use player_media_selection_criteria::{
    player_eligible_for_hdr_playback_did_change_notification, MediaCharacteristic,
    PlayerActionAtItemEnd, PlayerAudiovisualBackgroundPlaybackPolicy, PlayerMediaSelectionCriteria,
    PlayerNetworkResourcePriority, PlayerRateDidChangeEvent, PlayerRateDidChangeObserver,
    PlayerRateDidChangeReason, PlayerTimeControlStatus, PlayerWaitingReason,
};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use player_video_output::{
    AffineTransform, PlayerVideoOutput, PlayerVideoOutputConfiguration, PlayerVideoOutputSample,
    PlayerVideoOutputSettings, PlayerVideoOutputTagCollection,
    PlayerVideoOutputTagCollectionPreset, PlayerVideoTaggedBuffer, PlayerVideoTaggedBufferKind,
    VideoOutputSpecification,
};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use queue_player::QueuePlayer;
/// Re-exports the `AVPlayer` framework surface for this item.
pub use reader::{
    AssetReader, AssetReaderAudioMixOutput, AssetReaderStatus, AssetReaderTrackOutput,
    AssetReaderVideoCompositionOutput, AudioOutputSettings, VideoOutputSettings,
};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use reader_extras::{
    AssetReaderOutput, AssetReaderOutputCaptionAdaptor, AssetReaderOutputMetadataAdaptor,
    AssetReaderSampleReferenceOutput, CaptionGroupInfo, CaptionValidationEvent,
    CaptionValidationObserver,
};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use resource_loader::{
    AssetResourceLoader, AssetResourceLoaderEvent, AssetResourceLoaderObserver,
    AssetResourceLoadingContentInformationRequest, AssetResourceLoadingDataRequest,
    AssetResourceLoadingRequest, AssetResourceLoadingRequestor, AssetResourceRenewalRequest,
};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use sample_buffer_display_layer::{
    QueuedSampleBufferRenderingStatus, SampleBufferDisplayLayer,
};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use time::{Time, TimeRange};
/// Re-exports the `AVPlayer` framework surface for this item.
pub use url_asset::UrlAssetOptions;

/// Common imports.
pub mod prelude {
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::asset::{
        Asset, AssetTrack, KeyLoadStatus, KeyValueStatus, MediaType, Size, UrlAsset,
    };
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::asset_cache::AssetCache;
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::asset_download::{
        AggregateAssetDownloadTask, AssetDownloadConfiguration,
        AssetDownloadContentConfiguration, AssetDownloadDelegateEvent,
        AssetDownloadStorageManagementPolicy, AssetDownloadStorageManager,
        AssetDownloadTask, AssetDownloadTaskState, AssetDownloadURLSession,
        AssetDownloadedAssetEvictionPriority,
    };
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::asset_image_generator::{
        AssetImage, AssetImageGenerator, AssetImageGeneratorApertureMode,
        AssetImageGeneratorDynamicRangePolicy, GeneratedAssetImage,
    };
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::asset_playback_assistant::{
        AssetPlaybackAssistant, AssetPlaybackConfigurationOption,
    };
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::asset_variant::{
        AssetVariant, AssetVariantAudioAttributes, AssetVariantAudioRenditionSpecificAttributes,
        AssetVariantQualifier, AssetVariantVideoAttributes, AssetVariantVideoLayoutAttributes,
    };
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::content_key_session::{
        ContentKey, ContentKeyEvent, ContentKeyIdentifier, ContentKeyRequest,
        ContentKeyRequestOptions, ContentKeyRequestRetryReason, ContentKeyRequestStatus,
        ContentKeyResponse, ContentKeySession, ContentKeySessionEvent,
        ContentKeySessionEventStream, ContentKeySessionObserver, ContentKeySpecifier,
        ContentKeySystem, ExternalContentProtectionStatus, PersistableContentKeyRequest,
    };
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::error::AVPlayerError;
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::fragmented_asset::{
        FragmentedAsset, FragmentedAssetMinder, FragmentedAssetTrack, MediaExtensionProperties,
        MediaExtensionPropertiesInfo,
    };
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::media_selection::{
        CustomMediaSelectionScheme, MediaPresentationSelector, MediaPresentationSetting,
        MediaSelection, MediaSelectionGroup, MediaSelectionOption, MutableMediaSelection,
    };
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::metadata::MetadataItem;
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::metadata_groups::{
        DateRangeMetadataGroupHandle, MetadataGroup, MetadataItemFilter,
        MutableDateRangeMetadataGroup, MutableMetadataItem, MutableTimedMetadataGroup,
        TimedMetadataGroupHandle,
    };
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::player::{
        BoundaryTimeObserver, PeriodicTimeObserver, Player, PlayerItem, PlayerItemEvent,
        PlayerItemObserver, PlayerItemStatus, PlayerStatus,
    };
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::player_interstitial_event::{
        player_waiting_during_interstitial_event_reason, PlayerInterstitialEvent,
        PlayerInterstitialEventAssetListResponseStatus, PlayerInterstitialEventController,
        PlayerInterstitialEventCue, PlayerInterstitialEventInfo, PlayerInterstitialEventMonitor,
        PlayerInterstitialEventMonitorEvent, PlayerInterstitialEventMonitorObserver,
        PlayerInterstitialEventMonitorState, PlayerInterstitialEventRestrictions,
        PlayerInterstitialEventSkippableEventState, PlayerInterstitialEventTimelineOccupancy,
    };
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::player_item::{
        AudioTimePitchAlgorithm, ContentAuthorizationStatus, PlayerItemVideoCompositorInfo,
        VariantPreferences,
    };
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::player_item_access_log::{PlayerItemAccessLog, PlayerItemAccessLogEvent};
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::player_item_error_log::{PlayerItemErrorLog, PlayerItemErrorLogEvent};
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::player_item_integrated_timeline::{
        player_integrated_timeline_snapshots_out_of_sync_notification,
        player_integrated_timeline_snapshots_out_of_sync_reason_current_segment_changed,
        player_integrated_timeline_snapshots_out_of_sync_reason_key,
        player_integrated_timeline_snapshots_out_of_sync_reason_loaded_time_ranges_changed,
        player_integrated_timeline_snapshots_out_of_sync_reason_segments_changed,
        PlayerIntegratedTimelineOutOfSyncEvent, PlayerIntegratedTimelineSnapshotsOutOfSyncReason,
        PlayerItemIntegratedTimeline, PlayerItemIntegratedTimelineInfo,
        PlayerItemIntegratedTimelineObserver, PlayerItemIntegratedTimelineSegment,
        PlayerItemIntegratedTimelineSegmentInfo, PlayerItemIntegratedTimelineSnapshot,
        PlayerItemIntegratedTimelineSnapshotInfo, PlayerItemSegmentType,
    };
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::player_item_integrated_timeline::{
        PlayerItemIntegratedTimelineSegment as PlayerItemSegment,
        PlayerItemIntegratedTimelineSegmentInfo as PlayerItemSegmentInfo,
    };
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::player_item_legible_output::{
        PlayerItemLegibleOutput, PlayerItemLegibleOutputEvent, PlayerItemLegibleOutputObserver,
        PlayerItemLegibleOutputTextStylingResolution,
    };
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::player_item_media_data_collector::PlayerItemMediaDataCollector;
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::player_item_metadata_collector::{
        DateRangeMetadataGroup, MetadataCollectorEvent, MetadataCollectorObserver,
        PlayerItemMediaDataCollectorInfo, PlayerItemMediaDataCollectorKind,
        PlayerItemMetadataCollector,
    };
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::player_item_metadata_output::{
        MetadataOutputEvent, MetadataOutputObserver, PlayerItemMetadataOutput, TimedMetadataGroup,
    };
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::player_item_output::PlayerItemOutput;
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::player_item_rendered_legible_output::{
        PlayerItemRenderedLegibleOutput, RenderedCaptionImage, RenderedLegibleOutputEvent,
        RenderedLegibleOutputObserver,
    };
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::player_item_track::{PlayerItemTrack, PlayerItemTrackVideoFieldMode};
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::player_item_video_output::{
        PlayerItemVideoOutput, PlayerItemVideoOutputEvent, PlayerItemVideoOutputObserver,
        PlayerItemVideoOutputSettings,
    };
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::player_layer::{PlayerLayer, Rect, VideoGravity};
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::player_looper::{PlayerLooper, PlayerLooperItemOrdering, PlayerLooperStatus};
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::player_media_selection_criteria::{
        player_eligible_for_hdr_playback_did_change_notification, MediaCharacteristic,
        PlayerActionAtItemEnd, PlayerAudiovisualBackgroundPlaybackPolicy,
        PlayerMediaSelectionCriteria, PlayerNetworkResourcePriority, PlayerRateDidChangeEvent,
        PlayerRateDidChangeObserver, PlayerRateDidChangeReason, PlayerTimeControlStatus,
        PlayerWaitingReason,
    };
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::player_video_output::{
        AffineTransform, PlayerVideoOutput, PlayerVideoOutputConfiguration,
        PlayerVideoOutputSample, PlayerVideoOutputSettings, PlayerVideoOutputTagCollection,
        PlayerVideoOutputTagCollectionPreset, PlayerVideoTaggedBuffer, PlayerVideoTaggedBufferKind,
        VideoOutputSpecification,
    };
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::queue_player::QueuePlayer;
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::reader::{
        AssetReader, AssetReaderAudioMixOutput, AssetReaderStatus, AssetReaderTrackOutput,
        AssetReaderVideoCompositionOutput, AudioOutputSettings, VideoOutputSettings,
    };
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::reader_extras::{
        AssetReaderOutput, AssetReaderOutputCaptionAdaptor, AssetReaderOutputMetadataAdaptor,
        AssetReaderSampleReferenceOutput, CaptionGroupInfo, CaptionValidationEvent,
        CaptionValidationObserver,
    };
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::resource_loader::{
        AssetResourceLoader, AssetResourceLoaderEvent, AssetResourceLoaderObserver,
        AssetResourceLoadingContentInformationRequest, AssetResourceLoadingDataRequest,
        AssetResourceLoadingRequest, AssetResourceLoadingRequestor, AssetResourceRenewalRequest,
    };
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::sample_buffer_display_layer::{
        QueuedSampleBufferRenderingStatus, SampleBufferDisplayLayer,
    };
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::time::{Time, TimeRange};
    /// Re-exports the `AVPlayer` framework surface for this item.
    pub use crate::url_asset::UrlAssetOptions;
}
