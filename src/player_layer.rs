#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use crate::binary::InputString;
use crate::ffi::AVPValue;
use core::ffi::c_void;
use core::ptr;

use apple_cf::cv::CVPixelBuffer;

use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::player::Player;
use crate::retained::retain_release_wrapper;
use crate::util::parse_value_and_free;

/// Mirrors the `AVPlayer` framework counterpart for `Rect`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// Mirrors the `AVPlayer` framework property for `x`.
    pub x: f64,
    /// Mirrors the `AVPlayer` framework property for `y`.
    pub y: f64,
    /// Mirrors the `AVPlayer` framework property for `width`.
    pub width: f64,
    /// Mirrors the `AVPlayer` framework property for `height`.
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq)]
struct PlayerLayerInfoPayload {
    has_player: bool,
    video_gravity: String,
    ready_for_display: bool,
    video_rect: Rect,
}

/// Mirrors the `AVPlayer` framework counterpart for `VideoGravity`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum VideoGravity {
    /// Mirrors the `AVPlayer` framework case `ResizeAspect`.
    ResizeAspect,
    /// Mirrors the `AVPlayer` framework case `ResizeAspectFill`.
    ResizeAspectFill,
    /// Mirrors the `AVPlayer` framework case `Resize`.
    Resize,
}

impl VideoGravity {
    /// Mirrors the `AVPlayer` framework constant `fn`.
    #[must_use]
    pub const fn as_raw(self) -> &'static str {
        match self {
            Self::ResizeAspect => "resize_aspect",
            Self::ResizeAspectFill => "resize_aspect_fill",
            Self::Resize => "resize",
        }
    }

    /// Calls the `AVPlayer` framework counterpart for `from_raw`.
    #[must_use]
    pub fn from_raw(raw: &str) -> Self {
        match raw {
            "resize" => Self::Resize,
            "resize_aspect_fill" => Self::ResizeAspectFill,
            _ => Self::ResizeAspect,
        }
    }
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerLayer`.
#[derive(Debug)]
pub struct PlayerLayer {
    ptr: *mut c_void,
}

retain_release_wrapper!(PlayerLayer, release = ffi::av_player_layer_release);

// SAFETY: AVPlayerLayer ObjC handles are safe to transfer across thread
// boundaries; method calls are internally dispatched safely.
unsafe impl Send for PlayerLayer {}

impl PlayerLayer {
    /// Calls the `AVPlayer` framework counterpart for `new`.
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
        let mut err: *mut AVPValue = ptr::null_mut();
        let value_ptr = unsafe { ffi::av_player_layer_info_value(self.ptr, &raw mut err) };
        if value_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_value_and_free(value_ptr)
    }

    /// Calls the `AVPlayer` framework counterpart for `has_player`.
    pub fn has_player(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.has_player)
    }

    /// Calls the `AVPlayer` framework counterpart for `set_player`.
    pub fn set_player(&self, player: Option<&Player>) {
        unsafe {
            ffi::av_player_layer_set_player(
                self.ptr,
                player.map_or(ptr::null_mut(), |player| player.ptr),
            );
        }
    }

    /// Calls the `AVPlayer` framework counterpart for `video_gravity`.
    pub fn video_gravity(&self) -> Result<VideoGravity, AVPlayerError> {
        Ok(VideoGravity::from_raw(&self.info()?.video_gravity))
    }

    /// Calls the `AVPlayer` framework counterpart for `set_video_gravity`.
    pub fn set_video_gravity(&self, video_gravity: VideoGravity) -> Result<(), AVPlayerError> {
        let video_gravity = InputString::new(video_gravity.as_raw()).map_err(|error| {
            AVPlayerError::InvalidArgument(format!("video gravity contains NUL byte: {error}"))
        })?;
        unsafe { ffi::av_player_layer_set_video_gravity(self.ptr, video_gravity.as_ptr()) };
        Ok(())
    }

    /// Calls the `AVPlayer` framework counterpart for `is_ready_for_display`.
    pub fn is_ready_for_display(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.ready_for_display)
    }

    /// Calls the `AVPlayer` framework counterpart for `video_rect`.
    pub fn video_rect(&self) -> Result<Rect, AVPlayerError> {
        Ok(self.info()?.video_rect)
    }

    /// Calls the `AVPlayer` framework counterpart for `copy_displayed_pixel_buffer`.
    pub fn copy_displayed_pixel_buffer(&self) -> Option<CVPixelBuffer> {
        let ptr = unsafe { ffi::av_player_layer_copy_displayed_pixel_buffer(self.ptr) };
        unsafe { CVPixelBuffer::from_raw(ptr) }
    }

    pub const fn as_ptr(&self) -> *mut c_void {
        self.ptr
    }
}

impl crate::binary::FromValue for Rect {
    fn from_value(value: &crate::binary::Value) -> Result<Self, AVPlayerError> {
        Ok(Self {
            x: value.field(0)?,
            y: value.field(1)?,
            width: value.field(2)?,
            height: value.field(3)?,
        })
    }
}

impl crate::binary::FromValue for PlayerLayerInfoPayload {
    fn from_value(value: &crate::binary::Value) -> Result<Self, AVPlayerError> {
        Ok(Self {
            has_player: value.field(0)?,
            video_gravity: value.field(1)?,
            ready_for_display: value.field(2)?,
            video_rect: value.field(3)?,
        })
    }
}
