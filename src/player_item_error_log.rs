#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;

use serde::Deserialize;

use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::player::PlayerItem;
use crate::util::parse_json_and_free;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ErrorLogPayload {
    extended_log: Option<String>,
    extended_log_data_string_encoding: usize,
    events: Vec<ErrorLogEventPayload>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
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

impl Drop for PlayerItemErrorLog {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_player_item_error_log_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

// SAFETY: AVPlayerItemErrorLog ObjC handles are safe to transfer across thread
// boundaries; method calls are internally dispatched safely.
unsafe impl Send for PlayerItemErrorLog {}

impl PlayerItemErrorLog {
    fn info(&self) -> Result<ErrorLogPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_player_item_error_log_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
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
