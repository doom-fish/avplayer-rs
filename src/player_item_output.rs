#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use crate::ffi::AVPValue;
use core::ffi::c_void;
use core::marker::PhantomData;
use core::ptr;

use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::time::Time;
use crate::util::parse_value_and_free;

/// Borrowed view of the abstract `AVPlayerItemOutput` base class.
#[derive(Debug, Clone, Copy)]
pub struct PlayerItemOutput<'a> {
    pub(crate) ptr: *mut c_void,
    _marker: PhantomData<&'a c_void>,
}

// SAFETY: Borrowed AVPlayerItemOutput handles are safe to transfer across
// thread boundaries while borrowed; method calls are internally dispatched
// safely.
unsafe impl Send for PlayerItemOutput<'_> {}

impl PlayerItemOutput<'_> {
    pub(crate) const fn from_ptr(ptr: *mut c_void) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    /// Calls the `AVPlayer` framework counterpart for `suppresses_player_rendering`.
    pub fn suppresses_player_rendering(&self) -> bool {
        unsafe { ffi::av_player_item_output_suppresses_player_rendering(self.ptr) }
    }

    /// Calls the `AVPlayer` framework counterpart for `set_suppresses_player_rendering`.
    pub fn set_suppresses_player_rendering(&self, suppresses: bool) {
        unsafe { ffi::av_player_item_output_set_suppresses_player_rendering(self.ptr, suppresses) };
    }

    /// Calls the `AVPlayer` framework counterpart for `item_time_for_host_time`.
    pub fn item_time_for_host_time(&self, host_time_seconds: f64) -> Result<Time, AVPlayerError> {
        let mut err: *mut AVPValue = ptr::null_mut();
        let value_ptr = unsafe {
            ffi::av_player_item_output_item_time_for_host_time_value(
                self.ptr,
                host_time_seconds,
                &raw mut err,
            )
        };
        if value_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_value_and_free(value_ptr)
    }

    /// Calls the `AVPlayer` framework counterpart for `item_time_for_mach_absolute_time`.
    pub fn item_time_for_mach_absolute_time(
        &self,
        mach_absolute_time: i64,
    ) -> Result<Time, AVPlayerError> {
        let mut err: *mut AVPValue = ptr::null_mut();
        let value_ptr = unsafe {
            ffi::av_player_item_output_item_time_for_mach_absolute_time_value(
                self.ptr,
                mach_absolute_time,
                &raw mut err,
            )
        };
        if value_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_value_and_free(value_ptr)
    }
}
