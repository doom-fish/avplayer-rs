#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;

use serde::Deserialize;

use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::player::{PlayerItem, PlayerStatus};
use crate::player_media_selection_criteria::PlayerActionAtItemEnd;
use crate::time::Time;
use crate::util::parse_json_and_free;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct QueuePlayerInfoPayload {
    status: i32,
    error_message: Option<String>,
    rate: f32,
    current_time: Time,
    duration: Time,
    action_at_item_end: Option<i32>,
}

/// Mirrors the `AVPlayer` framework counterpart for `QueuePlayer`.
#[derive(Debug)]
pub struct QueuePlayer {
    pub(crate) ptr: *mut c_void,
}

impl Drop for QueuePlayer {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_queue_player_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

// SAFETY: AVQueuePlayer ObjC handles are safe to transfer across thread
// boundaries; method calls are internally dispatched safely.
unsafe impl Send for QueuePlayer {}

impl QueuePlayer {
/// Calls the `AVPlayer` framework counterpart for `new`.
    pub fn new() -> Result<Self, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::av_queue_player_create(&mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::PLAYER_CREATE_FAILED, err) });
        }
        Ok(Self { ptr })
    }

/// Calls the `AVPlayer` framework counterpart for `with_items`.
    pub fn with_items(items: &[&PlayerItem]) -> Result<Self, AVPlayerError> {
        let items = items.iter().map(|item| item.ptr).collect::<Vec<_>>();
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_queue_player_create_with_items(items.as_ptr(), items.len(), &mut err)
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::PLAYER_CREATE_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn info(&self) -> Result<QueuePlayerInfoPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_player_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

/// Calls the `AVPlayer` framework counterpart for `status`.
    pub fn status(&self) -> Result<PlayerStatus, AVPlayerError> {
        Ok(PlayerStatus::from_raw(self.info()?.status))
    }

/// Calls the `AVPlayer` framework counterpart for `error`.
    pub fn error(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.error_message)
    }

/// Calls the `AVPlayer` framework counterpart for `rate`.
    pub fn rate(&self) -> Result<f32, AVPlayerError> {
        Ok(self.info()?.rate)
    }

/// Calls the `AVPlayer` framework counterpart for `current_time`.
    pub fn current_time(&self) -> Result<Time, AVPlayerError> {
        Ok(self.info()?.current_time)
    }

/// Calls the `AVPlayer` framework counterpart for `duration`.
    pub fn duration(&self) -> Result<Time, AVPlayerError> {
        Ok(self.info()?.duration)
    }

/// Calls the `AVPlayer` framework counterpart for `action_at_item_end`.
    pub fn action_at_item_end(&self) -> Result<PlayerActionAtItemEnd, AVPlayerError> {
        Ok(PlayerActionAtItemEnd::from_raw(
            self.info()?.action_at_item_end.unwrap_or(1),
        ))
    }

/// Calls the `AVPlayer` framework counterpart for `set_action_at_item_end`.
    pub fn set_action_at_item_end(
        &self,
        action: PlayerActionAtItemEnd,
    ) -> Result<(), AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status =
            unsafe { ffi::av_player_set_action_at_item_end(self.ptr, action.as_raw(), &mut err) };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

/// Calls the `AVPlayer` framework counterpart for `play`.
    pub fn play(&self) {
        unsafe { ffi::av_player_play(self.ptr) };
    }

/// Calls the `AVPlayer` framework counterpart for `pause`.
    pub fn pause(&self) {
        unsafe { ffi::av_player_pause(self.ptr) };
    }

/// Calls the `AVPlayer` framework counterpart for `set_rate`.
    pub fn set_rate(&self, rate: f32) {
        unsafe { ffi::av_player_set_rate(self.ptr, rate) };
    }

/// Calls the `AVPlayer` framework counterpart for `current_item`.
    pub fn current_item(&self) -> Option<PlayerItem> {
        let ptr = unsafe { ffi::av_player_copy_current_item(self.ptr) };
        if ptr.is_null() {
            None
        } else {
            Some(PlayerItem { ptr })
        }
    }

/// Calls the `AVPlayer` framework counterpart for `items`.
    pub fn items(&self) -> Result<Vec<PlayerItem>, AVPlayerError> {
        let count = unsafe { ffi::av_queue_player_item_count(self.ptr) };
        if count < 0 {
            return Err(AVPlayerError::OperationFailed(format!(
                "queue item count unexpectedly negative: {count}"
            )));
        }
        let count = usize::try_from(count).map_err(|error| {
            AVPlayerError::OperationFailed(format!(
                "queue item count exceeds addressable size: {error}"
            ))
        })?;

        let mut items = Vec::with_capacity(count);
        for index in 0..count {
            let ptr = unsafe {
                ffi::av_queue_player_copy_item_at_index(
                    self.ptr,
                    i32::try_from(index).map_err(|error| {
                        AVPlayerError::OperationFailed(format!(
                            "queue item index exceeds bridge range: {error}"
                        ))
                    })?,
                )
            };
            if ptr.is_null() {
                return Err(AVPlayerError::OperationFailed(format!(
                    "bridge returned null queue item at index {index}"
                )));
            }
            items.push(PlayerItem { ptr });
        }
        Ok(items)
    }

/// Calls the `AVPlayer` framework counterpart for `advance_to_next_item`.
    pub fn advance_to_next_item(&self) {
        unsafe { ffi::av_queue_player_advance_to_next_item(self.ptr) };
    }

/// Calls the `AVPlayer` framework counterpart for `can_insert_item_after`.
    pub fn can_insert_item_after(
        &self,
        item: &PlayerItem,
        after_item: Option<&PlayerItem>,
    ) -> bool {
        unsafe {
            ffi::av_queue_player_can_insert_item_after_item(
                self.ptr,
                item.ptr,
                after_item.map_or(ptr::null_mut(), |item| item.ptr),
            )
        }
    }

/// Calls the `AVPlayer` framework counterpart for `insert_item_after`.
    pub fn insert_item_after(
        &self,
        item: &PlayerItem,
        after_item: Option<&PlayerItem>,
    ) -> Result<(), AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::av_queue_player_insert_item_after_item(
                self.ptr,
                item.ptr,
                after_item.map_or(ptr::null_mut(), |item| item.ptr),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

/// Calls the `AVPlayer` framework counterpart for `remove_item`.
    pub fn remove_item(&self, item: &PlayerItem) {
        unsafe { ffi::av_queue_player_remove_item(self.ptr, item.ptr) };
    }

/// Calls the `AVPlayer` framework counterpart for `remove_all_items`.
    pub fn remove_all_items(&self) {
        unsafe { ffi::av_queue_player_remove_all_items(self.ptr) };
    }
}
