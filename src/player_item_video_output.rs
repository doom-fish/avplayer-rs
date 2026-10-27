#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use crate::ffi::AVPValue;
use core::ffi::c_void;
use core::ptr;

use apple_cf::cv::CVPixelBuffer;

use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::player::PlayerItem;
use crate::player_item_output::PlayerItemOutput;
use crate::reader::VideoOutputSettings;
use crate::retained::retain_release_wrapper;
use crate::time::Time;
use crate::util::{
    deliver, maybe_value_buffer, parse_value_and_free, to_cstring, value_buffer, value_payload,
    Handler, Registration,
};

#[derive(Debug, Clone, PartialEq)]
struct VideoOutputInfoPayload {
    suppresses_player_rendering: bool,
    has_delegate: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct VideoOutputEventPayload {
    event: String,
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerItemVideoOutputEvent`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerItemVideoOutputEvent {
    /// Mirrors the `AVPlayer` framework case `MediaDataWillChange`.
    MediaDataWillChange,
    /// Mirrors the `AVPlayer` framework case `SequenceWasFlushed`.
    SequenceWasFlushed,
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerItemVideoOutputSettings`.
pub type PlayerItemVideoOutputSettings = VideoOutputSettings;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixelBufferAttributes {
    pixel_format: u32,
    width: Option<i32>,
    height: Option<i32>,
    io_surface: bool,
    metal_compatible: bool,
}

impl PixelBufferAttributes {
    #[must_use]
    pub const fn new(pixel_format: u32) -> Self {
        Self {
            pixel_format,
            width: None,
            height: None,
            io_surface: false,
            metal_compatible: false,
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

    #[must_use]
    pub const fn with_io_surface(mut self) -> Self {
        self.io_surface = true;
        self
    }

    #[must_use]
    pub const fn with_metal_compatibility(mut self) -> Self {
        self.metal_compatible = true;
        self
    }
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerItemVideoOutput`.
#[derive(Debug)]
pub struct PlayerItemVideoOutput {
    pub(crate) ptr: *mut c_void,
}

retain_release_wrapper!(
    PlayerItemVideoOutput,
    release = ffi::av_player_item_output_release
);

impl PlayerItemVideoOutput {
    /// Calls the `AVPlayer` framework counterpart for `new`.
    pub fn new(settings: Option<&PlayerItemVideoOutputSettings>) -> Result<Self, AVPlayerError> {
        let settings = maybe_value_buffer(settings, "player-item video output settings")?;
        let mut err: *mut AVPValue = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_player_item_video_output_create(
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

    pub fn with_pixel_buffer_attributes(
        attributes: &PixelBufferAttributes,
    ) -> Result<Self, AVPlayerError> {
        let attributes = value_buffer(
            attributes,
            "player-item video output pixel buffer attributes",
        )?;
        let mut err: *mut AVPValue = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_player_item_video_output_create_with_pixel_buffer_attributes(
                attributes.as_ptr(),
                &raw mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn info(&self) -> Result<VideoOutputInfoPayload, AVPlayerError> {
        let mut err: *mut AVPValue = ptr::null_mut();
        let value_ptr =
            unsafe { ffi::av_player_item_video_output_info_value(self.ptr, &raw mut err) };
        if value_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_value_and_free(value_ptr)
    }

    /// Calls the `AVPlayer` framework counterpart for `suppresses_player_rendering`.
    pub fn suppresses_player_rendering(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.suppresses_player_rendering)
    }

    /// Calls the `AVPlayer` framework counterpart for `has_delegate`.
    pub fn has_delegate(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.has_delegate)
    }

    /// Mirrors the `AVPlayer` framework constant `fn`.
    pub const fn as_output(&self) -> PlayerItemOutput<'_> {
        PlayerItemOutput::from_ptr(self.ptr)
    }

    /// Calls the `AVPlayer` framework counterpart for `set_suppresses_player_rendering`.
    pub fn set_suppresses_player_rendering(&self, suppresses: bool) {
        unsafe { ffi::av_player_item_output_set_suppresses_player_rendering(self.ptr, suppresses) };
    }

    /// Calls the `AVPlayer` framework counterpart for `request_notification_of_media_data_change`.
    pub fn request_notification_of_media_data_change(&self, interval: f64) {
        unsafe {
            ffi::av_player_item_video_output_request_notification_of_media_data_change(
                self.ptr, interval,
            );
        }
    }

    /// Calls the `AVPlayer` framework counterpart for `observe`.
    pub fn observe<F>(
        &self,
        queue_label: Option<&str>,
        callback: F,
    ) -> Result<PlayerItemVideoOutputObserver, AVPlayerError>
    where
        F: Fn(PlayerItemVideoOutputEvent) + Send + Sync + 'static,
    {
        let queue_label = queue_label
            .map(|label| to_cstring(label, "video output queue label"))
            .transpose()?;
        let handler: Handler<PlayerItemVideoOutputEvent> = Box::new(callback);
        let inner = Registration::new(
            handler,
            ffi::av_player_item_video_output_observer_release,
            |userdata, drop_userdata, err| unsafe {
                ffi::av_player_item_video_output_add_observer(
                    self.ptr,
                    queue_label
                        .as_ref()
                        .map_or(ptr::null(), super::binary::InputString::as_ptr),
                    Some(video_output_event_trampoline),
                    userdata,
                    drop_userdata,
                    err,
                )
            },
        )?;
        Ok(PlayerItemVideoOutputObserver { _inner: inner })
    }

    /// Calls the `AVPlayer` framework counterpart for `has_new_pixel_buffer_for_item_time`.
    pub fn has_new_pixel_buffer_for_item_time(&self, item_time: Time) -> bool {
        let (value, timescale, kind) = item_time.to_raw();
        unsafe {
            ffi::av_player_item_video_output_has_new_pixel_buffer_for_item_time(
                self.ptr, value, timescale, kind,
            )
        }
    }

    /// Calls the `AVPlayer` framework counterpart for `copy_pixel_buffer_for_item_time`.
    pub fn copy_pixel_buffer_for_item_time(&self, item_time: Time) -> Option<CVPixelBuffer> {
        let (value, timescale, kind) = item_time.to_raw();
        let ptr = unsafe {
            ffi::av_player_item_video_output_copy_pixel_buffer_for_item_time(
                self.ptr, value, timescale, kind,
            )
        };
        unsafe { CVPixelBuffer::from_raw(ptr) }
    }
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerItemVideoOutputObserver`.
#[derive(Debug)]
pub struct PlayerItemVideoOutputObserver {
    _inner: Registration,
}

// SAFETY: These video-output handles are safe to transfer across thread
// boundaries; method calls are internally dispatched safely.
unsafe impl Send for PlayerItemVideoOutput {}
unsafe impl Send for PlayerItemVideoOutputObserver {}

impl PlayerItem {
    /// Calls the `AVPlayer` framework counterpart for `add_video_output`.
    pub fn add_video_output(&self, output: &PlayerItemVideoOutput) -> Result<(), AVPlayerError> {
        let mut err: *mut AVPValue = ptr::null_mut();
        let status = unsafe { ffi::av_player_item_add_output(self.ptr, output.ptr, &raw mut err) };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Calls the `AVPlayer` framework counterpart for `remove_video_output`.
    pub fn remove_video_output(&self, output: &PlayerItemVideoOutput) {
        unsafe { ffi::av_player_item_remove_output(self.ptr, output.ptr) };
    }
}

unsafe extern "C" fn video_output_event_trampoline(
    userdata: *mut c_void,
    payload_value: *const AVPValue,
) {
    let Some(payload) = (unsafe { value_payload::<VideoOutputEventPayload>(payload_value) }) else {
        return;
    };

    let event = match payload.event.as_str() {
        "media_data_will_change" => PlayerItemVideoOutputEvent::MediaDataWillChange,
        "sequence_was_flushed" => PlayerItemVideoOutputEvent::SequenceWasFlushed,
        _ => return,
    };

    unsafe { deliver(userdata, "video_output_event_trampoline", event) };
}

impl crate::binary::FromValue for VideoOutputInfoPayload {
    fn from_value(value: &crate::binary::Value) -> Result<Self, AVPlayerError> {
        Ok(Self {
            suppresses_player_rendering: value.field(0)?,
            has_delegate: value.field(1)?,
        })
    }
}

impl crate::binary::FromValue for VideoOutputEventPayload {
    fn from_value(value: &crate::binary::Value) -> Result<Self, AVPlayerError> {
        Ok(Self {
            event: value.field(0)?,
        })
    }
}

impl crate::binary::ToValue for PixelBufferAttributes {
    fn to_value(&self) -> crate::binary::Value {
        crate::binary::Value::Array(vec![
            crate::binary::ToValue::to_value(&self.pixel_format),
            crate::binary::ToValue::to_value(&self.width),
            crate::binary::ToValue::to_value(&self.height),
            crate::binary::ToValue::to_value(&self.io_surface),
            crate::binary::ToValue::to_value(&self.metal_compatible),
        ])
    }
}
