#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::c_char;
use std::ffi::{CStr, CString};

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::AVPlayerError;
use crate::ffi;

pub fn parse_json_and_free<T: DeserializeOwned>(json_ptr: *mut c_char) -> Result<T, AVPlayerError> {
    let json = unsafe { CStr::from_ptr(json_ptr) }
        .to_string_lossy()
        .into_owned();
    unsafe { ffi::avp_string_free(json_ptr) };
    serde_json::from_str::<T>(&json).map_err(|error| {
        AVPlayerError::OperationFailed(format!("failed to decode bridge JSON: {error}"))
    })
}

pub fn to_cstring(value: &str, what: &str) -> Result<CString, AVPlayerError> {
    CString::new(value).map_err(|error| {
        AVPlayerError::InvalidArgument(format!("{what} contains NUL byte: {error}"))
    })
}

pub fn json_cstring<T: Serialize + ?Sized>(
    value: &T,
    what: &str,
) -> Result<CString, AVPlayerError> {
    let json = serde_json::to_string(value).map_err(|error| {
        AVPlayerError::InvalidArgument(format!("failed to encode {what}: {error}"))
    })?;
    to_cstring(&json, &format!("{what} JSON"))
}

pub fn maybe_json_cstring<T: Serialize>(
    value: Option<&T>,
    what: &str,
) -> Result<Option<CString>, AVPlayerError> {
    value.map(|value| json_cstring(value, what)).transpose()
}

/// Catch a panic inside an `extern "C"` callback so it cannot unwind
/// across the FFI boundary into Swift (which is undefined behaviour).
pub fn catch_cb_panic<F: FnOnce()>(site: &str, f: F) {
    use std::panic::AssertUnwindSafe;

    if let Err(payload) = std::panic::catch_unwind(AssertUnwindSafe(f)) {
        let msg = payload.downcast_ref::<&str>().copied().unwrap_or_else(|| {
            payload
                .downcast_ref::<String>()
                .map_or("<non-string panic>", String::as_str)
        });
        eprintln!("avplayer: panic in {site} caught at C ABI boundary: {msg}");
    }
}
