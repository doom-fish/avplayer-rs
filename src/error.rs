//! Errors produced by the `AVPlayer` / `AVAssetReader` bridge.

use core::fmt;

use crate::ffi;

/// Top-level error type returned by fallible APIs in this crate.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AVPlayerError {
    /// Invalid caller input (UTF-8 / NUL / unsupported configuration).
    InvalidArgument(String),
    /// Asset or URL-asset construction failed.
    AssetCreateFailed(String),
    /// Player or player-item creation failed.
    PlayerCreateFailed(String),
    /// Asset-reader construction failed.
    ReaderCreateFailed(String),
    /// An operation on an existing object failed.
    OperationFailed(String),
    /// Observer registration failed.
    ObserverFailed(String),
    /// Asynchronous key loading failed or timed out.
    LoadFailed(String),
}

impl fmt::Display for AVPlayerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgument(message) => write!(f, "invalid argument: {message}"),
            Self::AssetCreateFailed(message) => write!(f, "asset creation failed: {message}"),
            Self::PlayerCreateFailed(message) => write!(f, "player creation failed: {message}"),
            Self::ReaderCreateFailed(message) => {
                write!(f, "asset reader creation failed: {message}")
            }
            Self::OperationFailed(message) => write!(f, "operation failed: {message}"),
            Self::ObserverFailed(message) => write!(f, "observer registration failed: {message}"),
            Self::LoadFailed(message) => write!(f, "load failed: {message}"),
        }
    }
}

impl std::error::Error for AVPlayerError {}

pub unsafe fn from_swift(status: i32, error_str: *mut core::ffi::c_char) -> AVPlayerError {
    let message = if error_str.is_null() {
        String::new()
    } else {
        let s = core::ffi::CStr::from_ptr(error_str)
            .to_string_lossy()
            .into_owned();
        ffi::avp_string_free(error_str);
        s
    };

    match status {
        ffi::status::INVALID_ARGUMENT => AVPlayerError::InvalidArgument(message),
        ffi::status::ASSET_CREATE_FAILED => AVPlayerError::AssetCreateFailed(message),
        ffi::status::PLAYER_CREATE_FAILED => AVPlayerError::PlayerCreateFailed(message),
        ffi::status::READER_CREATE_FAILED => AVPlayerError::ReaderCreateFailed(message),
        ffi::status::OPERATION_FAILED => AVPlayerError::OperationFailed(message),
        ffi::status::OBSERVER_FAILED => AVPlayerError::ObserverFailed(message),
        ffi::status::LOAD_FAILED => AVPlayerError::LoadFailed(message),
        _ => AVPlayerError::OperationFailed(format!("unknown status {status}: {message}")),
    }
}
