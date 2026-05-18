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
struct AccessLogPayload {
    extended_log: Option<String>,
    extended_log_data_string_encoding: usize,
    events: Vec<AccessLogEventPayload>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AccessLogEventPayload {
    number_of_media_requests: i64,
    playback_start_date: Option<String>,
    uri: Option<String>,
    server_address: Option<String>,
    number_of_server_address_changes: i64,
    playback_session_id: Option<String>,
    playback_start_offset: f64,
    segments_downloaded_duration: f64,
    duration_watched: f64,
    number_of_stalls: i64,
    number_of_bytes_transferred: i64,
    transfer_duration: f64,
    observed_bitrate: f64,
    indicated_bitrate: f64,
    indicated_average_bitrate: f64,
    average_video_bitrate: f64,
    average_audio_bitrate: f64,
    number_of_dropped_video_frames: i64,
    startup_time: f64,
    download_overdue: i64,
    observed_bitrate_standard_deviation: f64,
    playback_type: Option<String>,
    media_requests_wwan: i64,
    switch_bitrate: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerItemAccessLogEvent {
    pub number_of_media_requests: i64,
    pub playback_start_date: Option<String>,
    pub uri: Option<String>,
    pub server_address: Option<String>,
    pub number_of_server_address_changes: i64,
    pub playback_session_id: Option<String>,
    pub playback_start_offset: f64,
    pub segments_downloaded_duration: f64,
    pub duration_watched: f64,
    pub number_of_stalls: i64,
    pub number_of_bytes_transferred: i64,
    pub transfer_duration: f64,
    pub observed_bitrate: f64,
    pub indicated_bitrate: f64,
    pub indicated_average_bitrate: f64,
    pub average_video_bitrate: f64,
    pub average_audio_bitrate: f64,
    pub number_of_dropped_video_frames: i64,
    pub startup_time: f64,
    pub download_overdue: i64,
    pub observed_bitrate_standard_deviation: f64,
    pub playback_type: Option<String>,
    pub media_requests_wwan: i64,
    pub switch_bitrate: f64,
}

impl From<AccessLogEventPayload> for PlayerItemAccessLogEvent {
    fn from(payload: AccessLogEventPayload) -> Self {
        Self {
            number_of_media_requests: payload.number_of_media_requests,
            playback_start_date: payload.playback_start_date,
            uri: payload.uri,
            server_address: payload.server_address,
            number_of_server_address_changes: payload.number_of_server_address_changes,
            playback_session_id: payload.playback_session_id,
            playback_start_offset: payload.playback_start_offset,
            segments_downloaded_duration: payload.segments_downloaded_duration,
            duration_watched: payload.duration_watched,
            number_of_stalls: payload.number_of_stalls,
            number_of_bytes_transferred: payload.number_of_bytes_transferred,
            transfer_duration: payload.transfer_duration,
            observed_bitrate: payload.observed_bitrate,
            indicated_bitrate: payload.indicated_bitrate,
            indicated_average_bitrate: payload.indicated_average_bitrate,
            average_video_bitrate: payload.average_video_bitrate,
            average_audio_bitrate: payload.average_audio_bitrate,
            number_of_dropped_video_frames: payload.number_of_dropped_video_frames,
            startup_time: payload.startup_time,
            download_overdue: payload.download_overdue,
            observed_bitrate_standard_deviation: payload.observed_bitrate_standard_deviation,
            playback_type: payload.playback_type,
            media_requests_wwan: payload.media_requests_wwan,
            switch_bitrate: payload.switch_bitrate,
        }
    }
}

#[derive(Debug)]
pub struct PlayerItemAccessLog {
    pub(crate) ptr: *mut c_void,
}

impl Drop for PlayerItemAccessLog {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_player_item_access_log_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

// SAFETY: AVPlayerItemAccessLog ObjC handles are safe to transfer across
// thread boundaries; method calls are internally dispatched safely.
unsafe impl Send for PlayerItemAccessLog {}

impl PlayerItemAccessLog {
    fn info(&self) -> Result<AccessLogPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_player_item_access_log_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn extended_log(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.extended_log)
    }

    pub fn extended_log_data_string_encoding(&self) -> Result<usize, AVPlayerError> {
        Ok(self.info()?.extended_log_data_string_encoding)
    }

    pub fn events(&self) -> Result<Vec<PlayerItemAccessLogEvent>, AVPlayerError> {
        Ok(self
            .info()?
            .events
            .into_iter()
            .map(PlayerItemAccessLogEvent::from)
            .collect())
    }
}

impl PlayerItem {
    pub fn access_log(&self) -> Result<Option<PlayerItemAccessLog>, AVPlayerError> {
        let ptr = unsafe { ffi::av_player_item_copy_access_log(self.ptr) };
        if ptr.is_null() {
            return Ok(None);
        }
        Ok(Some(PlayerItemAccessLog { ptr }))
    }
}
