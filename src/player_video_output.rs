#![allow(
    clippy::derive_partial_eq_without_eq,
    clippy::missing_errors_doc,
    clippy::must_use_candidate
)]

use core::ffi::{c_char, c_void};
use core::ptr;

use apple_cf::cm::CMSampleBuffer;
use apple_cf::cv::CVPixelBuffer;
use serde::Deserialize;

use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::player::Player;
use crate::reader::VideoOutputSettings;
use crate::retained::retain_release_wrapper;
use crate::time::Time;
use crate::util::{maybe_json_cstring, parse_json_and_free, take_object_array};

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

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PlayerVideoTaggedBufferData {
    PixelBuffer(CVPixelBuffer),
    SampleBuffer(CMSampleBuffer),
    Unavailable,
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
    pub buffer: PlayerVideoTaggedBufferData,
}

impl PlayerVideoTaggedBuffer {
    fn from_payload(payload: PlayerVideoTaggedBufferPayload, retained_buffer: *mut c_void) -> Self {
        let kind = PlayerVideoTaggedBufferKind::from_raw(&payload.buffer_kind);
        let buffer = match kind {
            PlayerVideoTaggedBufferKind::PixelBuffer => {
                unsafe { CVPixelBuffer::from_raw(retained_buffer) }.map_or(
                    PlayerVideoTaggedBufferData::Unavailable,
                    PlayerVideoTaggedBufferData::PixelBuffer,
                )
            }
            PlayerVideoTaggedBufferKind::SampleBuffer => {
                unsafe { CMSampleBuffer::from_raw(retained_buffer) }.map_or(
                    PlayerVideoTaggedBufferData::Unavailable,
                    PlayerVideoTaggedBufferData::SampleBuffer,
                )
            }
            PlayerVideoTaggedBufferKind::Unknown(_) => {
                release_object(retained_buffer);
                PlayerVideoTaggedBufferData::Unavailable
            }
        };
        Self {
            tags: payload.tags,
            kind,
            pixel_buffer_width: payload.pixel_buffer_width,
            pixel_buffer_height: payload.pixel_buffer_height,
            buffer,
        }
    }
}

fn release_object(object: *mut c_void) {
    if !object.is_null() {
        unsafe { ffi::av_ns_object_release(object) };
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

impl PlayerVideoOutputSample {
    fn from_payload(
        payload: PlayerVideoOutputSamplePayload,
        retained_buffers: Vec<*mut c_void>,
    ) -> Self {
        let mut retained_buffers = retained_buffers.into_iter();
        let tagged_buffers = payload
            .tagged_buffers
            .into_iter()
            .map(|buffer| {
                PlayerVideoTaggedBuffer::from_payload(
                    buffer,
                    retained_buffers.next().unwrap_or(ptr::null_mut()),
                )
            })
            .collect();
        retained_buffers.for_each(release_object);
        Self {
            tagged_buffers,
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

retain_release_wrapper!(
    PlayerVideoOutputTagCollection,
    release = ffi::av_player_video_output_tag_collection_release
);

impl PlayerVideoOutputTagCollection {
    /// Calls the `AVPlayer` framework counterpart for `from_preset`.
    pub fn from_preset(
        preset: PlayerVideoOutputTagCollectionPreset,
    ) -> Result<Self, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_player_video_output_tag_collection_create_with_preset(
                preset.raw(),
                &raw mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn info(&self) -> Result<PlayerVideoOutputTagCollectionPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::av_player_video_output_tag_collection_info_json(self.ptr, &raw mut err) };
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

retain_release_wrapper!(
    VideoOutputSpecification,
    release = ffi::av_video_output_specification_release
);

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
            ffi::av_video_output_specification_create(ptrs.as_ptr(), ptrs.len(), &raw mut err)
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
        let json_ptr =
            unsafe { ffi::av_video_output_specification_info_json(self.ptr, &raw mut err) };
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
                &raw mut err,
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
                &raw mut err,
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

retain_release_wrapper!(
    PlayerVideoOutput,
    release = ffi::av_player_video_output_release
);

// SAFETY: These player-video-output handles are safe to transfer across thread
// boundaries; method calls are internally dispatched safely.
unsafe impl Send for PlayerVideoOutputTagCollection {}
unsafe impl Send for VideoOutputSpecification {}
unsafe impl Send for PlayerVideoOutput {}

impl PlayerVideoOutput {
    /// Calls the `AVPlayer` framework counterpart for `new`.
    pub fn new(specification: &VideoOutputSpecification) -> Result<Self, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::av_player_video_output_create(specification.ptr, &raw mut err) };
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
        let mut buffers: *mut c_void = ptr::null_mut();
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::av_player_video_output_sample_json(
                self.ptr,
                value,
                timescale,
                kind,
                &raw mut buffers,
                &raw mut err,
            )
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        let retained_buffers = unsafe { take_object_array(buffers) };
        match parse_json_and_free::<Option<PlayerVideoOutputSamplePayload>>(json_ptr) {
            Ok(payload) => Ok(payload
                .map(|payload| PlayerVideoOutputSample::from_payload(payload, retained_buffers))),
            Err(error) => {
                retained_buffers.into_iter().for_each(release_object);
                Err(error)
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    const BGRA: u32 = u32::from_be_bytes(*b"BGRA");

    fn tagged(kind: &str) -> PlayerVideoTaggedBufferPayload {
        PlayerVideoTaggedBufferPayload {
            tags: vec!["tag".into()],
            buffer_kind: kind.into(),
            pixel_buffer_width: Some(4),
            pixel_buffer_height: Some(2),
        }
    }

    #[test]
    fn samples_adopt_the_retained_buffers_in_order() {
        let pixel_buffer = CVPixelBuffer::create(4, 2, BGRA).unwrap();
        let retained = pixel_buffer.clone();
        let retained_ptr = retained.as_ptr();
        core::mem::forget(retained);
        let payload = PlayerVideoOutputSamplePayload {
            tagged_buffers: vec![
                tagged("pixel_buffer"),
                tagged("sample_buffer"),
                tagged("other"),
            ],
            presentation_time: Time::new(1, 30),
            active_configuration: PlayerVideoOutputConfiguration {
                has_source_player_item: true,
                data_channel_descriptions: Vec::new(),
                preferred_transform: None,
                activation_time: Time::new(0, 1),
            },
        };

        let sample = PlayerVideoOutputSample::from_payload(
            payload,
            vec![retained_ptr, ptr::null_mut(), ptr::null_mut()],
        );

        assert_eq!(sample.tagged_buffers.len(), 3);
        assert_eq!(
            sample.tagged_buffers[0].buffer,
            PlayerVideoTaggedBufferData::PixelBuffer(pixel_buffer)
        );
        assert_eq!(
            sample.tagged_buffers[1].buffer,
            PlayerVideoTaggedBufferData::Unavailable
        );
        assert_eq!(
            sample.tagged_buffers[2].kind,
            PlayerVideoTaggedBufferKind::Unknown("other".into())
        );
        assert_eq!(sample.presentation_time, Time::new(1, 30));
    }
}
