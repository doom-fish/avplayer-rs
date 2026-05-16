#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod asset;
mod error;
pub mod ffi;
mod metadata;
mod player;
mod reader;
mod time;

pub use asset::{Asset, AssetTrack, KeyLoadStatus, KeyValueStatus, MediaType, Size, UrlAsset};
pub use error::AVPlayerError;
pub use metadata::MetadataItem;
pub use player::{
    BoundaryTimeObserver, Player, PlayerItem, PlayerItemEvent, PlayerItemObserver,
    PlayerItemStatus, PlayerStatus, PeriodicTimeObserver,
};
pub use reader::{
    AssetReader, AssetReaderAudioMixOutput, AssetReaderStatus, AssetReaderTrackOutput,
    AssetReaderVideoCompositionOutput, AudioOutputSettings, VideoOutputSettings,
};
pub use time::{Time, TimeRange};

/// Common imports.
pub mod prelude {
    pub use crate::asset::{
        Asset, AssetTrack, KeyLoadStatus, KeyValueStatus, MediaType, Size, UrlAsset,
    };
    pub use crate::error::AVPlayerError;
    pub use crate::metadata::MetadataItem;
    pub use crate::player::{
        BoundaryTimeObserver, Player, PlayerItem, PlayerItemEvent, PlayerItemObserver,
        PlayerItemStatus, PlayerStatus, PeriodicTimeObserver,
    };
    pub use crate::reader::{
        AssetReader, AssetReaderAudioMixOutput, AssetReaderStatus, AssetReaderTrackOutput,
        AssetReaderVideoCompositionOutput, AudioOutputSettings, VideoOutputSettings,
    };
    pub use crate::time::{Time, TimeRange};
}
