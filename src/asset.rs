#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use std::ffi::{CStr, CString};
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::metadata::MetadataItem;
use crate::time::Time;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssetInfoPayload {
    url: Option<String>,
    duration: Time,
    metadata: Vec<MetadataItem>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TrackInfoPayload {
    track_id: i32,
    media_type: String,
    natural_size: Size,
    nominal_frame_rate: String,
    estimated_data_rate: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KeyLoadStatusPayload {
    key: String,
    status: i32,
    error_message: Option<String>,
}

/// Simplified media-type classification for asset tracks and reader outputs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MediaType {
    Audio,
    Video,
    Text,
    Subtitle,
    ClosedCaption,
    Metadata,
    Timecode,
    Muxed,
    DepthData,
    Unknown(String),
}

impl MediaType {
    #[must_use]
    pub(crate) fn from_raw(raw: &str) -> Self {
        match raw {
            "audio" => Self::Audio,
            "video" => Self::Video,
            "text" => Self::Text,
            "subtitle" => Self::Subtitle,
            "closed_caption" => Self::ClosedCaption,
            "metadata" => Self::Metadata,
            "timecode" => Self::Timecode,
            "muxed" => Self::Muxed,
            "depth_data" => Self::DepthData,
            other => Self::Unknown(other.to_owned()),
        }
    }
}

/// Serializable `CGSize` mirror.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct Size {
    /// Width in points / pixels depending on API context.
    pub width: f64,
    /// Height in points / pixels depending on API context.
    pub height: f64,
}

/// Per-key loading state returned by `AVAsynchronousKeyValueLoading`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum KeyValueStatus {
    Unknown,
    Loading,
    Loaded,
    Failed,
    Cancelled,
}

impl KeyValueStatus {
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            1 => Self::Loading,
            2 => Self::Loaded,
            3 => Self::Failed,
            4 => Self::Cancelled,
            _ => Self::Unknown,
        }
    }
}

/// Result for a single key passed to `load_values_asynchronously`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyLoadStatus {
    pub key: String,
    pub status: KeyValueStatus,
    pub error_message: Option<String>,
}

/// Safe wrapper around `AVAsset`.
pub struct Asset {
    pub(crate) ptr: *mut c_void,
}

impl Drop for Asset {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_asset_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl Asset {
    fn info(&self) -> Result<AssetInfoPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_asset_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Asset duration.
    pub fn duration(&self) -> Result<Time, AVPlayerError> {
        Ok(self.info()?.duration)
    }

    /// Static metadata attached to the asset.
    pub fn metadata(&self) -> Result<Vec<MetadataItem>, AVPlayerError> {
        Ok(self.info()?.metadata)
    }

    /// Asset URL when backed by `AVURLAsset`.
    pub fn url(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.url)
    }

    /// Query the current load status of a key without triggering loading.
    pub fn status_of_value(&self, key: &str) -> Result<KeyValueStatus, AVPlayerError> {
        let key = CString::new(key).map_err(|error| {
            AVPlayerError::InvalidArgument(format!("key contains NUL byte: {error}"))
        })?;
        let mut err: *mut c_char = ptr::null_mut();
        let raw = unsafe { ffi::av_asset_status_of_value(self.ptr, key.as_ptr(), &mut err) };
        if raw < 0 {
            return Err(unsafe { from_swift(raw, err) });
        }
        Ok(KeyValueStatus::from_raw(raw))
    }

    /// Trigger asynchronous loading for the given keys and wait for completion.
    pub fn load_values_asynchronously<I, S>(
        &self,
        keys: I,
    ) -> Result<Vec<KeyLoadStatus>, AVPlayerError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let keys = keys
            .into_iter()
            .map(|key| key.as_ref().to_owned())
            .collect::<Vec<_>>();
        let json = serde_json::to_string(&keys).map_err(|error| {
            AVPlayerError::InvalidArgument(format!("failed to encode keys: {error}"))
        })?;
        let json = CString::new(json).map_err(|error| {
            AVPlayerError::InvalidArgument(format!("keys JSON contains NUL byte: {error}"))
        })?;
        let mut err: *mut c_char = ptr::null_mut();
        let statuses_ptr =
            unsafe { ffi::av_asset_load_values_json(self.ptr, json.as_ptr(), 30, &mut err) };
        if statuses_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::LOAD_FAILED, err) });
        }
        let raw_statuses: Vec<KeyLoadStatusPayload> = parse_json_and_free(statuses_ptr)?;
        Ok(raw_statuses
            .into_iter()
            .map(|status| KeyLoadStatus {
                key: status.key,
                status: KeyValueStatus::from_raw(status.status),
                error_message: status.error_message,
            })
            .collect())
    }

    /// Enumerate all tracks owned by the asset.
    pub fn tracks(&self) -> Result<Vec<AssetTrack>, AVPlayerError> {
        let count = unsafe { ffi::av_asset_track_count(self.ptr) };
        if count < 0 {
            return Err(AVPlayerError::OperationFailed(format!(
                "track count unexpectedly negative: {count}"
            )));
        }

        let capacity = usize::try_from(count).map_err(|error| {
            AVPlayerError::OperationFailed(format!("invalid track count: {error}"))
        })?;
        let mut tracks = Vec::with_capacity(capacity);
        for index in 0..count {
            let ptr = unsafe { ffi::av_asset_copy_track_at_index(self.ptr, index) };
            if ptr.is_null() {
                return Err(AVPlayerError::OperationFailed(format!(
                    "bridge returned null track at index {index}"
                )));
            }
            tracks.push(AssetTrack { ptr });
        }
        Ok(tracks)
    }
}

/// `AVURLAsset` convenience wrapper around [`Asset`].
pub struct UrlAsset {
    pub(crate) asset: Asset,
}

impl UrlAsset {
    /// Create a URL asset from a filesystem path.
    pub fn from_file_path(path: impl AsRef<Path>) -> Result<Self, AVPlayerError> {
        let path = path
            .as_ref()
            .to_str()
            .ok_or_else(|| AVPlayerError::InvalidArgument("path is not valid UTF-8".into()))?;
        Self::from_raw_url(path, true)
    }

    /// Create a URL asset from a remote URL string.
    pub fn from_remote_url(url: impl AsRef<str>) -> Result<Self, AVPlayerError> {
        Self::from_raw_url(url.as_ref(), false)
    }

    pub(crate) fn from_raw_url_with_options(
        url: &str,
        is_file_url: bool,
        prefer_precise_duration_and_timing: bool,
    ) -> Result<Self, AVPlayerError> {
        let url = CString::new(url).map_err(|error| {
            AVPlayerError::InvalidArgument(format!("URL contains NUL byte: {error}"))
        })?;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_url_asset_create(
                url.as_ptr(),
                is_file_url,
                prefer_precise_duration_and_timing,
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::ASSET_CREATE_FAILED, err) });
        }
        Ok(Self {
            asset: Asset { ptr },
        })
    }

    fn from_raw_url(url: &str, is_file_url: bool) -> Result<Self, AVPlayerError> {
        Self::from_raw_url_with_options(url, is_file_url, true)
    }

    /// Borrow the underlying `AVAsset` wrapper.
    #[must_use]
    pub const fn as_asset(&self) -> &Asset {
        &self.asset
    }

    /// Consume `self`, yielding the underlying [`Asset`].
    #[must_use]
    pub fn into_asset(self) -> Asset {
        self.asset
    }

    /// URL string used to create the asset.
    pub fn url(&self) -> Result<String, AVPlayerError> {
        self.asset.url()?.ok_or_else(|| {
            AVPlayerError::OperationFailed("asset is not backed by AVURLAsset".into())
        })
    }

    /// Forwarding convenience for `AVAsynchronousKeyValueLoading`.
    pub fn load_values_asynchronously<I, S>(
        &self,
        keys: I,
    ) -> Result<Vec<KeyLoadStatus>, AVPlayerError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.asset.load_values_asynchronously(keys)
    }

    /// Forwarding convenience for the underlying asset duration.
    pub fn duration(&self) -> Result<Time, AVPlayerError> {
        self.asset.duration()
    }

    /// Forwarding convenience for the underlying asset metadata.
    pub fn metadata(&self) -> Result<Vec<MetadataItem>, AVPlayerError> {
        self.asset.metadata()
    }

    /// Forwarding convenience for the underlying asset track enumeration.
    pub fn tracks(&self) -> Result<Vec<AssetTrack>, AVPlayerError> {
        self.asset.tracks()
    }

    /// Forwarding convenience for per-key load status queries.
    pub fn status_of_value(&self, key: &str) -> Result<KeyValueStatus, AVPlayerError> {
        self.asset.status_of_value(key)
    }
}

/// Safe wrapper around `AVAssetTrack`.
pub struct AssetTrack {
    pub(crate) ptr: *mut c_void,
}

impl Drop for AssetTrack {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_asset_track_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

// SAFETY: These AVFoundation asset handles are safe to transfer across thread
// boundaries; method calls are internally dispatched safely.
unsafe impl Send for Asset {}
unsafe impl Send for UrlAsset {}
unsafe impl Send for AssetTrack {}

impl AssetTrack {
    fn info(&self) -> Result<TrackInfoPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_asset_track_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Persistent track identifier.
    pub fn track_id(&self) -> Result<i32, AVPlayerError> {
        Ok(self.info()?.track_id)
    }

    /// Track media type.
    pub fn media_type(&self) -> Result<MediaType, AVPlayerError> {
        Ok(MediaType::from_raw(&self.info()?.media_type))
    }

    /// Natural presentation size.
    pub fn natural_size(&self) -> Result<Size, AVPlayerError> {
        Ok(self.info()?.natural_size)
    }

    /// Nominal frame rate when video-bearing.
    pub fn nominal_frame_rate(&self) -> Result<Option<f32>, AVPlayerError> {
        self.info()?
            .nominal_frame_rate
            .parse::<f32>()
            .ok()
            .map_or_else(|| Ok(None), |value| Ok(Some(value)))
    }

    /// Estimated data rate in bits per second.
    pub fn estimated_data_rate(&self) -> Result<Option<f32>, AVPlayerError> {
        self.info()?
            .estimated_data_rate
            .parse::<f32>()
            .ok()
            .map_or_else(|| Ok(None), |value| Ok(Some(value)))
    }
}

fn parse_json_and_free<T: DeserializeOwned>(json_ptr: *mut c_char) -> Result<T, AVPlayerError> {
    let json = unsafe { CStr::from_ptr(json_ptr) }
        .to_string_lossy()
        .into_owned();
    unsafe { ffi::avp_string_free(json_ptr) };
    serde_json::from_str::<T>(&json).map_err(|error| {
        AVPlayerError::OperationFailed(format!("failed to decode bridge JSON: {error}"))
    })
}
