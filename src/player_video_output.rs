#![allow(
    clippy::derive_partial_eq_without_eq,
    clippy::missing_errors_doc,
    clippy::must_use_candidate
)]

use core::ffi::{c_char, c_void};
use core::ptr;

use serde::Deserialize;

use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::player::Player;
use crate::reader::VideoOutputSettings;
use crate::time::Time;
use crate::util::{maybe_json_cstring, parse_json_and_free};

/// Mirrors the `AVPlayer` framework counterpart for `PlayerVideoOutputSettings`.
pub type PlayerVideoOutputSettings = VideoOutputSettings;

/// Mirrors the `AVPlayer` framework counterpart for `PlayerVideoOutputTagCollectionPreset`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerVideoOutputTagCollectionPreset {
/// Mirrors the `AVPlayer` framework case `Monoscopic`.
    Monoscopic,
/// Mirrors the `AVPlayer` framework case `Stereoscopic`.
    Stereoscopic,
}

impl PlayerVideoOutputTagCollectionPreset {
    const fn raw(self) -> u32 {
        match self {
            Self::Monoscopic => 0,
            Self::Stereoscopic => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerVideoOutputTagCollectionPayload {
    tags: Vec<String>,
}

/// Mirrors the `AVPlayer` framework counterpart for `AffineTransform`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AffineTransform {
/// Mirrors the `AVPlayer` framework property for `a`.
    pub a: f64,
/// Mirrors the `AVPlayer` framework property for `b`.
    pub b: f64,
/// Mirrors the `AVPlayer` framework property for `c`.
    pub c: f64,
/// Mirrors the `AVPlayer` framework property for `d`.
    pub d: f64,
/// Mirrors the `AVPlayer` framework property for `tx`.
    pub tx: f64,
/// Mirrors the `AVPlayer` framework property for `ty`.
    pub ty: f64,
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerVideoOutputConfiguration`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerVideoOutputConfiguration {
/// Mirrors the `AVPlayer` framework property for `has_source_player_item`.
    pub has_source_player_item: bool,
/// Mirrors the `AVPlayer` framework property for `data_channel_descriptions`.
    pub data_channel_descriptions: Vec<Vec<String>>,
/// Mirrors the `AVPlayer` framework property for `preferred_transform`.
    pub preferred_transform: Option<AffineTransform>,
/// Mirrors the `AVPlayer` framework property for `activation_time`.
    pub activation_time: Time,
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerVideoTaggedBufferKind`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PlayerVideoTaggedBufferKind {
/// Mirrors the `AVPlayer` framework case `PixelBuffer`.
    PixelBuffer,
/// Mirrors the `AVPlayer` framework case `SampleBuffer`.
    SampleBuffer,
/// Mirrors the `AVPlayer` framework case `Unknown`.
    Unknown(String),
}

impl PlayerVideoTaggedBufferKind {
    fn from_raw(raw: &str) -> Self {
        match raw {
            "pixel_buffer" => Self::PixelBuffer,
            "sample_buffer" => Self::SampleBuffer,
            other => Self::Unknown(other.to_owned()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerVideoTaggedBufferPayload {
    tags: Vec<String>,
    buffer_kind: String,
    pixel_buffer_width: Option<usize>,
    pixel_buffer_height: Option<usize>,
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerVideoTaggedBuffer`.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerVideoTaggedBuffer {
/// Mirrors the `AVPlayer` framework property for `tags`.
    pub tags: Vec<String>,
/// Mirrors the `AVPlayer` framework property for `kind`.
    pub kind: PlayerVideoTaggedBufferKind,
/// Mirrors the `AVPlayer` framework property for `pixel_buffer_width`.
    pub pixel_buffer_width: Option<usize>,
/// Mirrors the `AVPlayer` framework property for `pixel_buffer_height`.
    pub pixel_buffer_height: Option<usize>,
}

impl From<PlayerVideoTaggedBufferPayload> for PlayerVideoTaggedBuffer {
    fn from(payload: PlayerVideoTaggedBufferPayload) -> Self {
        Self {
            tags: payload.tags,
            kind: PlayerVideoTaggedBufferKind::from_raw(&payload.buffer_kind),
            pixel_buffer_width: payload.pixel_buffer_width,
            pixel_buffer_height: payload.pixel_buffer_height,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerVideoOutputSamplePayload {
    tagged_buffers: Vec<PlayerVideoTaggedBufferPayload>,
    presentation_time: Time,
    active_configuration: PlayerVideoOutputConfiguration,
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerVideoOutputSample`.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerVideoOutputSample {
/// Mirrors the `AVPlayer` framework property for `tagged_buffers`.
    pub tagged_buffers: Vec<PlayerVideoTaggedBuffer>,
/// Mirrors the `AVPlayer` framework property for `presentation_time`.
    pub presentation_time: Time,
/// Mirrors the `AVPlayer` framework property for `active_configuration`.
    pub active_configuration: PlayerVideoOutputConfiguration,
}

impl From<PlayerVideoOutputSamplePayload> for PlayerVideoOutputSample {
    fn from(payload: PlayerVideoOutputSamplePayload) -> Self {
        Self {
            tagged_buffers: payload
                .tagged_buffers
                .into_iter()
                .map(PlayerVideoTaggedBuffer::from)
                .collect(),
            presentation_time: payload.presentation_time,
            active_configuration: payload.active_configuration,
        }
    }
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerVideoOutputTagCollection`.
#[derive(Debug)]
pub struct PlayerVideoOutputTagCollection {
    pub(crate) ptr: *mut c_void,
}

impl Drop for PlayerVideoOutputTagCollection {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_player_video_output_tag_collection_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl PlayerVideoOutputTagCollection {
/// Calls the `AVPlayer` framework counterpart for `from_preset`.
    pub fn from_preset(
        preset: PlayerVideoOutputTagCollectionPreset,
    ) -> Result<Self, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_player_video_output_tag_collection_create_with_preset(preset.raw(), &mut err)
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn info(&self) -> Result<PlayerVideoOutputTagCollectionPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::av_player_video_output_tag_collection_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

/// Calls the `AVPlayer` framework counterpart for `tags`.
    pub fn tags(&self) -> Result<Vec<String>, AVPlayerError> {
        Ok(self.info()?.tags)
    }
}

/// Mirrors the `AVPlayer` framework counterpart for `VideoOutputSpecification`.
#[derive(Debug)]
pub struct VideoOutputSpecification {
    pub(crate) ptr: *mut c_void,
}

impl Drop for VideoOutputSpecification {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_video_output_specification_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl VideoOutputSpecification {
/// Calls the `AVPlayer` framework counterpart for `new`.
    pub fn new(tag_collections: &[&PlayerVideoOutputTagCollection]) -> Result<Self, AVPlayerError> {
        if tag_collections.is_empty() {
            return Err(AVPlayerError::InvalidArgument(
                "video output specifications require at least one tag collection".into(),
            ));
        }
        let mut err: *mut c_char = ptr::null_mut();
        let ptrs = tag_collections
            .iter()
            .map(|collection| collection.ptr)
            .collect::<Vec<_>>();
        let ptr = unsafe {
            ffi::av_video_output_specification_create(ptrs.as_ptr(), ptrs.len(), &mut err)
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn preferred_tag_collection_payloads(
        &self,
    ) -> Result<Vec<PlayerVideoOutputTagCollectionPayload>, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_video_output_specification_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

/// Calls the `AVPlayer` framework counterpart for `preferred_tag_collections`.
    pub fn preferred_tag_collections(&self) -> Result<Vec<Vec<String>>, AVPlayerError> {
        Ok(self
            .preferred_tag_collection_payloads()?
            .into_iter()
            .map(|payload| payload.tags)
            .collect())
    }

/// Calls the `AVPlayer` framework counterpart for `set_default_output_settings`.
    pub fn set_default_output_settings(
        &self,
        settings: Option<&PlayerVideoOutputSettings>,
    ) -> Result<(), AVPlayerError> {
        let settings = maybe_json_cstring(settings, "default video output settings")?;
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::av_video_output_specification_set_default_output_settings(
                self.ptr,
                settings
                    .as_ref()
                    .map_or(ptr::null(), |settings| settings.as_ptr()),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

/// Calls the `AVPlayer` framework counterpart for `set_output_settings_for_tag_collection`.
    pub fn set_output_settings_for_tag_collection(
        &self,
        tag_collection: &PlayerVideoOutputTagCollection,
        settings: Option<&PlayerVideoOutputSettings>,
    ) -> Result<(), AVPlayerError> {
        let settings = maybe_json_cstring(settings, "video output tag collection settings")?;
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::av_video_output_specification_set_output_settings_for_tag_collection(
                self.ptr,
                settings
                    .as_ref()
                    .map_or(ptr::null(), |settings| settings.as_ptr()),
                tag_collection.ptr,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerVideoOutput`.
#[derive(Debug)]
pub struct PlayerVideoOutput {
    pub(crate) ptr: *mut c_void,
}

impl Drop for PlayerVideoOutput {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_player_video_output_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

// SAFETY: These player-video-output handles are safe to transfer across thread
// boundaries; method calls are internally dispatched safely.
unsafe impl Send for PlayerVideoOutputTagCollection {}
unsafe impl Send for VideoOutputSpecification {}
unsafe impl Send for PlayerVideoOutput {}

impl PlayerVideoOutput {
/// Calls the `AVPlayer` framework counterpart for `new`.
    pub fn new(specification: &VideoOutputSpecification) -> Result<Self, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::av_player_video_output_create(specification.ptr, &mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

/// Calls the `AVPlayer` framework counterpart for `sample_for_host_time`.
    pub fn sample_for_host_time(
        &self,
        host_time: Time,
    ) -> Result<Option<PlayerVideoOutputSample>, AVPlayerError> {
        let (value, timescale, kind) = host_time.to_raw();
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::av_player_video_output_sample_json(self.ptr, value, timescale, kind, &mut err)
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(
            parse_json_and_free::<Option<PlayerVideoOutputSamplePayload>>(json_ptr)?
                .map(PlayerVideoOutputSample::from),
        )
    }
}

impl Player {
/// Calls the `AVPlayer` framework counterpart for `set_video_output`.
    pub fn set_video_output(&self, output: Option<&PlayerVideoOutput>) {
        unsafe {
            ffi::av_player_set_video_output(
                self.ptr,
                output.map_or(ptr::null_mut(), |output| output.ptr),
            );
        }
    }

/// Calls the `AVPlayer` framework counterpart for `video_output`.
    pub fn video_output(&self) -> Option<PlayerVideoOutput> {
        let ptr = unsafe { ffi::av_player_copy_video_output(self.ptr) };
        (!ptr.is_null()).then_some(PlayerVideoOutput { ptr })
    }
}
