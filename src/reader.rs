#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use crate::ffi::AVPValue;
use crate::util::parse_value_and_free;
use core::ffi::c_void;
use core::ptr;

use apple_cf::cm::CMSampleBuffer;
use apple_cf::cv::CVPixelBuffer;

use crate::asset::{Asset, AssetTrack, MediaType};
use crate::binary::ToValue;
use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::retained::retain_release_wrapper;
use crate::time::TimeRange;

#[derive(Debug, Clone, PartialEq)]
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
    /// Mirrors the `AVPlayer` framework case `Unknown`.
    Unknown,
    /// Mirrors the `AVPlayer` framework case `Reading`.
    Reading,
    /// Mirrors the `AVPlayer` framework case `Completed`.
    Completed,
    /// Mirrors the `AVPlayer` framework case `Failed`.
    Failed,
    /// Mirrors the `AVPlayer` framework case `Cancelled`.
    Cancelled,
}

impl AssetReaderStatus {
    /// Mirrors the `AVPlayer` framework constant `fn`.
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoOutputSettings {
    pixel_format: u32,
    width: Option<i32>,
    height: Option<i32>,
}

impl VideoOutputSettings {
    /// Mirrors the `AVPlayer` framework constant `fn`.
    #[must_use]
    pub const fn new(pixel_format: u32) -> Self {
        Self {
            pixel_format,
            width: None,
            height: None,
        }
    }

    /// Mirrors the `AVPlayer` framework constant `fn`.
    #[must_use]
    pub const fn bgra() -> Self {
        Self::new(u32::from_be_bytes(*b"BGRA"))
    }

    /// Mirrors the `AVPlayer` framework constant `fn`.
    #[must_use]
    pub const fn with_dimensions(mut self, width: i32, height: i32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }
}

/// Builder for linear-PCM audio `AVAssetReader` output settings.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioOutputSettings {
    sample_rate: Option<f64>,
    channel_count: Option<u32>,
    bits_per_channel: u32,
    is_float: bool,
    is_non_interleaved: bool,
}

impl AudioOutputSettings {
    /// Mirrors the `AVPlayer` framework constant `fn`.
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

    /// Mirrors the `AVPlayer` framework constant `fn`.
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

    /// Mirrors the `AVPlayer` framework constant `fn`.
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

    /// Mirrors the `AVPlayer` framework constant `fn`.
    #[must_use]
    pub const fn non_interleaved(mut self, non_interleaved: bool) -> Self {
        self.is_non_interleaved = non_interleaved;
        self
    }
}

/// Safe wrapper around `AVAssetReader`.
#[derive(Debug)]
pub struct AssetReader {
    pub(crate) ptr: *mut c_void,
}

retain_release_wrapper!(AssetReader, release = ffi::av_reader_release);

impl AssetReader {
    /// Calls the `AVPlayer` framework counterpart for `new`.
    pub fn new(asset: &Asset) -> Result<Self, AVPlayerError> {
        let mut err: *mut AVPValue = ptr::null_mut();
        let ptr = unsafe { ffi::av_reader_create(asset.ptr, &raw mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::READER_CREATE_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn info(&self) -> Result<ReaderInfoPayload, AVPlayerError> {
        let mut err: *mut AVPValue = ptr::null_mut();
        let value_ptr = unsafe { ffi::av_reader_info_value(self.ptr, &raw mut err) };
        if value_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_value_and_free(value_ptr)
    }

    /// Calls the `AVPlayer` framework counterpart for `status`.
    pub fn status(&self) -> Result<AssetReaderStatus, AVPlayerError> {
        Ok(AssetReaderStatus::from_raw(self.info()?.status))
    }

    /// Calls the `AVPlayer` framework counterpart for `error`.
    pub fn error(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.error_message)
    }

    /// Calls the `AVPlayer` framework counterpart for `time_range`.
    pub fn time_range(&self) -> Result<TimeRange, AVPlayerError> {
        Ok(self.info()?.time_range)
    }

    /// Calls the `AVPlayer` framework counterpart for `output_count`.
    pub fn output_count(&self) -> Result<usize, AVPlayerError> {
        Ok(self.info()?.output_count)
    }

    /// Calls the `AVPlayer` framework counterpart for `set_time_range`.
    pub fn set_time_range(&self, time_range: TimeRange) -> Result<(), AVPlayerError> {
        let (start_value, start_timescale, start_kind) = time_range.start.to_raw();
        let (duration_value, duration_timescale, duration_kind) = time_range.duration.to_raw();
        let mut err: *mut AVPValue = ptr::null_mut();
        let status = unsafe {
            ffi::av_reader_set_time_range(
                self.ptr,
                start_value,
                start_timescale,
                start_kind,
                duration_value,
                duration_timescale,
                duration_kind,
                &raw mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Calls the `AVPlayer` framework counterpart for `start_reading`.
    pub fn start_reading(&self) -> Result<(), AVPlayerError> {
        let mut err: *mut AVPValue = ptr::null_mut();
        let status = unsafe { ffi::av_reader_start(self.ptr, &raw mut err) };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Calls the `AVPlayer` framework counterpart for `cancel_reading`.
    pub fn cancel_reading(&self) {
        unsafe { ffi::av_reader_cancel(self.ptr) };
    }

    /// Calls the `AVPlayer` framework counterpart for `can_add_track_output`.
    pub fn can_add_track_output(&self, output: &AssetReaderTrackOutput) -> bool {
        unsafe { ffi::av_reader_can_add_output(self.ptr, output.ptr) }
    }

    /// Calls the `AVPlayer` framework counterpart for `can_add_audio_mix_output`.
    pub fn can_add_audio_mix_output(&self, output: &AssetReaderAudioMixOutput) -> bool {
        unsafe { ffi::av_reader_can_add_output(self.ptr, output.ptr) }
    }

    /// Calls the `AVPlayer` framework counterpart for `can_add_video_composition_output`.
    pub fn can_add_video_composition_output(
        &self,
        output: &AssetReaderVideoCompositionOutput,
    ) -> bool {
        unsafe { ffi::av_reader_can_add_output(self.ptr, output.ptr) }
    }

    /// Calls the `AVPlayer` framework counterpart for `add_track_output`.
    pub fn add_track_output(&self, output: &AssetReaderTrackOutput) -> Result<(), AVPlayerError> {
        self.add_output_ptr(output.ptr)
    }

    /// Calls the `AVPlayer` framework counterpart for `add_audio_mix_output`.
    pub fn add_audio_mix_output(
        &self,
        output: &AssetReaderAudioMixOutput,
    ) -> Result<(), AVPlayerError> {
        self.add_output_ptr(output.ptr)
    }

    /// Calls the `AVPlayer` framework counterpart for `add_video_composition_output`.
    pub fn add_video_composition_output(
        &self,
        output: &AssetReaderVideoCompositionOutput,
    ) -> Result<(), AVPlayerError> {
        self.add_output_ptr(output.ptr)
    }

    fn add_output_ptr(&self, output_ptr: *mut c_void) -> Result<(), AVPlayerError> {
        let mut err: *mut AVPValue = ptr::null_mut();
        let status = unsafe { ffi::av_reader_add_output(self.ptr, output_ptr, &raw mut err) };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }
}

/// `AVAssetReaderTrackOutput`.
#[derive(Debug)]
pub struct AssetReaderTrackOutput {
    pub(crate) ptr: *mut c_void,
}

retain_release_wrapper!(
    AssetReaderTrackOutput,
    release = ffi::av_reader_output_release
);

impl AssetReaderTrackOutput {
    /// Calls the `AVPlayer` framework counterpart for `video`.
    pub fn video(
        track: &AssetTrack,
        settings: Option<&VideoOutputSettings>,
    ) -> Result<Self, AVPlayerError> {
        create_track_output(ffi::av_reader_track_output_create_video, track, settings)
            .map(|ptr| Self { ptr })
    }

    /// Calls the `AVPlayer` framework counterpart for `audio`.
    pub fn audio(
        track: &AssetTrack,
        settings: Option<&AudioOutputSettings>,
    ) -> Result<Self, AVPlayerError> {
        create_track_output(ffi::av_reader_track_output_create_audio, track, settings)
            .map(|ptr| Self { ptr })
    }

    /// Calls the `AVPlayer` framework counterpart for `passthrough`.
    pub fn passthrough(track: &AssetTrack) -> Result<Self, AVPlayerError> {
        let mut err: *mut AVPValue = ptr::null_mut();
        let ptr =
            unsafe { ffi::av_reader_track_output_create_passthrough(track.ptr, &raw mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    /// Calls the `AVPlayer` framework counterpart for `set_always_copies_sample_data`.
    pub fn set_always_copies_sample_data(&self, always_copies: bool) -> Result<(), AVPlayerError> {
        output_set_always_copies_sample_data(self.ptr, always_copies)
    }

    /// Calls the `AVPlayer` framework counterpart for `media_type`.
    pub fn media_type(&self) -> Result<MediaType, AVPlayerError> {
        output_media_type(self.ptr)
    }

    /// Calls the `AVPlayer` framework counterpart for `copy_next_sample_buffer`.
    pub fn copy_next_sample_buffer(&self) -> Result<Option<CMSampleBuffer>, AVPlayerError> {
        output_copy_next_sample_buffer(self.ptr)
    }

    /// Calls the `AVPlayer` framework counterpart for `copy_next_video_pixel_buffer`.
    pub fn copy_next_video_pixel_buffer(&self) -> Result<Option<CVPixelBuffer>, AVPlayerError> {
        output_copy_next_video_pixel_buffer(self.ptr)
    }
}

/// `AVAssetReaderAudioMixOutput`.
#[derive(Debug)]
pub struct AssetReaderAudioMixOutput {
    pub(crate) ptr: *mut c_void,
}

retain_release_wrapper!(
    AssetReaderAudioMixOutput,
    release = ffi::av_reader_output_release
);

impl AssetReaderAudioMixOutput {
    /// Calls the `AVPlayer` framework counterpart for `new`.
    pub fn new(
        tracks: &[AssetTrack],
        settings: Option<&AudioOutputSettings>,
    ) -> Result<Self, AVPlayerError> {
        let settings = settings_value(settings)?;
        let track_ptrs = track_ptrs(tracks);
        let mut err: *mut AVPValue = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_reader_audio_mix_output_create(
                track_ptrs.as_ptr(),
                track_ptrs.len(),
                settings
                    .as_ref()
                    .map_or(ptr::null(), super::binary::InputValue::as_ptr),
                &raw mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    /// Calls the `AVPlayer` framework counterpart for `set_always_copies_sample_data`.
    pub fn set_always_copies_sample_data(&self, always_copies: bool) -> Result<(), AVPlayerError> {
        output_set_always_copies_sample_data(self.ptr, always_copies)
    }

    /// Calls the `AVPlayer` framework counterpart for `media_type`.
    pub fn media_type(&self) -> Result<MediaType, AVPlayerError> {
        output_media_type(self.ptr)
    }

    /// Calls the `AVPlayer` framework counterpart for `copy_next_sample_buffer`.
    pub fn copy_next_sample_buffer(&self) -> Result<Option<CMSampleBuffer>, AVPlayerError> {
        output_copy_next_sample_buffer(self.ptr)
    }
}

/// `AVAssetReaderVideoCompositionOutput`.
#[derive(Debug)]
pub struct AssetReaderVideoCompositionOutput {
    pub(crate) ptr: *mut c_void,
}

retain_release_wrapper!(
    AssetReaderVideoCompositionOutput,
    release = ffi::av_reader_output_release
);

// SAFETY: These AVAssetReader handles are safe to transfer across thread
// boundaries; method calls are internally dispatched safely.
unsafe impl Send for AssetReader {}
unsafe impl Send for AssetReaderTrackOutput {}
unsafe impl Send for AssetReaderAudioMixOutput {}
unsafe impl Send for AssetReaderVideoCompositionOutput {}

impl AssetReaderVideoCompositionOutput {
    /// Calls the `AVPlayer` framework counterpart for `new`.
    pub fn new(
        tracks: &[AssetTrack],
        settings: Option<&VideoOutputSettings>,
    ) -> Result<Self, AVPlayerError> {
        let settings = settings_value(settings)?;
        let track_ptrs = track_ptrs(tracks);
        let mut err: *mut AVPValue = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_reader_video_composition_output_create(
                track_ptrs.as_ptr(),
                track_ptrs.len(),
                settings
                    .as_ref()
                    .map_or(ptr::null(), super::binary::InputValue::as_ptr),
                &raw mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    /// Calls the `AVPlayer` framework counterpart for `set_always_copies_sample_data`.
    pub fn set_always_copies_sample_data(&self, always_copies: bool) -> Result<(), AVPlayerError> {
        output_set_always_copies_sample_data(self.ptr, always_copies)
    }

    /// Calls the `AVPlayer` framework counterpart for `media_type`.
    pub fn media_type(&self) -> Result<MediaType, AVPlayerError> {
        output_media_type(self.ptr)
    }

    /// Calls the `AVPlayer` framework counterpart for `copy_next_sample_buffer`.
    pub fn copy_next_sample_buffer(&self) -> Result<Option<CMSampleBuffer>, AVPlayerError> {
        output_copy_next_sample_buffer(self.ptr)
    }

    /// Calls the `AVPlayer` framework counterpart for `copy_next_video_pixel_buffer`.
    pub fn copy_next_video_pixel_buffer(&self) -> Result<Option<CVPixelBuffer>, AVPlayerError> {
        output_copy_next_video_pixel_buffer(self.ptr)
    }
}

fn create_track_output<T: ToValue>(
    constructor: unsafe extern "C" fn(
        track: *mut c_void,
        settings_value: *const AVPValue,
        out_error_message: *mut *mut AVPValue,
    ) -> *mut c_void,
    track: &AssetTrack,
    settings: Option<&T>,
) -> Result<*mut c_void, AVPlayerError> {
    let settings = settings_value(settings)?;
    let mut err: *mut AVPValue = ptr::null_mut();
    let ptr = unsafe {
        constructor(
            track.ptr,
            settings
                .as_ref()
                .map_or(ptr::null(), super::binary::InputValue::as_ptr),
            &raw mut err,
        )
    };
    if ptr.is_null() {
        return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
    }
    Ok(ptr)
}

fn settings_value<T: ToValue>(
    settings: Option<&T>,
) -> Result<Option<crate::binary::InputValue>, AVPlayerError> {
    settings
        .map(|settings| crate::util::value_buffer(settings, "reader settings"))
        .transpose()
}

pub fn output_copy_next_sample_buffer(
    output: *mut c_void,
) -> Result<Option<CMSampleBuffer>, AVPlayerError> {
    let mut err: *mut AVPValue = ptr::null_mut();
    let ptr = unsafe { ffi::av_reader_output_copy_next_sample_buffer(output, &raw mut err) };
    if !err.is_null() {
        return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
    }
    Ok(unsafe { CMSampleBuffer::from_raw(ptr) })
}

pub fn output_copy_next_video_pixel_buffer(
    output: *mut c_void,
) -> Result<Option<CVPixelBuffer>, AVPlayerError> {
    let mut err: *mut AVPValue = ptr::null_mut();
    let ptr = unsafe { ffi::av_reader_output_copy_next_video_pixel_buffer(output, &raw mut err) };
    if !err.is_null() {
        return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
    }
    Ok(unsafe { CVPixelBuffer::from_raw(ptr) })
}

pub fn output_set_always_copies_sample_data(
    output: *mut c_void,
    always_copies: bool,
) -> Result<(), AVPlayerError> {
    let mut err: *mut AVPValue = ptr::null_mut();
    let status = unsafe {
        ffi::av_reader_output_set_always_copies_sample_data(output, always_copies, &raw mut err)
    };
    if status != ffi::status::OK {
        return Err(unsafe { from_swift(status, err) });
    }
    Ok(())
}

pub fn output_media_type(output: *mut c_void) -> Result<MediaType, AVPlayerError> {
    let raw = unsafe { ffi::av_reader_output_media_type(output) };
    if raw.is_null() {
        return Err(AVPlayerError::OperationFailed(
            "reader output did not return a media type".into(),
        ));
    }
    let media_type = unsafe { crate::binary::read_string(raw) }.unwrap_or_default();
    unsafe { ffi::avp_value_free(raw) };
    Ok(MediaType::from_raw(&media_type))
}

fn track_ptrs(tracks: &[AssetTrack]) -> Vec<*mut c_void> {
    tracks.iter().map(|track| track.ptr).collect()
}

impl crate::binary::FromValue for ReaderInfoPayload {
    fn from_value(value: &crate::binary::Value) -> Result<Self, AVPlayerError> {
        Ok(Self {
            status: value.field(0)?,
            error_message: value.field(1)?,
            time_range: value.field(2)?,
            output_count: value.field(3)?,
        })
    }
}

impl crate::binary::ToValue for VideoOutputSettings {
    fn to_value(&self) -> crate::binary::Value {
        crate::binary::Value::Array(vec![
            crate::binary::ToValue::to_value(&self.pixel_format),
            crate::binary::ToValue::to_value(&self.width),
            crate::binary::ToValue::to_value(&self.height),
        ])
    }
}

impl crate::binary::ToValue for AudioOutputSettings {
    fn to_value(&self) -> crate::binary::Value {
        crate::binary::Value::Array(vec![
            crate::binary::ToValue::to_value(&self.sample_rate),
            crate::binary::ToValue::to_value(&self.channel_count),
            crate::binary::ToValue::to_value(&self.bits_per_channel),
            crate::binary::ToValue::to_value(&self.is_float),
            crate::binary::ToValue::to_value(&self.is_non_interleaved),
        ])
    }
}
