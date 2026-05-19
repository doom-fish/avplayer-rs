#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::marker::PhantomData;
use std::ffi::CStr;

use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::player_item_metadata_collector::{
    PlayerItemMediaDataCollectorKind, PlayerItemMetadataCollector,
};

/// Borrowed `AVPlayerItemMediaDataCollector` view.
#[derive(Debug, Clone, Copy)]
pub struct PlayerItemMediaDataCollector<'a> {
    ptr: *mut c_void,
    _marker: PhantomData<&'a c_void>,
}

impl PlayerItemMediaDataCollector<'_> {
    pub(crate) const fn from_ptr(ptr: *mut c_void) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    pub fn kind(&self) -> Result<PlayerItemMediaDataCollectorKind, AVPlayerError> {
        let mut err: *mut c_char = core::ptr::null_mut();
        let raw = unsafe { ffi::av_player_item_media_data_collector_kind(self.ptr, &mut err) };
        if raw.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        let value = unsafe { CStr::from_ptr(raw) }
            .to_string_lossy()
            .into_owned();
        unsafe { ffi::avp_string_free(raw) };
        Ok(PlayerItemMediaDataCollectorKind::from_raw(&value))
    }
}

impl PlayerItemMetadataCollector {
    pub const fn as_media_data_collector(&self) -> PlayerItemMediaDataCollector<'_> {
        PlayerItemMediaDataCollector::from_ptr(self.ptr)
    }
}
