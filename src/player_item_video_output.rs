#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;

use apple_cf::cv::CVPixelBuffer;
use serde::Deserialize;

use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::player::PlayerItem;
use crate::player_item_output::PlayerItemOutput;
use crate::reader::VideoOutputSettings;
use crate::time::Time;
use crate::util::{maybe_json_cstring, parse_json_and_free, to_cstring};

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VideoOutputInfoPayload {
    suppresses_player_rendering: bool,
    has_delegate: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VideoOutputEventPayload {
    event: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerItemVideoOutputEvent {
    MediaDataWillChange,
    SequenceWasFlushed,
}

struct VideoOutputObserverState {
    callback: Box<dyn Fn(PlayerItemVideoOutputEvent) + Send + 'static>,
}

pub type PlayerItemVideoOutputSettings = VideoOutputSettings;

#[derive(Debug)]
pub struct PlayerItemVideoOutput {
    pub(crate) ptr: *mut c_void,
}

impl Drop for PlayerItemVideoOutput {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_player_item_output_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl PlayerItemVideoOutput {
    pub fn new(settings: Option<&PlayerItemVideoOutputSettings>) -> Result<Self, AVPlayerError> {
        let settings = maybe_json_cstring(settings, "player-item video output settings")?;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_player_item_video_output_create(
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

    fn info(&self) -> Result<VideoOutputInfoPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_player_item_video_output_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn suppresses_player_rendering(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.suppresses_player_rendering)
    }

    pub fn has_delegate(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.has_delegate)
    }

    pub const fn as_output(&self) -> PlayerItemOutput<'_> {
        PlayerItemOutput::from_ptr(self.ptr)
    }

    pub fn set_suppresses_player_rendering(&self, suppresses: bool) {
        unsafe { ffi::av_player_item_output_set_suppresses_player_rendering(self.ptr, suppresses) };
    }

    pub fn request_notification_of_media_data_change(&self, interval: f64) {
        unsafe {
            ffi::av_player_item_video_output_request_notification_of_media_data_change(
                self.ptr, interval,
            );
        }
    }

    pub fn observe<F>(
        &self,
        queue_label: Option<&str>,
        callback: F,
    ) -> Result<PlayerItemVideoOutputObserver, AVPlayerError>
    where
        F: Fn(PlayerItemVideoOutputEvent) + Send + 'static,
    {
        let queue_label = queue_label
            .map(|label| to_cstring(label, "video output queue label"))
            .transpose()?;
        let state = Box::new(VideoOutputObserverState {
            callback: Box::new(callback),
        });
        let userdata = Box::into_raw(state).cast::<c_void>();
        let mut err: *mut c_char = ptr::null_mut();
        let token = unsafe {
            ffi::av_player_item_video_output_add_observer(
                self.ptr,
                queue_label
                    .as_ref()
                    .map_or(ptr::null(), |label| label.as_ptr()),
                Some(video_output_event_trampoline),
                userdata,
                Some(video_output_observer_drop),
                &mut err,
            )
        };
        if token.is_null() {
            unsafe { video_output_observer_drop(userdata) };
            return Err(unsafe { from_swift(ffi::status::OBSERVER_FAILED, err) });
        }
        Ok(PlayerItemVideoOutputObserver { token })
    }

    pub fn has_new_pixel_buffer_for_item_time(&self, item_time: Time) -> bool {
        let (value, timescale, kind) = item_time.to_raw();
        unsafe {
            ffi::av_player_item_video_output_has_new_pixel_buffer_for_item_time(
                self.ptr, value, timescale, kind,
            )
        }
    }

    pub fn copy_pixel_buffer_for_item_time(&self, item_time: Time) -> Option<CVPixelBuffer> {
        let (value, timescale, kind) = item_time.to_raw();
        let ptr = unsafe {
            ffi::av_player_item_video_output_copy_pixel_buffer_for_item_time(
                self.ptr, value, timescale, kind,
            )
        };
        CVPixelBuffer::from_raw(ptr)
    }
}

#[derive(Debug)]
pub struct PlayerItemVideoOutputObserver {
    token: *mut c_void,
}

impl Drop for PlayerItemVideoOutputObserver {
    fn drop(&mut self) {
        if !self.token.is_null() {
            unsafe { ffi::av_player_item_video_output_observer_release(self.token) };
            self.token = ptr::null_mut();
        }
    }
}

// SAFETY: These video-output handles are safe to transfer across thread
// boundaries; method calls are internally dispatched safely.
unsafe impl Send for PlayerItemVideoOutput {}
unsafe impl Send for PlayerItemVideoOutputObserver {}

impl PlayerItem {
    pub fn add_video_output(&self, output: &PlayerItemVideoOutput) -> Result<(), AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe { ffi::av_player_item_add_output(self.ptr, output.ptr, &mut err) };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn remove_video_output(&self, output: &PlayerItemVideoOutput) {
        unsafe { ffi::av_player_item_remove_output(self.ptr, output.ptr) };
    }
}

unsafe extern "C" fn video_output_event_trampoline(
    userdata: *mut c_void,
    payload_json: *const c_char,
) {
    if userdata.is_null() || payload_json.is_null() {
        return;
    }

    let callback = &*userdata.cast::<VideoOutputObserverState>();
    let Ok(payload) = core::ffi::CStr::from_ptr(payload_json).to_str() else {
        return;
    };
    let Ok(payload) = serde_json::from_str::<VideoOutputEventPayload>(payload) else {
        return;
    };

    let event = match payload.event.as_str() {
        "media_data_will_change" => PlayerItemVideoOutputEvent::MediaDataWillChange,
        "sequence_was_flushed" => PlayerItemVideoOutputEvent::SequenceWasFlushed,
        _ => return,
    };

    crate::util::catch_cb_panic("video_output_event_trampoline", || {
        (callback.callback)(event);
    });
}

unsafe extern "C" fn video_output_observer_drop(userdata: *mut c_void) {
    if !userdata.is_null() {
        drop(Box::from_raw(userdata.cast::<VideoOutputObserverState>()));
    }
}
