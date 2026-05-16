#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;

use serde::Deserialize;

use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::player::PlayerItem;
use crate::util::{json_cstring, parse_json_and_free};

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegibleOutputInfoPayload {
    suppresses_player_rendering: bool,
    advance_interval_for_delegate_invocation: f64,
    native_representation_subtypes: Vec<u32>,
}

pub struct PlayerItemLegibleOutput {
    pub(crate) ptr: *mut c_void,
}

impl Drop for PlayerItemLegibleOutput {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_player_item_output_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl PlayerItemLegibleOutput {
    pub fn new(native_representation_subtypes: Option<&[u32]>) -> Result<Self, AVPlayerError> {
        let native_representation_subtypes = native_representation_subtypes
            .map(|subtypes| json_cstring(subtypes, "native legible output subtypes"))
            .transpose()?;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_player_item_legible_output_create(
                native_representation_subtypes
                    .as_ref()
                    .map_or(ptr::null(), |subtypes| subtypes.as_ptr()),
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn info(&self) -> Result<LegibleOutputInfoPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_player_item_legible_output_info_json(self.ptr, &mut err) };
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

    pub fn advance_interval_for_delegate_invocation(&self) -> Result<f64, AVPlayerError> {
        Ok(self.info()?.advance_interval_for_delegate_invocation)
    }

    pub fn set_advance_interval_for_delegate_invocation(&self, interval: f64) {
        unsafe { ffi::av_player_item_legible_output_set_advance_interval(self.ptr, interval) };
    }

    pub fn native_representation_subtypes(&self) -> Result<Vec<u32>, AVPlayerError> {
        Ok(self.info()?.native_representation_subtypes)
    }
}

impl PlayerItem {
    pub fn add_legible_output(
        &self,
        output: &PlayerItemLegibleOutput,
    ) -> Result<(), AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe { ffi::av_player_item_add_output(self.ptr, output.ptr, &mut err) };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn remove_legible_output(&self, output: &PlayerItemLegibleOutput) {
        unsafe { ffi::av_player_item_remove_output(self.ptr, output.ptr) };
    }
}
