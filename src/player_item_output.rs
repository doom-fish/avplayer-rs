#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::marker::PhantomData;
use core::ptr;

use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::time::Time;
use crate::util::parse_json_and_free;

/// Borrowed view of the abstract `AVPlayerItemOutput` base class.
#[derive(Clone, Copy)]
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

    pub fn suppresses_player_rendering(&self) -> bool {
        unsafe { ffi::av_player_item_output_suppresses_player_rendering(self.ptr) }
    }

    pub fn set_suppresses_player_rendering(&self, suppresses: bool) {
        unsafe { ffi::av_player_item_output_set_suppresses_player_rendering(self.ptr, suppresses) };
    }

    pub fn item_time_for_host_time(&self, host_time_seconds: f64) -> Result<Time, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::av_player_item_output_item_time_for_host_time_json(
                self.ptr,
                host_time_seconds,
                &mut err,
            )
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn item_time_for_mach_absolute_time(
        &self,
        mach_absolute_time: i64,
    ) -> Result<Time, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::av_player_item_output_item_time_for_mach_absolute_time_json(
                self.ptr,
                mach_absolute_time,
                &mut err,
            )
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }
}
