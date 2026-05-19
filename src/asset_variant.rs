#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;

use serde::Deserialize;

use crate::asset::{Asset, Size, UrlAsset};
use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::media_selection::MediaSelectionOption;
use crate::util::parse_json_and_free;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssetVariantPayload {
    peak_bit_rate: Option<f64>,
    average_bit_rate: Option<f64>,
    url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssetVariantVideoAttributesPayload {
    video_range: String,
    codec_types: Vec<u32>,
    presentation_size: Size,
    nominal_frame_rate: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssetVariantVideoLayoutAttributesPayload {
    stereo_view_components: u32,
    projection_type: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssetVariantAudioAttributesPayload {
    format_ids: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssetVariantAudioRenditionPayload {
    channel_count: Option<i64>,
    binaural: Option<bool>,
    immersive: Option<bool>,
    downmix: Option<bool>,
}

/// Safe wrapper around `AVAssetVariant`.
#[derive(Debug)]
pub struct AssetVariant {
    ptr: *mut c_void,
}

impl Drop for AssetVariant {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl AssetVariant {
    fn info(&self) -> Result<AssetVariantPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_asset_variant_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn peak_bit_rate(&self) -> Result<Option<f64>, AVPlayerError> {
        Ok(self.info()?.peak_bit_rate)
    }

    pub fn average_bit_rate(&self) -> Result<Option<f64>, AVPlayerError> {
        Ok(self.info()?.average_bit_rate)
    }

    pub fn url(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.url)
    }

    pub fn video_attributes(&self) -> Option<AssetVariantVideoAttributes> {
        let ptr = unsafe { ffi::av_asset_variant_copy_video_attributes(self.ptr) };
        if ptr.is_null() {
            None
        } else {
            Some(AssetVariantVideoAttributes { ptr })
        }
    }

    pub fn audio_attributes(&self) -> Option<AssetVariantAudioAttributes> {
        let ptr = unsafe { ffi::av_asset_variant_copy_audio_attributes(self.ptr) };
        if ptr.is_null() {
            None
        } else {
            Some(AssetVariantAudioAttributes { ptr })
        }
    }
}

/// Safe wrapper around `AVAssetVariantVideoAttributes`.
#[derive(Debug)]
pub struct AssetVariantVideoAttributes {
    ptr: *mut c_void,
}

impl Drop for AssetVariantVideoAttributes {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl AssetVariantVideoAttributes {
    fn info(&self) -> Result<AssetVariantVideoAttributesPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::av_asset_variant_video_attributes_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn video_range(&self) -> Result<String, AVPlayerError> {
        Ok(self.info()?.video_range)
    }

    pub fn codec_types(&self) -> Result<Vec<u32>, AVPlayerError> {
        Ok(self.info()?.codec_types)
    }

    pub fn presentation_size(&self) -> Result<Size, AVPlayerError> {
        Ok(self.info()?.presentation_size)
    }

    pub fn nominal_frame_rate(&self) -> Result<Option<f64>, AVPlayerError> {
        Ok(self.info()?.nominal_frame_rate)
    }

    pub fn video_layout_attributes(
        &self,
    ) -> Result<Vec<AssetVariantVideoLayoutAttributes>, AVPlayerError> {
        let count = unsafe { ffi::av_asset_variant_video_layout_attribute_count(self.ptr) };
        let count = usize::try_from(count).map_err(|error| {
            AVPlayerError::OperationFailed(format!("invalid video-layout attribute count: {error}"))
        })?;
        let mut values = Vec::with_capacity(count);
        for index in 0..count {
            let ptr = unsafe {
                ffi::av_asset_variant_video_layout_attribute_copy_at_index(
                    self.ptr,
                    i32::try_from(index).unwrap_or(i32::MAX),
                )
            };
            if ptr.is_null() {
                return Err(AVPlayerError::OperationFailed(format!(
                    "bridge returned null video-layout attribute at index {index}"
                )));
            }
            values.push(AssetVariantVideoLayoutAttributes { ptr });
        }
        Ok(values)
    }
}

/// Safe wrapper around `AVAssetVariantVideoLayoutAttributes`.
#[derive(Debug)]
pub struct AssetVariantVideoLayoutAttributes {
    ptr: *mut c_void,
}

impl Drop for AssetVariantVideoLayoutAttributes {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl AssetVariantVideoLayoutAttributes {
    fn info(&self) -> Result<AssetVariantVideoLayoutAttributesPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::av_asset_variant_video_layout_attributes_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn stereo_view_components(&self) -> Result<u32, AVPlayerError> {
        Ok(self.info()?.stereo_view_components)
    }

    pub fn projection_type(&self) -> Result<String, AVPlayerError> {
        Ok(self.info()?.projection_type)
    }
}

/// Safe wrapper around `AVAssetVariantAudioAttributes`.
#[derive(Debug)]
pub struct AssetVariantAudioAttributes {
    ptr: *mut c_void,
}

impl Drop for AssetVariantAudioAttributes {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl AssetVariantAudioAttributes {
    fn info(&self) -> Result<AssetVariantAudioAttributesPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::av_asset_variant_audio_attributes_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn format_ids(&self) -> Result<Vec<u32>, AVPlayerError> {
        Ok(self.info()?.format_ids)
    }

    pub fn rendition_specific_attributes_for_media_option(
        &self,
        media_option: &MediaSelectionOption,
    ) -> Option<AssetVariantAudioRenditionSpecificAttributes> {
        let ptr = unsafe {
            ffi::av_asset_variant_audio_attributes_copy_rendition_specific_attributes(
                self.ptr,
                media_option.ptr,
            )
        };
        if ptr.is_null() {
            None
        } else {
            Some(AssetVariantAudioRenditionSpecificAttributes { ptr })
        }
    }
}

/// Safe wrapper around `AVAssetVariantAudioRenditionSpecificAttributes`.
#[derive(Debug)]
pub struct AssetVariantAudioRenditionSpecificAttributes {
    ptr: *mut c_void,
}

impl Drop for AssetVariantAudioRenditionSpecificAttributes {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl AssetVariantAudioRenditionSpecificAttributes {
    fn info(&self) -> Result<AssetVariantAudioRenditionPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::av_asset_variant_audio_rendition_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn channel_count(&self) -> Result<Option<i64>, AVPlayerError> {
        Ok(self.info()?.channel_count)
    }

    pub fn is_binaural(&self) -> Result<Option<bool>, AVPlayerError> {
        Ok(self.info()?.binaural)
    }

    pub fn is_immersive(&self) -> Result<Option<bool>, AVPlayerError> {
        Ok(self.info()?.immersive)
    }

    pub fn is_downmix(&self) -> Result<Option<bool>, AVPlayerError> {
        Ok(self.info()?.downmix)
    }
}

/// Safe wrapper around `AVAssetVariantQualifier`.
#[derive(Debug)]
pub struct AssetVariantQualifier {
    ptr: *mut c_void,
}

impl Drop for AssetVariantQualifier {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl AssetVariantQualifier {
    pub fn from_variant(variant: &AssetVariant) -> Result<Self, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr =
            unsafe { ffi::av_asset_variant_qualifier_create_with_variant(variant.ptr, &mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }
}

impl Asset {
    pub fn variants(&self) -> Result<Vec<AssetVariant>, AVPlayerError> {
        let count = unsafe { ffi::av_asset_variant_count(self.ptr) };
        let count = usize::try_from(count).map_err(|error| {
            AVPlayerError::OperationFailed(format!("invalid asset variant count: {error}"))
        })?;
        let mut variants = Vec::with_capacity(count);
        for index in 0..count {
            let ptr = unsafe {
                ffi::av_asset_copy_variant_at_index(
                    self.ptr,
                    i32::try_from(index).unwrap_or(i32::MAX),
                )
            };
            if ptr.is_null() {
                return Err(AVPlayerError::OperationFailed(format!(
                    "bridge returned null asset variant at index {index}"
                )));
            }
            variants.push(AssetVariant { ptr });
        }
        Ok(variants)
    }
}

impl UrlAsset {
    pub fn variants(&self) -> Result<Vec<AssetVariant>, AVPlayerError> {
        self.as_asset().variants()
    }
}

// SAFETY: These Objective-C asset-variant handles are safe to transfer across
// thread boundaries; method calls are internally dispatched safely.
unsafe impl Send for AssetVariant {}
unsafe impl Send for AssetVariantVideoAttributes {}
unsafe impl Send for AssetVariantVideoLayoutAttributes {}
unsafe impl Send for AssetVariantAudioAttributes {}
unsafe impl Send for AssetVariantAudioRenditionSpecificAttributes {}
unsafe impl Send for AssetVariantQualifier {}
