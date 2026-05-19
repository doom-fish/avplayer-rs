#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use std::ffi::CString;

use apple_cf::cm::CMSampleBuffer;
use serde::Deserialize;

use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::player_layer::VideoGravity;
use crate::util::parse_json_and_free;

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SampleBufferDisplayLayerPayload {
    status: i32,
    error_message: Option<String>,
    video_gravity: String,
    ready_for_display: bool,
    ready_for_more_media_data: bool,
    has_sufficient_media_data_for_reliable_playback_start: bool,
    requires_flush_to_resume_decoding: bool,
    prevents_capture: bool,
    prevents_display_sleep_during_video_playback: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum QueuedSampleBufferRenderingStatus {
    Unknown,
    Rendering,
    Failed,
}

impl QueuedSampleBufferRenderingStatus {
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            1 => Self::Rendering,
            2 => Self::Failed,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct SampleBufferDisplayLayer {
    ptr: *mut c_void,
}

impl Drop for SampleBufferDisplayLayer {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_sample_buffer_display_layer_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

unsafe impl Send for SampleBufferDisplayLayer {}

impl SampleBufferDisplayLayer {
    pub fn new() -> Result<Self, AVPlayerError> {
        let ptr = unsafe { ffi::av_sample_buffer_display_layer_create() };
        if ptr.is_null() {
            return Err(AVPlayerError::OperationFailed(
                "bridge returned null AVSampleBufferDisplayLayer".into(),
            ));
        }
        Ok(Self { ptr })
    }

    fn info(&self) -> Result<SampleBufferDisplayLayerPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_sample_buffer_display_layer_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn status(&self) -> Result<QueuedSampleBufferRenderingStatus, AVPlayerError> {
        Ok(QueuedSampleBufferRenderingStatus::from_raw(
            self.info()?.status,
        ))
    }

    pub fn error(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.error_message)
    }

    pub fn video_gravity(&self) -> Result<VideoGravity, AVPlayerError> {
        Ok(VideoGravity::from_raw(&self.info()?.video_gravity))
    }

    pub fn set_video_gravity(&self, video_gravity: VideoGravity) -> Result<(), AVPlayerError> {
        let video_gravity = CString::new(video_gravity.as_raw()).map_err(|error| {
            AVPlayerError::InvalidArgument(format!("video gravity contains NUL byte: {error}"))
        })?;
        unsafe {
            ffi::av_sample_buffer_display_layer_set_video_gravity(self.ptr, video_gravity.as_ptr());
        };
        Ok(())
    }

    pub fn is_ready_for_display(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.ready_for_display)
    }

    pub fn is_ready_for_more_media_data(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.ready_for_more_media_data)
    }

    pub fn has_sufficient_media_data_for_reliable_playback_start(
        &self,
    ) -> Result<bool, AVPlayerError> {
        Ok(self
            .info()?
            .has_sufficient_media_data_for_reliable_playback_start)
    }

    pub fn requires_flush_to_resume_decoding(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.requires_flush_to_resume_decoding)
    }

    pub fn prevents_capture(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.prevents_capture)
    }

    pub fn set_prevents_capture(&self, prevents_capture: bool) {
        unsafe {
            ffi::av_sample_buffer_display_layer_set_prevents_capture(self.ptr, prevents_capture);
        };
    }

    pub fn prevents_display_sleep_during_video_playback(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.prevents_display_sleep_during_video_playback)
    }

    pub fn set_prevents_display_sleep_during_video_playback(&self, prevents_display_sleep: bool) {
        unsafe {
            ffi::av_sample_buffer_display_layer_set_prevents_display_sleep(
                self.ptr,
                prevents_display_sleep,
            );
        };
    }

    pub fn enqueue_sample_buffer(&self, sample_buffer: &CMSampleBuffer) {
        unsafe {
            ffi::av_sample_buffer_display_layer_enqueue_sample_buffer(
                self.ptr,
                sample_buffer.as_ptr(),
            );
        };
    }

    pub fn flush(&self) {
        unsafe { ffi::av_sample_buffer_display_layer_flush(self.ptr) };
    }

    pub fn flush_and_remove_image(&self) {
        unsafe { ffi::av_sample_buffer_display_layer_flush_and_remove_image(self.ptr) };
    }

    pub fn stop_requesting_media_data(&self) {
        unsafe { ffi::av_sample_buffer_display_layer_stop_requesting_media_data(self.ptr) };
    }
}
