#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use std::ffi::CString;

use serde::Deserialize;

use crate::asset::{Asset, Size, UrlAsset};
use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::time::Time;
use crate::util::parse_json_and_free;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssetImageGeneratorInfoPayload {
    applies_preferred_track_transform: bool,
    maximum_size: Size,
    aperture_mode: Option<String>,
    requested_time_tolerance_before: Time,
    requested_time_tolerance_after: Time,
    dynamic_range_policy: Option<String>,
    has_custom_video_compositor: bool,
    custom_video_compositor_class_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssetImagePayload {
    width: usize,
    height: usize,
    bits_per_component: usize,
    bits_per_pixel: usize,
    bytes_per_row: usize,
    alpha_info: u32,
    bitmap_info: u32,
    rendering_intent: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AssetImageGeneratorApertureMode {
    CleanAperture,
    ProductionAperture,
    EncodedPixels,
    Unknown(String),
}

impl AssetImageGeneratorApertureMode {
    fn from_raw(raw: &str) -> Self {
        match raw {
            "clean_aperture" => Self::CleanAperture,
            "production_aperture" => Self::ProductionAperture,
            "encoded_pixels" => Self::EncodedPixels,
            other => Self::Unknown(other.to_owned()),
        }
    }

    fn as_raw(&self) -> &str {
        match self {
            Self::CleanAperture => "clean_aperture",
            Self::ProductionAperture => "production_aperture",
            Self::EncodedPixels => "encoded_pixels",
            Self::Unknown(raw) => raw,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AssetImageGeneratorDynamicRangePolicy {
    ForceSdr,
    MatchSource,
    Unknown(String),
}

impl AssetImageGeneratorDynamicRangePolicy {
    fn from_raw(raw: &str) -> Self {
        match raw {
            "force_sdr" => Self::ForceSdr,
            "match_source" => Self::MatchSource,
            other => Self::Unknown(other.to_owned()),
        }
    }

    fn as_raw(&self) -> &str {
        match self {
            Self::ForceSdr => "force_sdr",
            Self::MatchSource => "match_source",
            Self::Unknown(raw) => raw,
        }
    }
}

#[derive(Debug)]
pub struct AssetImageGenerator {
    ptr: *mut c_void,
}

impl Drop for AssetImageGenerator {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

#[derive(Debug)]
pub struct AssetImage {
    ptr: *mut c_void,
}

impl Drop for AssetImage {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

#[derive(Debug)]
pub struct GeneratedAssetImage {
    pub image: AssetImage,
    pub actual_time: Time,
}

impl Asset {
    pub fn image_generator(&self) -> AssetImageGenerator {
        let ptr = unsafe { ffi::av_asset_image_generator_create(self.ptr) };
        AssetImageGenerator { ptr }
    }
}

impl UrlAsset {
    pub fn image_generator(&self) -> AssetImageGenerator {
        self.asset.image_generator()
    }
}

impl AssetImageGenerator {
    fn info(&self) -> Result<AssetImageGeneratorInfoPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_asset_image_generator_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn applies_preferred_track_transform(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.applies_preferred_track_transform)
    }

    pub fn set_applies_preferred_track_transform(&self, applies: bool) {
        unsafe {
            ffi::av_asset_image_generator_set_applies_preferred_track_transform(self.ptr, applies);
        }
    }

    pub fn maximum_size(&self) -> Result<Size, AVPlayerError> {
        Ok(self.info()?.maximum_size)
    }

    pub fn set_maximum_size(&self, size: Size) {
        unsafe {
            ffi::av_asset_image_generator_set_maximum_size(self.ptr, size.width, size.height);
        }
    }

    pub fn aperture_mode(&self) -> Result<Option<AssetImageGeneratorApertureMode>, AVPlayerError> {
        Ok(self
            .info()?
            .aperture_mode
            .as_deref()
            .map(AssetImageGeneratorApertureMode::from_raw))
    }

    pub fn set_aperture_mode(
        &self,
        mode: Option<AssetImageGeneratorApertureMode>,
    ) -> Result<(), AVPlayerError> {
        let mode = mode
            .map(|mode| {
                CString::new(mode.as_raw()).map_err(|error| {
                    AVPlayerError::InvalidArgument(format!(
                        "aperture mode contains NUL byte: {error}"
                    ))
                })
            })
            .transpose()?;
        unsafe {
            ffi::av_asset_image_generator_set_aperture_mode(
                self.ptr,
                mode.as_ref().map_or(ptr::null(), |mode| mode.as_ptr()),
            );
        }
        Ok(())
    }

    pub fn requested_time_tolerance_before(&self) -> Result<Time, AVPlayerError> {
        Ok(self.info()?.requested_time_tolerance_before)
    }

    pub fn set_requested_time_tolerance_before(&self, tolerance: Time) {
        let (value, timescale, kind) = tolerance.to_raw();
        unsafe {
            ffi::av_asset_image_generator_set_requested_time_tolerance_before(
                self.ptr, value, timescale, kind,
            );
        }
    }

    pub fn requested_time_tolerance_after(&self) -> Result<Time, AVPlayerError> {
        Ok(self.info()?.requested_time_tolerance_after)
    }

    pub fn set_requested_time_tolerance_after(&self, tolerance: Time) {
        let (value, timescale, kind) = tolerance.to_raw();
        unsafe {
            ffi::av_asset_image_generator_set_requested_time_tolerance_after(
                self.ptr, value, timescale, kind,
            );
        }
    }

    pub fn dynamic_range_policy(
        &self,
    ) -> Result<Option<AssetImageGeneratorDynamicRangePolicy>, AVPlayerError> {
        Ok(self
            .info()?
            .dynamic_range_policy
            .as_deref()
            .map(AssetImageGeneratorDynamicRangePolicy::from_raw))
    }

    pub fn set_dynamic_range_policy(
        &self,
        policy: &AssetImageGeneratorDynamicRangePolicy,
    ) -> Result<(), AVPlayerError> {
        let policy = CString::new(policy.as_raw()).map_err(|error| {
            AVPlayerError::InvalidArgument(format!(
                "dynamic-range policy contains NUL byte: {error}"
            ))
        })?;
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::av_asset_image_generator_set_dynamic_range_policy(
                self.ptr,
                policy.as_ptr(),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn has_custom_video_compositor(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.has_custom_video_compositor)
    }

    pub fn custom_video_compositor_class_name(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.custom_video_compositor_class_name)
    }

    pub fn copy_image_at_time(
        &self,
        requested_time: Time,
    ) -> Result<GeneratedAssetImage, AVPlayerError> {
        let (value, timescale, kind) = requested_time.to_raw();
        let mut actual_value = 0_i64;
        let mut actual_timescale = 0_i32;
        let mut actual_kind = 1_i32;
        let mut err: *mut c_char = ptr::null_mut();
        let image_ptr = unsafe {
            ffi::av_asset_image_generator_copy_image_at_time(
                self.ptr,
                value,
                timescale,
                kind,
                &mut actual_value,
                &mut actual_timescale,
                &mut actual_kind,
                &mut err,
            )
        };
        if image_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(GeneratedAssetImage {
            image: AssetImage { ptr: image_ptr },
            actual_time: Time::from_raw(actual_value, actual_timescale, actual_kind),
        })
    }

    pub fn cancel_all_image_generation(&self) {
        unsafe { ffi::av_asset_image_generator_cancel_all_image_generation(self.ptr) };
    }
}

impl AssetImage {
    fn info(&self) -> Result<AssetImagePayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_asset_image_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn width(&self) -> Result<usize, AVPlayerError> {
        Ok(self.info()?.width)
    }

    pub fn height(&self) -> Result<usize, AVPlayerError> {
        Ok(self.info()?.height)
    }

    pub fn bits_per_component(&self) -> Result<usize, AVPlayerError> {
        Ok(self.info()?.bits_per_component)
    }

    pub fn bits_per_pixel(&self) -> Result<usize, AVPlayerError> {
        Ok(self.info()?.bits_per_pixel)
    }

    pub fn bytes_per_row(&self) -> Result<usize, AVPlayerError> {
        Ok(self.info()?.bytes_per_row)
    }

    pub fn alpha_info(&self) -> Result<u32, AVPlayerError> {
        Ok(self.info()?.alpha_info)
    }

    pub fn bitmap_info(&self) -> Result<u32, AVPlayerError> {
        Ok(self.info()?.bitmap_info)
    }

    pub fn rendering_intent(&self) -> Result<u32, AVPlayerError> {
        Ok(self.info()?.rendering_intent)
    }
}
