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
struct MetadataOutputInfoPayload {
    suppresses_player_rendering: bool,
    advance_interval_for_delegate_invocation: f64,
    identifiers: Option<Vec<String>>,
}

pub struct PlayerItemMetadataOutput {
    pub(crate) ptr: *mut c_void,
}

impl Drop for PlayerItemMetadataOutput {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_player_item_output_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl PlayerItemMetadataOutput {
    pub fn new(identifiers: Option<&[impl AsRef<str>]>) -> Result<Self, AVPlayerError> {
        let identifiers = identifiers.map(|identifiers| {
            identifiers
                .iter()
                .map(|identifier| identifier.as_ref().to_owned())
                .collect::<Vec<_>>()
        });
        let identifiers = identifiers
            .as_ref()
            .map(|identifiers| json_cstring(identifiers, "metadata output identifiers"))
            .transpose()?;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_player_item_metadata_output_create(
                identifiers
                    .as_ref()
                    .map_or(ptr::null(), |identifiers| identifiers.as_ptr()),
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn info(&self) -> Result<MetadataOutputInfoPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_player_item_metadata_output_info_json(self.ptr, &mut err) };
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
        unsafe { ffi::av_player_item_metadata_output_set_advance_interval(self.ptr, interval) };
    }

    pub fn identifiers(&self) -> Result<Vec<String>, AVPlayerError> {
        Ok(self.info()?.identifiers.unwrap_or_default())
    }
}

impl PlayerItem {
    pub fn add_metadata_output(
        &self,
        output: &PlayerItemMetadataOutput,
    ) -> Result<(), AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe { ffi::av_player_item_add_output(self.ptr, output.ptr, &mut err) };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn remove_metadata_output(&self, output: &PlayerItemMetadataOutput) {
        unsafe { ffi::av_player_item_remove_output(self.ptr, output.ptr) };
    }
}
