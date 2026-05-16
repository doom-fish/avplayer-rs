#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use std::ffi::{CStr, CString};

use apple_cf::cm::CMSampleBuffer;
use apple_cf::cv::CVPixelBuffer;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::asset::{Asset, AssetTrack, MediaType};
use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::time::TimeRange;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReaderInfoPayload {
    status: i32,
    error_message: Option<String>,
    time_range: TimeRange,
    output_count: usize,
}

/// `AVAssetReaderStatus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AssetReaderStatus {
    Unknown,
    Reading,
    Completed,
    Failed,
    Cancelled,
}

impl AssetReaderStatus {
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            1 => Self::Reading,
            2 => Self::Completed,
            3 => Self::Failed,
            4 => Self::Cancelled,
            _ => Self::Unknown,
        }
    }
}

/// Builder for uncompressed video `AVAssetReader` output settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoOutputSettings {
    pixel_format: u32,
    width: Option<i32>,
    height: Option<i32>,
}

impl VideoOutputSettings {
    #[must_use]
    pub const fn new(pixel_format: u32) -> Self {
        Self {
            pixel_format,
            width: None,
            height: None,
        }
    }

    #[must_use]
    pub const fn bgra() -> Self {
        Self::new(u32::from_be_bytes(*b"BGRA"))
    }

    #[must_use]
    pub const fn with_dimensions(mut self, width: i32, height: i32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }
}

/// Builder for linear-PCM audio `AVAssetReader` output settings.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioOutputSettings {
    sample_rate: Option<f64>,
    channel_count: Option<u32>,
    bits_per_channel: u32,
    is_float: bool,
    is_non_interleaved: bool,
}

impl AudioOutputSettings {
    #[must_use]
    pub const fn pcm_i16(sample_rate: f64, channel_count: u32) -> Self {
        Self {
            sample_rate: Some(sample_rate),
            channel_count: Some(channel_count),
            bits_per_channel: 16,
            is_float: false,
            is_non_interleaved: false,
        }
    }

    #[must_use]
    pub const fn pcm_i32(sample_rate: f64, channel_count: u32) -> Self {
        Self {
            sample_rate: Some(sample_rate),
            channel_count: Some(channel_count),
            bits_per_channel: 32,
            is_float: false,
            is_non_interleaved: false,
        }
    }

    #[must_use]
    pub const fn pcm_f32(sample_rate: f64, channel_count: u32) -> Self {
        Self {
            sample_rate: Some(sample_rate),
            channel_count: Some(channel_count),
            bits_per_channel: 32,
            is_float: true,
            is_non_interleaved: false,
        }
    }

    #[must_use]
    pub const fn non_interleaved(mut self, non_interleaved: bool) -> Self {
        self.is_non_interleaved = non_interleaved;
        self
    }
}

/// Safe wrapper around `AVAssetReader`.
pub struct AssetReader {
    ptr: *mut c_void,
}

impl Drop for AssetReader {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_reader_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl AssetReader {
    pub fn new(asset: &Asset) -> Result<Self, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::av_reader_create(asset.ptr, &mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::READER_CREATE_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn info(&self) -> Result<ReaderInfoPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_reader_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn status(&self) -> Result<AssetReaderStatus, AVPlayerError> {
        Ok(AssetReaderStatus::from_raw(self.info()?.status))
    }

    pub fn error(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.error_message)
    }

    pub fn time_range(&self) -> Result<TimeRange, AVPlayerError> {
        Ok(self.info()?.time_range)
    }

    pub fn output_count(&self) -> Result<usize, AVPlayerError> {
        Ok(self.info()?.output_count)
    }

    pub fn set_time_range(&self, time_range: TimeRange) -> Result<(), AVPlayerError> {
        let (start_value, start_timescale, start_kind) = time_range.start.to_raw();
        let (duration_value, duration_timescale, duration_kind) = time_range.duration.to_raw();
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::av_reader_set_time_range(
                self.ptr,
                start_value,
                start_timescale,
                start_kind,
                duration_value,
                duration_timescale,
                duration_kind,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn start_reading(&self) -> Result<(), AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe { ffi::av_reader_start(self.ptr, &mut err) };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn cancel_reading(&self) {
        unsafe { ffi::av_reader_cancel(self.ptr) };
    }

    pub fn can_add_track_output(&self, output: &AssetReaderTrackOutput) -> bool {
        unsafe { ffi::av_reader_can_add_output(self.ptr, output.ptr) }
    }

    pub fn can_add_audio_mix_output(&self, output: &AssetReaderAudioMixOutput) -> bool {
        unsafe { ffi::av_reader_can_add_output(self.ptr, output.ptr) }
    }

    pub fn can_add_video_composition_output(&self, output: &AssetReaderVideoCompositionOutput) -> bool {
        unsafe { ffi::av_reader_can_add_output(self.ptr, output.ptr) }
    }

    pub fn add_track_output(&self, output: &AssetReaderTrackOutput) -> Result<(), AVPlayerError> {
        self.add_output_ptr(output.ptr)
    }

    pub fn add_audio_mix_output(&self, output: &AssetReaderAudioMixOutput) -> Result<(), AVPlayerError> {
        self.add_output_ptr(output.ptr)
    }

    pub fn add_video_composition_output(
        &self,
        output: &AssetReaderVideoCompositionOutput,
    ) -> Result<(), AVPlayerError> {
        self.add_output_ptr(output.ptr)
    }

    fn add_output_ptr(&self, output_ptr: *mut c_void) -> Result<(), AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe { ffi::av_reader_add_output(self.ptr, output_ptr, &mut err) };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }
}

/// `AVAssetReaderTrackOutput`.
pub struct AssetReaderTrackOutput {
    pub(crate) ptr: *mut c_void,
}

impl Drop for AssetReaderTrackOutput {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_reader_output_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl AssetReaderTrackOutput {
    pub fn video(
        track: &AssetTrack,
        settings: Option<&VideoOutputSettings>,
    ) -> Result<Self, AVPlayerError> {
        create_track_output(ffi::av_reader_track_output_create_video, track, settings)
            .map(|ptr| Self { ptr })
    }

    pub fn audio(
        track: &AssetTrack,
        settings: Option<&AudioOutputSettings>,
    ) -> Result<Self, AVPlayerError> {
        create_track_output(ffi::av_reader_track_output_create_audio, track, settings)
            .map(|ptr| Self { ptr })
    }

    pub fn passthrough(track: &AssetTrack) -> Result<Self, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::av_reader_track_output_create_passthrough(track.ptr, &mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    pub fn set_always_copies_sample_data(&self, always_copies: bool) {
        unsafe { ffi::av_reader_output_set_always_copies_sample_data(self.ptr, always_copies) };
    }

    pub fn media_type(&self) -> Result<MediaType, AVPlayerError> {
        output_media_type(self.ptr)
    }

    pub fn copy_next_sample_buffer(&self) -> Option<CMSampleBuffer> {
        let ptr = unsafe { ffi::av_reader_output_copy_next_sample_buffer(self.ptr) };
        CMSampleBuffer::from_raw(ptr)
    }

    pub fn copy_next_video_pixel_buffer(&self) -> Option<CVPixelBuffer> {
        let ptr = unsafe { ffi::av_reader_output_copy_next_video_pixel_buffer(self.ptr) };
        CVPixelBuffer::from_raw(ptr)
    }
}

/// `AVAssetReaderAudioMixOutput`.
pub struct AssetReaderAudioMixOutput {
    pub(crate) ptr: *mut c_void,
}

impl Drop for AssetReaderAudioMixOutput {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_reader_output_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl AssetReaderAudioMixOutput {
    pub fn new(
        tracks: &[AssetTrack],
        settings: Option<&AudioOutputSettings>,
    ) -> Result<Self, AVPlayerError> {
        let settings = settings_json(settings)?;
        let track_ptrs = track_ptrs(tracks);
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_reader_audio_mix_output_create(
                track_ptrs.as_ptr(),
                track_ptrs.len(),
                settings
                    .as_ref()
                    .map_or(ptr::null(), |settings| settings.as_ptr()),
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    pub fn set_always_copies_sample_data(&self, always_copies: bool) {
        unsafe { ffi::av_reader_output_set_always_copies_sample_data(self.ptr, always_copies) };
    }

    pub fn media_type(&self) -> Result<MediaType, AVPlayerError> {
        output_media_type(self.ptr)
    }

    pub fn copy_next_sample_buffer(&self) -> Option<CMSampleBuffer> {
        let ptr = unsafe { ffi::av_reader_output_copy_next_sample_buffer(self.ptr) };
        CMSampleBuffer::from_raw(ptr)
    }
}

/// `AVAssetReaderVideoCompositionOutput`.
pub struct AssetReaderVideoCompositionOutput {
    pub(crate) ptr: *mut c_void,
}

impl Drop for AssetReaderVideoCompositionOutput {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_reader_output_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl AssetReaderVideoCompositionOutput {
    pub fn new(
        tracks: &[AssetTrack],
        settings: Option<&VideoOutputSettings>,
    ) -> Result<Self, AVPlayerError> {
        let settings = settings_json(settings)?;
        let track_ptrs = track_ptrs(tracks);
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_reader_video_composition_output_create(
                track_ptrs.as_ptr(),
                track_ptrs.len(),
                settings
                    .as_ref()
                    .map_or(ptr::null(), |settings| settings.as_ptr()),
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    pub fn set_always_copies_sample_data(&self, always_copies: bool) {
        unsafe { ffi::av_reader_output_set_always_copies_sample_data(self.ptr, always_copies) };
    }

    pub fn media_type(&self) -> Result<MediaType, AVPlayerError> {
        output_media_type(self.ptr)
    }

    pub fn copy_next_sample_buffer(&self) -> Option<CMSampleBuffer> {
        let ptr = unsafe { ffi::av_reader_output_copy_next_sample_buffer(self.ptr) };
        CMSampleBuffer::from_raw(ptr)
    }

    pub fn copy_next_video_pixel_buffer(&self) -> Option<CVPixelBuffer> {
        let ptr = unsafe { ffi::av_reader_output_copy_next_video_pixel_buffer(self.ptr) };
        CVPixelBuffer::from_raw(ptr)
    }
}

fn create_track_output<T: Serialize>(
    constructor: unsafe extern "C" fn(
        track: *mut c_void,
        settings_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void,
    track: &AssetTrack,
    settings: Option<&T>,
) -> Result<*mut c_void, AVPlayerError> {
    let settings = settings_json(settings)?;
    let mut err: *mut c_char = ptr::null_mut();
    let ptr = unsafe {
        constructor(
            track.ptr,
            settings
                .as_ref()
                .map_or(ptr::null(), |settings| settings.as_ptr()),
            &mut err,
        )
    };
    if ptr.is_null() {
        return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
    }
    Ok(ptr)
}

fn settings_json<T: Serialize>(settings: Option<&T>) -> Result<Option<CString>, AVPlayerError> {
    settings
        .map(|settings| serde_json::to_string(settings))
        .transpose()
        .map_err(|error| AVPlayerError::InvalidArgument(format!("failed to encode reader settings: {error}")))?
        .map(|json| {
            CString::new(json).map_err(|error| {
                AVPlayerError::InvalidArgument(format!("reader settings JSON contains NUL byte: {error}"))
            })
        })
        .transpose()
}

fn output_media_type(output: *mut c_void) -> Result<MediaType, AVPlayerError> {
    let raw = unsafe { ffi::av_reader_output_media_type(output) };
    if raw.is_null() {
        return Err(AVPlayerError::OperationFailed(
            "reader output did not return a media type".into(),
        ));
    }
    let media_type = unsafe { CStr::from_ptr(raw) }
        .to_string_lossy()
        .into_owned();
    unsafe { ffi::avp_string_free(raw) };
    Ok(MediaType::from_raw(&media_type))
}

fn track_ptrs(tracks: &[AssetTrack]) -> Vec<*mut c_void> {
    tracks.iter().map(|track| track.ptr).collect()
}

fn parse_json_and_free<T: DeserializeOwned>(json_ptr: *mut c_char) -> Result<T, AVPlayerError> {
    let json = unsafe { CStr::from_ptr(json_ptr) }
        .to_string_lossy()
        .into_owned();
    unsafe { ffi::avp_string_free(json_ptr) };
    serde_json::from_str::<T>(&json)
        .map_err(|error| AVPlayerError::OperationFailed(format!("failed to decode bridge JSON: {error}")))
}
