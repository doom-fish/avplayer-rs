#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use crate::ffi::AVPValue;
use core::ffi::c_void;
use core::ptr;

use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::player::PlayerItem;
use crate::retained::retain_release_wrapper;
use crate::util::parse_value_and_free;

#[derive(Debug, Clone, PartialEq)]
struct ErrorLogPayload {
    extended_log: Option<String>,
    extended_log_data_string_encoding: usize,
    events: Vec<ErrorLogEventPayload>,
}

#[derive(Debug, Clone, PartialEq)]
struct ErrorLogEventPayload {
    date: Option<String>,
    uri: Option<String>,
    server_address: Option<String>,
    playback_session_id: Option<String>,
    error_status_code: i64,
    error_domain: String,
    error_comment: Option<String>,
    all_http_response_header_fields: Option<std::collections::BTreeMap<String, String>>,
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerItemErrorLogEvent`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerItemErrorLogEvent {
    /// Mirrors the `AVPlayer` framework property for `date`.
    pub date: Option<String>,
    /// Mirrors the `AVPlayer` framework property for `uri`.
    pub uri: Option<String>,
    /// Mirrors the `AVPlayer` framework property for `server_address`.
    pub server_address: Option<String>,
    /// Mirrors the `AVPlayer` framework property for `playback_session_id`.
    pub playback_session_id: Option<String>,
    /// Mirrors the `AVPlayer` framework property for `error_status_code`.
    pub error_status_code: i64,
    /// Mirrors the `AVPlayer` framework property for `error_domain`.
    pub error_domain: String,
    /// Mirrors the `AVPlayer` framework property for `error_comment`.
    pub error_comment: Option<String>,
    /// Mirrors the `AVPlayer` framework property for `all_http_response_header_fields`.
    pub all_http_response_header_fields: Option<std::collections::BTreeMap<String, String>>,
}

impl From<ErrorLogEventPayload> for PlayerItemErrorLogEvent {
    fn from(payload: ErrorLogEventPayload) -> Self {
        Self {
            date: payload.date,
            uri: payload.uri,
            server_address: payload.server_address,
            playback_session_id: payload.playback_session_id,
            error_status_code: payload.error_status_code,
            error_domain: payload.error_domain,
            error_comment: payload.error_comment,
            all_http_response_header_fields: payload.all_http_response_header_fields,
        }
    }
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerItemErrorLog`.
#[derive(Debug)]
pub struct PlayerItemErrorLog {
    pub(crate) ptr: *mut c_void,
}

retain_release_wrapper!(
    PlayerItemErrorLog,
    release = ffi::av_player_item_error_log_release
);

// SAFETY: AVPlayerItemErrorLog ObjC handles are safe to transfer across thread
// boundaries; method calls are internally dispatched safely.
unsafe impl Send for PlayerItemErrorLog {}

impl PlayerItemErrorLog {
    fn info(&self) -> Result<ErrorLogPayload, AVPlayerError> {
        let mut err: *mut AVPValue = ptr::null_mut();
        let value_ptr = unsafe { ffi::av_player_item_error_log_info_value(self.ptr, &raw mut err) };
        if value_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_value_and_free(value_ptr)
    }

    /// Calls the `AVPlayer` framework counterpart for `extended_log`.
    pub fn extended_log(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.extended_log)
    }

    /// Calls the `AVPlayer` framework counterpart for `extended_log_data_string_encoding`.
    pub fn extended_log_data_string_encoding(&self) -> Result<usize, AVPlayerError> {
        Ok(self.info()?.extended_log_data_string_encoding)
    }

    /// Calls the `AVPlayer` framework counterpart for `events`.
    pub fn events(&self) -> Result<Vec<PlayerItemErrorLogEvent>, AVPlayerError> {
        Ok(self
            .info()?
            .events
            .into_iter()
            .map(PlayerItemErrorLogEvent::from)
            .collect())
    }
}

impl PlayerItem {
    /// Calls the `AVPlayer` framework counterpart for `error_log`.
    pub fn error_log(&self) -> Result<Option<PlayerItemErrorLog>, AVPlayerError> {
        let ptr = unsafe { ffi::av_player_item_copy_error_log(self.ptr) };
        if ptr.is_null() {
            return Ok(None);
        }
        Ok(Some(PlayerItemErrorLog { ptr }))
    }
}

impl crate::binary::FromValue for ErrorLogPayload {
    fn from_value(value: &crate::binary::Value) -> Result<Self, AVPlayerError> {
        Ok(Self {
            extended_log: value.field(0)?,
            extended_log_data_string_encoding: value.field(1)?,
            events: value.field(2)?,
        })
    }
}

impl crate::binary::FromValue for ErrorLogEventPayload {
    fn from_value(value: &crate::binary::Value) -> Result<Self, AVPlayerError> {
        Ok(Self {
            date: value.field(0)?,
            uri: value.field(1)?,
            server_address: value.field(2)?,
            playback_session_id: value.field(3)?,
            error_status_code: value.field(4)?,
            error_domain: value.field(5)?,
            error_comment: value.field(6)?,
            all_http_response_header_fields: value.field(7)?,
        })
    }
}
