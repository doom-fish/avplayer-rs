#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;

use serde::Deserialize;

use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::player::PlayerItem;
use crate::queue_player::QueuePlayer;
use crate::time::TimeRange;
use crate::util::parse_json_and_free;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerLooperInfoPayload {
    status: i32,
    error_message: Option<String>,
    loop_count: i64,
    looping_item_count: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PlayerLooperStatus {
    Unknown,
    Ready,
    Failed,
    Cancelled,
}

impl PlayerLooperStatus {
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            1 => Self::Ready,
            2 => Self::Failed,
            3 => Self::Cancelled,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PlayerLooperItemOrdering {
    LoopingItemsPrecedeExistingItems,
    LoopingItemsFollowExistingItems,
}

impl PlayerLooperItemOrdering {
    #[must_use]
    pub const fn as_raw(self) -> i32 {
        match self {
            Self::LoopingItemsPrecedeExistingItems => 0,
            Self::LoopingItemsFollowExistingItems => 1,
        }
    }
}

pub struct PlayerLooper {
    ptr: *mut c_void,
}

impl Drop for PlayerLooper {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_player_looper_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl PlayerLooper {
    pub fn new(player: &QueuePlayer, template_item: &PlayerItem) -> Result<Self, AVPlayerError> {
        Self::with_time_range_and_ordering(
            player,
            template_item,
            None,
            PlayerLooperItemOrdering::LoopingItemsPrecedeExistingItems,
        )
    }

    pub fn with_time_range(
        player: &QueuePlayer,
        template_item: &PlayerItem,
        loop_range: TimeRange,
    ) -> Result<Self, AVPlayerError> {
        Self::with_time_range_and_ordering(
            player,
            template_item,
            Some(loop_range),
            PlayerLooperItemOrdering::LoopingItemsPrecedeExistingItems,
        )
    }

    pub fn with_time_range_and_ordering(
        player: &QueuePlayer,
        template_item: &PlayerItem,
        loop_range: Option<TimeRange>,
        item_ordering: PlayerLooperItemOrdering,
    ) -> Result<Self, AVPlayerError> {
        let (
            start_value,
            start_timescale,
            start_kind,
            duration_value,
            duration_timescale,
            duration_kind,
        ) = loop_range.map_or((0, 0, 1, 0, 0, 1), |loop_range| {
            let (start_value, start_timescale, start_kind) = loop_range.start.to_raw();
            let (duration_value, duration_timescale, duration_kind) = loop_range.duration.to_raw();
            (
                start_value,
                start_timescale,
                start_kind,
                duration_value,
                duration_timescale,
                duration_kind,
            )
        });
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_player_looper_create(
                player.ptr,
                template_item.ptr,
                loop_range.is_some(),
                start_value,
                start_timescale,
                start_kind,
                duration_value,
                duration_timescale,
                duration_kind,
                item_ordering.as_raw(),
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::PLAYER_CREATE_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn info(&self) -> Result<PlayerLooperInfoPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_player_looper_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn status(&self) -> Result<PlayerLooperStatus, AVPlayerError> {
        Ok(PlayerLooperStatus::from_raw(self.info()?.status))
    }

    pub fn error(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.error_message)
    }

    pub fn loop_count(&self) -> Result<i64, AVPlayerError> {
        Ok(self.info()?.loop_count)
    }

    pub fn looping_items(&self) -> Result<Vec<PlayerItem>, AVPlayerError> {
        let count = self.info()?.looping_item_count;
        if count < 0 {
            return Err(AVPlayerError::OperationFailed(format!(
                "looping item count unexpectedly negative: {count}"
            )));
        }
        let count = usize::try_from(count).map_err(|error| {
            AVPlayerError::OperationFailed(format!(
                "looping item count exceeds addressable size: {error}"
            ))
        })?;

        let mut items = Vec::with_capacity(count);
        for index in 0..count {
            let ptr = unsafe {
                ffi::av_player_looper_copy_looping_item_at_index(
                    self.ptr,
                    i32::try_from(index).map_err(|error| {
                        AVPlayerError::OperationFailed(format!(
                            "looping item index exceeds bridge range: {error}"
                        ))
                    })?,
                )
            };
            if ptr.is_null() {
                return Err(AVPlayerError::OperationFailed(format!(
                    "bridge returned null looping item at index {index}"
                )));
            }
            items.push(PlayerItem { ptr });
        }
        Ok(items)
    }

    pub fn disable_looping(&self) {
        unsafe { ffi::av_player_looper_disable_looping(self.ptr) };
    }
}
