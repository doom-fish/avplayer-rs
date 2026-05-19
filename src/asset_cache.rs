#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;

use serde::Deserialize;

use crate::asset::UrlAsset;
use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::media_selection::{MediaSelectionGroup, MediaSelectionOption};
use crate::util::parse_json_and_free;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssetCacheInfoPayload {
    playable_offline: bool,
}

#[derive(Debug)]
pub struct AssetCache {
    ptr: *mut c_void,
}

impl Drop for AssetCache {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl UrlAsset {
    pub fn asset_cache(&self) -> Option<AssetCache> {
        let ptr = unsafe { ffi::av_url_asset_copy_asset_cache(self.asset.ptr) };
        if ptr.is_null() {
            None
        } else {
            Some(AssetCache { ptr })
        }
    }
}

impl AssetCache {
    fn info(&self) -> Result<AssetCacheInfoPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_asset_cache_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn is_playable_offline(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.playable_offline)
    }

    pub fn media_selection_options_in_group(
        &self,
        group: &MediaSelectionGroup,
    ) -> Result<Vec<MediaSelectionOption>, AVPlayerError> {
        let count =
            unsafe { ffi::av_asset_cache_media_selection_option_count(self.ptr, group.ptr) };
        if count < 0 {
            return Err(AVPlayerError::OperationFailed(format!(
                "invalid asset-cache media-selection option count: {count}"
            )));
        }
        let count = usize::try_from(count).map_err(|error| {
            AVPlayerError::OperationFailed(format!(
                "invalid asset-cache media-selection option count: {error}"
            ))
        })?;
        let mut options = Vec::with_capacity(count);
        for index in 0..count {
            let ptr = unsafe {
                ffi::av_asset_cache_copy_media_selection_option_at_index(
                    self.ptr,
                    group.ptr,
                    i32::try_from(index).unwrap_or(i32::MAX),
                )
            };
            if ptr.is_null() {
                return Err(AVPlayerError::OperationFailed(format!(
                    "bridge returned null media-selection option at index {index}"
                )));
            }
            options.push(MediaSelectionOption { ptr });
        }
        Ok(options)
    }
}
