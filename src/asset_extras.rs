#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::c_char;
use core::ptr;

use serde::Deserialize;

use crate::asset::{Asset, AssetTrack, UrlAsset};
use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::metadata::MetadataItem;
use crate::time::{Time, TimeRange};
use crate::util::parse_json_and_free;

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssetExtraInfoPayload {
    preferred_rate: f32,
    preferred_volume: f32,
    overall_duration_hint: Time,
    available_metadata_formats: Vec<String>,
    available_chapter_locales: Vec<String>,
    common_metadata: Vec<MetadataItem>,
    creation_date: Option<MetadataItem>,
    lyrics: Option<String>,
    has_protected_content: bool,
    can_contain_fragments: bool,
    contains_fragments: bool,
    playable: bool,
    exportable: bool,
    readable: bool,
    composable: bool,
    compatible_with_air_play_video: bool,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssetTrackExtraInfoPayload {
    time_range: TimeRange,
    language_code: Option<String>,
    extended_language_tag: Option<String>,
    natural_time_scale: i32,
    preferred_volume: f32,
    enabled: bool,
    playable: bool,
    decodable: bool,
    self_contained: bool,
    total_sample_data_length: i64,
    available_metadata_formats: Vec<String>,
    available_track_association_types: Vec<String>,
    can_provide_sample_cursors: bool,
    min_frame_duration: Time,
    requires_frame_reordering: bool,
    audible: bool,
    visual: bool,
    legible: bool,
}

impl Asset {
    fn extra_info(&self) -> Result<AssetExtraInfoPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_asset_extra_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn cancel_loading(&self) {
        unsafe { ffi::av_asset_cancel_loading(self.ptr) };
    }

    pub fn preferred_rate(&self) -> Result<f32, AVPlayerError> {
        Ok(self.extra_info()?.preferred_rate)
    }

    pub fn preferred_volume(&self) -> Result<f32, AVPlayerError> {
        Ok(self.extra_info()?.preferred_volume)
    }

    pub fn overall_duration_hint(&self) -> Result<Time, AVPlayerError> {
        Ok(self.extra_info()?.overall_duration_hint)
    }

    pub fn available_metadata_formats(&self) -> Result<Vec<String>, AVPlayerError> {
        Ok(self.extra_info()?.available_metadata_formats)
    }

    pub fn available_chapter_locales(&self) -> Result<Vec<String>, AVPlayerError> {
        Ok(self.extra_info()?.available_chapter_locales)
    }

    pub fn common_metadata(&self) -> Result<Vec<MetadataItem>, AVPlayerError> {
        Ok(self.extra_info()?.common_metadata)
    }

    pub fn creation_date(&self) -> Result<Option<MetadataItem>, AVPlayerError> {
        Ok(self.extra_info()?.creation_date)
    }

    pub fn lyrics(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.extra_info()?.lyrics)
    }

    pub fn has_protected_content(&self) -> Result<bool, AVPlayerError> {
        Ok(self.extra_info()?.has_protected_content)
    }

    pub fn can_contain_fragments(&self) -> Result<bool, AVPlayerError> {
        Ok(self.extra_info()?.can_contain_fragments)
    }

    pub fn contains_fragments(&self) -> Result<bool, AVPlayerError> {
        Ok(self.extra_info()?.contains_fragments)
    }

    pub fn is_playable(&self) -> Result<bool, AVPlayerError> {
        Ok(self.extra_info()?.playable)
    }

    pub fn is_exportable(&self) -> Result<bool, AVPlayerError> {
        Ok(self.extra_info()?.exportable)
    }

    pub fn is_readable(&self) -> Result<bool, AVPlayerError> {
        Ok(self.extra_info()?.readable)
    }

    pub fn is_composable(&self) -> Result<bool, AVPlayerError> {
        Ok(self.extra_info()?.composable)
    }

    pub fn is_compatible_with_air_play_video(&self) -> Result<bool, AVPlayerError> {
        Ok(self.extra_info()?.compatible_with_air_play_video)
    }
}

impl UrlAsset {
    pub fn cancel_loading(&self) {
        self.asset.cancel_loading();
    }

    pub fn preferred_rate(&self) -> Result<f32, AVPlayerError> {
        self.asset.preferred_rate()
    }

    pub fn preferred_volume(&self) -> Result<f32, AVPlayerError> {
        self.asset.preferred_volume()
    }

    pub fn overall_duration_hint(&self) -> Result<Time, AVPlayerError> {
        self.asset.overall_duration_hint()
    }

    pub fn available_metadata_formats(&self) -> Result<Vec<String>, AVPlayerError> {
        self.asset.available_metadata_formats()
    }

    pub fn available_chapter_locales(&self) -> Result<Vec<String>, AVPlayerError> {
        self.asset.available_chapter_locales()
    }

    pub fn common_metadata(&self) -> Result<Vec<MetadataItem>, AVPlayerError> {
        self.asset.common_metadata()
    }

    pub fn creation_date(&self) -> Result<Option<MetadataItem>, AVPlayerError> {
        self.asset.creation_date()
    }

    pub fn lyrics(&self) -> Result<Option<String>, AVPlayerError> {
        self.asset.lyrics()
    }

    pub fn has_protected_content(&self) -> Result<bool, AVPlayerError> {
        self.asset.has_protected_content()
    }

    pub fn can_contain_fragments(&self) -> Result<bool, AVPlayerError> {
        self.asset.can_contain_fragments()
    }

    pub fn contains_fragments(&self) -> Result<bool, AVPlayerError> {
        self.asset.contains_fragments()
    }

    pub fn is_playable(&self) -> Result<bool, AVPlayerError> {
        self.asset.is_playable()
    }

    pub fn is_exportable(&self) -> Result<bool, AVPlayerError> {
        self.asset.is_exportable()
    }

    pub fn is_readable(&self) -> Result<bool, AVPlayerError> {
        self.asset.is_readable()
    }

    pub fn is_composable(&self) -> Result<bool, AVPlayerError> {
        self.asset.is_composable()
    }

    pub fn is_compatible_with_air_play_video(&self) -> Result<bool, AVPlayerError> {
        self.asset.is_compatible_with_air_play_video()
    }
}

impl AssetTrack {
    fn extra_info(&self) -> Result<AssetTrackExtraInfoPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_asset_track_extra_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn time_range(&self) -> Result<TimeRange, AVPlayerError> {
        Ok(self.extra_info()?.time_range)
    }

    pub fn language_code(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.extra_info()?.language_code)
    }

    pub fn extended_language_tag(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.extra_info()?.extended_language_tag)
    }

    pub fn natural_time_scale(&self) -> Result<i32, AVPlayerError> {
        Ok(self.extra_info()?.natural_time_scale)
    }

    pub fn preferred_volume(&self) -> Result<f32, AVPlayerError> {
        Ok(self.extra_info()?.preferred_volume)
    }

    pub fn is_enabled(&self) -> Result<bool, AVPlayerError> {
        Ok(self.extra_info()?.enabled)
    }

    pub fn is_playable(&self) -> Result<bool, AVPlayerError> {
        Ok(self.extra_info()?.playable)
    }

    pub fn is_decodable(&self) -> Result<bool, AVPlayerError> {
        Ok(self.extra_info()?.decodable)
    }

    pub fn is_self_contained(&self) -> Result<bool, AVPlayerError> {
        Ok(self.extra_info()?.self_contained)
    }

    pub fn total_sample_data_length(&self) -> Result<i64, AVPlayerError> {
        Ok(self.extra_info()?.total_sample_data_length)
    }

    pub fn available_metadata_formats(&self) -> Result<Vec<String>, AVPlayerError> {
        Ok(self.extra_info()?.available_metadata_formats)
    }

    pub fn available_track_association_types(&self) -> Result<Vec<String>, AVPlayerError> {
        Ok(self.extra_info()?.available_track_association_types)
    }

    pub fn can_provide_sample_cursors(&self) -> Result<bool, AVPlayerError> {
        Ok(self.extra_info()?.can_provide_sample_cursors)
    }

    pub fn min_frame_duration(&self) -> Result<Time, AVPlayerError> {
        Ok(self.extra_info()?.min_frame_duration)
    }

    pub fn requires_frame_reordering(&self) -> Result<bool, AVPlayerError> {
        Ok(self.extra_info()?.requires_frame_reordering)
    }

    pub fn is_audible(&self) -> Result<bool, AVPlayerError> {
        Ok(self.extra_info()?.audible)
    }

    pub fn is_visual(&self) -> Result<bool, AVPlayerError> {
        Ok(self.extra_info()?.visual)
    }

    pub fn is_legible(&self) -> Result<bool, AVPlayerError> {
        Ok(self.extra_info()?.legible)
    }
}
