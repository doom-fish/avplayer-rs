#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use crate::ffi::AVPValue;
use core::ffi::c_void;
use core::marker::PhantomData;

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
        let mut err: *mut AVPValue = core::ptr::null_mut();
        let raw = unsafe { ffi::av_player_item_media_data_collector_kind(self.ptr, &raw mut err) };
        if raw.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        let value = unsafe { crate::binary::read_string(raw) }.unwrap_or_default();
        unsafe { ffi::avp_value_free(raw) };
        Ok(PlayerItemMediaDataCollectorKind::from_raw(&value))
    }
}

impl PlayerItemMetadataCollector {
    pub const fn as_media_data_collector(&self) -> PlayerItemMediaDataCollector<'_> {
        PlayerItemMediaDataCollector::from_ptr(self.ptr)
    }
}
