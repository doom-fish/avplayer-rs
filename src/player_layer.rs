#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use std::ffi::CString;

use apple_cf::cv::CVPixelBuffer;
use serde::Deserialize;

use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::player::Player;
use crate::util::parse_json_and_free;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerLayerInfoPayload {
    has_player: bool,
    video_gravity: String,
    ready_for_display: bool,
    video_rect: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum VideoGravity {
    ResizeAspect,
    ResizeAspectFill,
    Resize,
}

impl VideoGravity {
    #[must_use]
    pub const fn as_raw(self) -> &'static str {
        match self {
            Self::ResizeAspect => "resize_aspect",
            Self::ResizeAspectFill => "resize_aspect_fill",
            Self::Resize => "resize",
        }
    }

    #[must_use]
    pub fn from_raw(raw: &str) -> Self {
        match raw {
            "resize" => Self::Resize,
            "resize_aspect_fill" => Self::ResizeAspectFill,
            _ => Self::ResizeAspect,
        }
    }
}

pub struct PlayerLayer {
    ptr: *mut c_void,
}

impl Drop for PlayerLayer {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_player_layer_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

// SAFETY: AVPlayerLayer ObjC handles are safe to transfer across thread
// boundaries; method calls are internally dispatched safely.
unsafe impl Send for PlayerLayer {}

impl PlayerLayer {
    pub fn new(player: Option<&Player>) -> Result<Self, AVPlayerError> {
        let ptr = unsafe {
            ffi::av_player_layer_create(player.map_or(ptr::null_mut(), |player| player.ptr))
        };
        if ptr.is_null() {
            return Err(AVPlayerError::OperationFailed(
                "bridge returned null AVPlayerLayer".into(),
            ));
        }
        Ok(Self { ptr })
    }

    fn info(&self) -> Result<PlayerLayerInfoPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_player_layer_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn has_player(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.has_player)
    }

    pub fn set_player(&self, player: Option<&Player>) {
        unsafe {
            ffi::av_player_layer_set_player(
                self.ptr,
                player.map_or(ptr::null_mut(), |player| player.ptr),
            );
        }
    }

    pub fn video_gravity(&self) -> Result<VideoGravity, AVPlayerError> {
        Ok(VideoGravity::from_raw(&self.info()?.video_gravity))
    }

    pub fn set_video_gravity(&self, video_gravity: VideoGravity) -> Result<(), AVPlayerError> {
        let video_gravity = CString::new(video_gravity.as_raw()).map_err(|error| {
            AVPlayerError::InvalidArgument(format!("video gravity contains NUL byte: {error}"))
        })?;
        unsafe { ffi::av_player_layer_set_video_gravity(self.ptr, video_gravity.as_ptr()) };
        Ok(())
    }

    pub fn is_ready_for_display(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.ready_for_display)
    }

    pub fn video_rect(&self) -> Result<Rect, AVPlayerError> {
        Ok(self.info()?.video_rect)
    }

    pub fn copy_displayed_pixel_buffer(&self) -> Option<CVPixelBuffer> {
        let ptr = unsafe { ffi::av_player_layer_copy_displayed_pixel_buffer(self.ptr) };
        CVPixelBuffer::from_raw(ptr)
    }
}
