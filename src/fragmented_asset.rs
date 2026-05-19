#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use std::ffi::CString;
use std::path::Path;

use serde::Deserialize;

use crate::asset::{Asset, AssetTrack, UrlAsset};
use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::util::parse_json_and_free;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FragmentedAssetMinderPayload {
    minding_interval: f64,
    asset_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MediaExtensionPropertiesPayload {
    containing_bundle_name: Option<String>,
    containing_bundle_url: Option<String>,
    extension_identifier: Option<String>,
    extension_name: Option<String>,
    extension_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaExtensionPropertiesInfo {
    pub containing_bundle_name: Option<String>,
    pub containing_bundle_url: Option<String>,
    pub extension_identifier: Option<String>,
    pub extension_name: Option<String>,
    pub extension_url: Option<String>,
}

impl From<MediaExtensionPropertiesPayload> for MediaExtensionPropertiesInfo {
    fn from(value: MediaExtensionPropertiesPayload) -> Self {
        Self {
            containing_bundle_name: value.containing_bundle_name,
            containing_bundle_url: value.containing_bundle_url,
            extension_identifier: value.extension_identifier,
            extension_name: value.extension_name,
            extension_url: value.extension_url,
        }
    }
}

#[derive(Debug)]
pub struct MediaExtensionProperties {
    ptr: *mut c_void,
}

impl Drop for MediaExtensionProperties {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl MediaExtensionProperties {
    const fn from_ptr(ptr: *mut c_void) -> Self {
        Self { ptr }
    }

    pub fn info(&self) -> Result<MediaExtensionPropertiesInfo, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_media_extension_properties_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(MediaExtensionPropertiesInfo::from(parse_json_and_free::<
            MediaExtensionPropertiesPayload,
        >(json_ptr)?))
    }

    pub fn containing_bundle_name(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.containing_bundle_name)
    }

    pub fn containing_bundle_url(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.containing_bundle_url)
    }

    pub fn extension_identifier(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.extension_identifier)
    }

    pub fn extension_name(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.extension_name)
    }

    pub fn extension_url(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.extension_url)
    }
}

#[derive(Debug)]
pub struct FragmentedAsset {
    asset: Asset,
}

impl FragmentedAsset {
    pub fn from_file_path(path: impl AsRef<Path>) -> Result<Self, AVPlayerError> {
        let path = path
            .as_ref()
            .to_str()
            .ok_or_else(|| AVPlayerError::InvalidArgument("path is not valid UTF-8".into()))?;
        Self::from_raw_url(path, true)
    }

    pub fn from_remote_url(url: impl AsRef<str>) -> Result<Self, AVPlayerError> {
        Self::from_raw_url(url.as_ref(), false)
    }

    fn from_raw_url(url: &str, is_file_url: bool) -> Result<Self, AVPlayerError> {
        let url = CString::new(url).map_err(|error| {
            AVPlayerError::InvalidArgument(format!("URL contains NUL byte: {error}"))
        })?;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::av_fragmented_asset_create(url.as_ptr(), is_file_url, &mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::ASSET_CREATE_FAILED, err) });
        }
        Ok(Self {
            asset: Asset { ptr },
        })
    }

    pub const fn as_asset(&self) -> &Asset {
        &self.asset
    }

    pub fn into_asset(self) -> Asset {
        self.asset
    }

    pub fn url(&self) -> Result<Option<String>, AVPlayerError> {
        self.asset.url()
    }

    pub fn tracks(&self) -> Result<Vec<FragmentedAssetTrack>, AVPlayerError> {
        let count = unsafe { ffi::av_fragmented_asset_track_count(self.asset.ptr) };
        if count < 0 {
            return Err(AVPlayerError::OperationFailed(format!(
                "fragmented track count unexpectedly negative: {count}"
            )));
        }
        let capacity = usize::try_from(count).map_err(|error| {
            AVPlayerError::OperationFailed(format!("invalid fragmented track count: {error}"))
        })?;
        let mut tracks = Vec::with_capacity(capacity);
        for index in 0..count {
            let ptr =
                unsafe { ffi::av_fragmented_asset_copy_track_at_index(self.asset.ptr, index) };
            if ptr.is_null() {
                return Err(AVPlayerError::OperationFailed(format!(
                    "bridge returned null fragmented track at index {index}"
                )));
            }
            tracks.push(FragmentedAssetTrack::from_ptr(ptr));
        }
        Ok(tracks)
    }

    pub fn track_with_id(&self, track_id: i32) -> Option<FragmentedAssetTrack> {
        let ptr = unsafe { ffi::av_fragmented_asset_copy_track_with_id(self.asset.ptr, track_id) };
        (!ptr.is_null()).then(|| FragmentedAssetTrack::from_ptr(ptr))
    }

    pub fn is_associated_with_minder(&self) -> bool {
        unsafe { ffi::av_fragmented_asset_is_associated_with_minder(self.asset.ptr) }
    }

    pub fn media_extension_properties(&self) -> Option<MediaExtensionProperties> {
        let ptr = unsafe { ffi::av_url_asset_copy_media_extension_properties(self.asset.ptr) };
        (!ptr.is_null()).then(|| MediaExtensionProperties::from_ptr(ptr))
    }

    pub fn expects_property_revised_notifications() -> bool {
        unsafe { ffi::av_fragmented_asset_expects_property_revised_notifications() }
    }

    pub fn is_playable_extended_mime_type(
        mime_type: impl AsRef<str>,
    ) -> Result<bool, AVPlayerError> {
        let mime_type = CString::new(mime_type.as_ref()).map_err(|error| {
            AVPlayerError::InvalidArgument(format!("MIME type contains NUL byte: {error}"))
        })?;
        Ok(unsafe { ffi::av_fragmented_asset_is_playable_extended_mime_type(mime_type.as_ptr()) })
    }
}

impl UrlAsset {
    pub fn media_extension_properties(&self) -> Option<MediaExtensionProperties> {
        let ptr = unsafe { ffi::av_url_asset_copy_media_extension_properties(self.asset.ptr) };
        (!ptr.is_null()).then(|| MediaExtensionProperties::from_ptr(ptr))
    }
}

#[derive(Debug)]
pub struct FragmentedAssetTrack {
    track: AssetTrack,
}

impl FragmentedAssetTrack {
    const fn from_ptr(ptr: *mut c_void) -> Self {
        Self {
            track: AssetTrack { ptr },
        }
    }

    pub const fn as_asset_track(&self) -> &AssetTrack {
        &self.track
    }

    pub fn segment_count(&self) -> Result<usize, AVPlayerError> {
        let count = unsafe { ffi::av_fragmented_asset_track_segment_count(self.track.ptr) };
        usize::try_from(count).map_err(|error| {
            AVPlayerError::OperationFailed(format!("invalid fragmented segment count: {error}"))
        })
    }

    pub fn track_id(&self) -> Result<i32, AVPlayerError> {
        self.track.track_id()
    }

    pub fn media_type(&self) -> Result<crate::asset::MediaType, AVPlayerError> {
        self.track.media_type()
    }

    pub fn natural_size(&self) -> Result<crate::asset::Size, AVPlayerError> {
        self.track.natural_size()
    }

    pub fn nominal_frame_rate(&self) -> Result<Option<f32>, AVPlayerError> {
        self.track.nominal_frame_rate()
    }

    pub fn estimated_data_rate(&self) -> Result<Option<f32>, AVPlayerError> {
        self.track.estimated_data_rate()
    }
}

#[derive(Debug)]
pub struct FragmentedAssetMinder {
    ptr: *mut c_void,
}

impl Drop for FragmentedAssetMinder {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl FragmentedAssetMinder {
    pub fn new(asset: &FragmentedAsset, minding_interval: f64) -> Result<Self, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_fragmented_asset_minder_create(asset.asset.ptr, minding_interval, &mut err)
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn info(&self) -> Result<FragmentedAssetMinderPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_fragmented_asset_minder_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn minding_interval(&self) -> Result<f64, AVPlayerError> {
        Ok(self.info()?.minding_interval)
    }

    pub fn asset_count(&self) -> Result<usize, AVPlayerError> {
        Ok(self.info()?.asset_count)
    }

    pub fn set_minding_interval(&self, minding_interval: f64) {
        unsafe { ffi::av_fragmented_asset_minder_set_interval(self.ptr, minding_interval) };
    }

    pub fn add_asset(&self, asset: &FragmentedAsset) {
        unsafe { ffi::av_fragmented_asset_minder_add_asset(self.ptr, asset.asset.ptr) };
    }

    pub fn remove_asset(&self, asset: &FragmentedAsset) {
        unsafe { ffi::av_fragmented_asset_minder_remove_asset(self.ptr, asset.asset.ptr) };
    }

    pub fn assets(&self) -> Result<Vec<FragmentedAsset>, AVPlayerError> {
        let count = self.asset_count()?;
        let mut assets = Vec::with_capacity(count);
        for index in 0..count {
            let index_i32 = i32::try_from(index).map_err(|error| {
                AVPlayerError::OperationFailed(format!("fragmented asset index overflow: {error}"))
            })?;
            let ptr =
                unsafe { ffi::av_fragmented_asset_minder_copy_asset_at_index(self.ptr, index_i32) };
            if ptr.is_null() {
                return Err(AVPlayerError::OperationFailed(format!(
                    "bridge returned null fragmented asset at index {index}"
                )));
            }
            assets.push(FragmentedAsset {
                asset: Asset { ptr },
            });
        }
        Ok(assets)
    }
}

unsafe impl Send for MediaExtensionProperties {}
unsafe impl Send for FragmentedAsset {}
unsafe impl Send for FragmentedAssetTrack {}
unsafe impl Send for FragmentedAssetMinder {}
