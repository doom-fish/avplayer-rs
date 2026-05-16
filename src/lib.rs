#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod asset;
mod error;
pub mod ffi;
mod metadata;
mod player;
mod player_item;
mod player_item_access_log;
mod player_item_error_log;
mod player_item_integrated_timeline;
mod player_item_legible_output;
mod player_item_metadata_collector;
mod player_item_metadata_output;
mod player_item_rendered_legible_output;
mod player_item_track;
mod player_item_video_output;
mod player_interstitial_event;
mod player_layer;
mod player_video_output;
mod player_looper;
mod player_media_selection_criteria;
mod queue_player;
mod reader;
mod time;
mod url_asset;
mod util;

pub use asset::{Asset, AssetTrack, KeyLoadStatus, KeyValueStatus, MediaType, Size, UrlAsset};
pub use error::AVPlayerError;
pub use metadata::MetadataItem;
pub use player::{
    BoundaryTimeObserver, PeriodicTimeObserver, Player, PlayerItem, PlayerItemEvent,
    PlayerItemObserver, PlayerItemStatus, PlayerStatus,
};
pub use player_item::AudioTimePitchAlgorithm;
pub use player_item_access_log::{PlayerItemAccessLog, PlayerItemAccessLogEvent};
pub use player_item_error_log::{PlayerItemErrorLog, PlayerItemErrorLogEvent};
pub use player_item_integrated_timeline::{
    player_integrated_timeline_snapshots_out_of_sync_notification,
    player_integrated_timeline_snapshots_out_of_sync_reason_current_segment_changed,
    player_integrated_timeline_snapshots_out_of_sync_reason_key,
    player_integrated_timeline_snapshots_out_of_sync_reason_loaded_time_ranges_changed,
    player_integrated_timeline_snapshots_out_of_sync_reason_segments_changed,
    PlayerIntegratedTimelineOutOfSyncEvent,
    PlayerIntegratedTimelineSnapshotsOutOfSyncReason, PlayerItemIntegratedTimeline,
    PlayerItemIntegratedTimelineInfo, PlayerItemIntegratedTimelineObserver,
    PlayerItemIntegratedTimelineSegment, PlayerItemIntegratedTimelineSegmentInfo,
    PlayerItemIntegratedTimelineSnapshot, PlayerItemIntegratedTimelineSnapshotInfo,
    PlayerItemSegmentType,
};
pub use player_item_legible_output::PlayerItemLegibleOutput;
pub use player_item_metadata_collector::{
    DateRangeMetadataGroup, MetadataCollectorEvent, MetadataCollectorObserver,
    PlayerItemMediaDataCollectorInfo, PlayerItemMediaDataCollectorKind,
    PlayerItemMetadataCollector,
};
pub use player_item_metadata_output::PlayerItemMetadataOutput;
pub use player_item_rendered_legible_output::{
    PlayerItemRenderedLegibleOutput, RenderedCaptionImage, RenderedLegibleOutputEvent,
    RenderedLegibleOutputObserver,
};
pub use player_item_track::PlayerItemTrack;
pub use player_item_video_output::{PlayerItemVideoOutput, PlayerItemVideoOutputSettings};
pub use player_interstitial_event::{
    player_waiting_during_interstitial_event_reason, PlayerInterstitialEvent,
    PlayerInterstitialEventAssetListResponseStatus, PlayerInterstitialEventController,
    PlayerInterstitialEventCue, PlayerInterstitialEventInfo, PlayerInterstitialEventMonitor,
    PlayerInterstitialEventMonitorEvent, PlayerInterstitialEventMonitorObserver,
    PlayerInterstitialEventMonitorState, PlayerInterstitialEventRestrictions,
    PlayerInterstitialEventSkippableEventState, PlayerInterstitialEventTimelineOccupancy,
};
pub use player_layer::{PlayerLayer, Rect, VideoGravity};
pub use player_looper::{PlayerLooper, PlayerLooperItemOrdering, PlayerLooperStatus};
pub use player_media_selection_criteria::{
    MediaCharacteristic, PlayerActionAtItemEnd, PlayerMediaSelectionCriteria,
    PlayerTimeControlStatus,
};
pub use player_video_output::{
    AffineTransform, PlayerVideoOutput, PlayerVideoOutputConfiguration,
    PlayerVideoOutputSample, PlayerVideoOutputSettings, PlayerVideoOutputTagCollection,
    PlayerVideoOutputTagCollectionPreset, PlayerVideoTaggedBuffer,
    PlayerVideoTaggedBufferKind, VideoOutputSpecification,
};
pub use queue_player::QueuePlayer;
pub use reader::{
    AssetReader, AssetReaderAudioMixOutput, AssetReaderStatus, AssetReaderTrackOutput,
    AssetReaderVideoCompositionOutput, AudioOutputSettings, VideoOutputSettings,
};
pub use time::{Time, TimeRange};
pub use url_asset::UrlAssetOptions;

/// Common imports.
pub mod prelude {
    pub use crate::asset::{
        Asset, AssetTrack, KeyLoadStatus, KeyValueStatus, MediaType, Size, UrlAsset,
    };
    pub use crate::error::AVPlayerError;
    pub use crate::metadata::MetadataItem;
    pub use crate::player::{
        BoundaryTimeObserver, PeriodicTimeObserver, Player, PlayerItem, PlayerItemEvent,
        PlayerItemObserver, PlayerItemStatus, PlayerStatus,
    };
    pub use crate::player_item::AudioTimePitchAlgorithm;
    pub use crate::player_item_access_log::{PlayerItemAccessLog, PlayerItemAccessLogEvent};
    pub use crate::player_item_error_log::{PlayerItemErrorLog, PlayerItemErrorLogEvent};
    pub use crate::player_item_integrated_timeline::{
        player_integrated_timeline_snapshots_out_of_sync_notification,
        player_integrated_timeline_snapshots_out_of_sync_reason_current_segment_changed,
        player_integrated_timeline_snapshots_out_of_sync_reason_key,
        player_integrated_timeline_snapshots_out_of_sync_reason_loaded_time_ranges_changed,
        player_integrated_timeline_snapshots_out_of_sync_reason_segments_changed,
        PlayerIntegratedTimelineOutOfSyncEvent,
        PlayerIntegratedTimelineSnapshotsOutOfSyncReason, PlayerItemIntegratedTimeline,
        PlayerItemIntegratedTimelineInfo, PlayerItemIntegratedTimelineObserver,
        PlayerItemIntegratedTimelineSegment, PlayerItemIntegratedTimelineSegmentInfo,
        PlayerItemIntegratedTimelineSnapshot, PlayerItemIntegratedTimelineSnapshotInfo,
        PlayerItemSegmentType,
    };
    pub use crate::player_item_legible_output::PlayerItemLegibleOutput;
    pub use crate::player_item_metadata_collector::{
        DateRangeMetadataGroup, MetadataCollectorEvent, MetadataCollectorObserver,
        PlayerItemMediaDataCollectorInfo, PlayerItemMediaDataCollectorKind,
        PlayerItemMetadataCollector,
    };
    pub use crate::player_item_metadata_output::PlayerItemMetadataOutput;
    pub use crate::player_item_rendered_legible_output::{
        PlayerItemRenderedLegibleOutput, RenderedCaptionImage, RenderedLegibleOutputEvent,
        RenderedLegibleOutputObserver,
    };
    pub use crate::player_item_track::PlayerItemTrack;
    pub use crate::player_item_video_output::{
        PlayerItemVideoOutput, PlayerItemVideoOutputSettings,
    };
    pub use crate::player_interstitial_event::{
        player_waiting_during_interstitial_event_reason, PlayerInterstitialEvent,
        PlayerInterstitialEventAssetListResponseStatus, PlayerInterstitialEventController,
        PlayerInterstitialEventCue, PlayerInterstitialEventInfo, PlayerInterstitialEventMonitor,
        PlayerInterstitialEventMonitorEvent, PlayerInterstitialEventMonitorObserver,
        PlayerInterstitialEventMonitorState, PlayerInterstitialEventRestrictions,
        PlayerInterstitialEventSkippableEventState, PlayerInterstitialEventTimelineOccupancy,
    };
    pub use crate::player_layer::{PlayerLayer, Rect, VideoGravity};
    pub use crate::player_looper::{PlayerLooper, PlayerLooperItemOrdering, PlayerLooperStatus};
    pub use crate::player_media_selection_criteria::{
        MediaCharacteristic, PlayerActionAtItemEnd, PlayerMediaSelectionCriteria,
        PlayerTimeControlStatus,
    };
    pub use crate::player_video_output::{
        AffineTransform, PlayerVideoOutput, PlayerVideoOutputConfiguration,
        PlayerVideoOutputSample, PlayerVideoOutputSettings, PlayerVideoOutputTagCollection,
        PlayerVideoOutputTagCollectionPreset, PlayerVideoTaggedBuffer,
        PlayerVideoTaggedBufferKind, VideoOutputSpecification,
    };
    pub use crate::queue_player::QueuePlayer;
    pub use crate::reader::{
        AssetReader, AssetReaderAudioMixOutput, AssetReaderStatus, AssetReaderTrackOutput,
        AssetReaderVideoCompositionOutput, AudioOutputSettings, VideoOutputSettings,
    };
    pub use crate::time::{Time, TimeRange};
    pub use crate::url_asset::UrlAssetOptions;
}
