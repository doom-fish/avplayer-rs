#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;

use apple_cf::cv::CVPixelBuffer;
use serde::Deserialize;

use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::player::PlayerItem;
use crate::reader::VideoOutputSettings;
use crate::time::Time;
use crate::util::{maybe_json_cstring, parse_json_and_free};

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VideoOutputInfoPayload {
    suppresses_player_rendering: bool,
}

pub type PlayerItemVideoOutputSettings = VideoOutputSettings;

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

    pub fn set_suppresses_player_rendering(&self, suppresses: bool) {
        unsafe { ffi::av_player_item_output_set_suppresses_player_rendering(self.ptr, suppresses) };
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
