#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use crate::ffi::AVPValue;
use core::ffi::c_void;
use core::ptr;

use crate::asset::{Asset, UrlAsset};
use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::retained::retain_release_wrapper;
use crate::util::parse_value_and_free;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AssetPlaybackConfigurationOption {
    StereoVideo,
    StereoMultiviewVideo,
    SpatialVideo,
    NonRectilinearProjection,
    AppleImmersiveVideo,
    Unknown(String),
}

impl AssetPlaybackConfigurationOption {
    fn from_raw(raw: &str) -> Self {
        match raw {
            "stereo_video" => Self::StereoVideo,
            "stereo_multiview_video" => Self::StereoMultiviewVideo,
            "spatial_video" => Self::SpatialVideo,
            "non_rectilinear_projection" => Self::NonRectilinearProjection,
            "apple_immersive_video" => Self::AppleImmersiveVideo,
            other => Self::Unknown(other.to_owned()),
        }
    }
}

#[derive(Debug)]
pub struct AssetPlaybackAssistant {
    ptr: *mut c_void,
}

retain_release_wrapper!(AssetPlaybackAssistant, release = ffi::av_ns_object_release);

impl Asset {
    pub fn playback_assistant(&self) -> Result<AssetPlaybackAssistant, AVPlayerError> {
        let mut err: *mut AVPValue = ptr::null_mut();
        let ptr = unsafe { ffi::av_asset_playback_assistant_create(self.ptr, &raw mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(AssetPlaybackAssistant { ptr })
    }
}

impl UrlAsset {
    pub fn playback_assistant(&self) -> Result<AssetPlaybackAssistant, AVPlayerError> {
        self.asset.playback_assistant()
    }
}

impl AssetPlaybackAssistant {
    pub fn playback_configuration_options(
        &self,
    ) -> Result<Vec<AssetPlaybackConfigurationOption>, AVPlayerError> {
        let mut err: *mut AVPValue = ptr::null_mut();
        let value_ptr =
            unsafe { ffi::av_asset_playback_assistant_copy_options_value(self.ptr, &raw mut err) };
        if value_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        let raw: Vec<String> = parse_value_and_free(value_ptr)?;
        Ok(raw
            .into_iter()
            .map(|value| AssetPlaybackConfigurationOption::from_raw(&value))
            .collect())
    }
}
