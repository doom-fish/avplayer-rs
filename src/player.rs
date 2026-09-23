#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use std::ffi::{CStr, CString};
use std::path::Path;
use std::sync::{Mutex, PoisonError};

use doom_fish_utils::callback_context::CallbackContext;

use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::asset::{Asset, Size};
use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::metadata::MetadataItem;
use crate::player_media_selection_criteria::PlayerTimeControlStatus;
use crate::retained::retain_release_wrapper;
use crate::time::Time;
use crate::util::{
    deliver, json_payload, validate_seek_time, validate_tolerance, Handler, Registration,
};

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerInfoPayload {
    status: i32,
    error_message: Option<String>,
    rate: f32,
    current_time: Time,
    duration: Time,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerItemInfoPayload {
    status: i32,
    error_message: Option<String>,
    duration: Time,
    presentation_size: Size,
    metadata: Vec<MetadataItem>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerItemEventPayload {
    event: String,
    status: Option<i32>,
    error_message: Option<String>,
    presentation_size: Option<Size>,
    has_originating_participant: Option<bool>,
    recommended_time_offset_from_live: Option<Time>,
}

/// `AVPlayerStatus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PlayerStatus {
    /// Mirrors the `AVPlayer` framework case `Unknown`.
    Unknown,
    /// Mirrors the `AVPlayer` framework case `ReadyToPlay`.
    ReadyToPlay,
    /// Mirrors the `AVPlayer` framework case `Failed`.
    Failed,
}

impl PlayerStatus {
    /// Mirrors the `AVPlayer` framework constant `fn`.
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            1 => Self::ReadyToPlay,
            2 => Self::Failed,
            _ => Self::Unknown,
        }
    }
}

/// `AVPlayerItemStatus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PlayerItemStatus {
    /// Mirrors the `AVPlayer` framework case `Unknown`.
    Unknown,
    /// Mirrors the `AVPlayer` framework case `ReadyToPlay`.
    ReadyToPlay,
    /// Mirrors the `AVPlayer` framework case `Failed`.
    Failed,
}

impl PlayerItemStatus {
    /// Mirrors the `AVPlayer` framework constant `fn`.
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            1 => Self::ReadyToPlay,
            2 => Self::Failed,
            _ => Self::Unknown,
        }
    }
}

/// Events emitted by `PlayerItemObserver`.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum PlayerItemEvent {
    /// Mirrors the `AVPlayer` framework case `StatusChanged`.
    StatusChanged {
        status: PlayerItemStatus,
        error_message: Option<String>,
    },
    /// Mirrors the `AVPlayer` framework case `PresentationSizeChanged`.
    PresentationSizeChanged(Size),
    /// Mirrors the `AVPlayer` framework case `TimeJumped`.
    TimeJumped { has_originating_participant: bool },
    /// Mirrors the `AVPlayer` framework case `DidPlayToEnd`.
    DidPlayToEnd,
    /// Mirrors the `AVPlayer` framework case `FailedToPlayToEnd`.
    FailedToPlayToEnd { error_message: Option<String> },
    /// Mirrors the `AVPlayer` framework case `PlaybackStalled`.
    PlaybackStalled,
    /// Mirrors the `AVPlayer` framework case `NewAccessLogEntry`.
    NewAccessLogEntry,
    /// Mirrors the `AVPlayer` framework case `NewErrorLogEntry`.
    NewErrorLogEntry,
    /// Mirrors the `AVPlayer` framework case `RecommendedTimeOffsetFromLiveDidChange`.
    RecommendedTimeOffsetFromLiveDidChange(Time),
    /// Mirrors the `AVPlayer` framework case `MediaSelectionChanged`.
    MediaSelectionChanged,
}

type PeriodicTimeHandler = Mutex<Box<dyn FnMut(Time) + Send + 'static>>;
type BoundaryTimeHandler = Mutex<Box<dyn FnMut() + Send + 'static>>;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerStatusEventPayload {
    event: String,
    status: Option<i32>,
    error_message: Option<String>,
    time_control_status: Option<i32>,
    reason_for_waiting_to_play: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PlayerStatusEvent {
    StatusChanged {
        status: PlayerStatus,
        error_message: Option<String>,
    },
    TimeControlStatusChanged {
        time_control_status: PlayerTimeControlStatus,
        reason_for_waiting_to_play: Option<String>,
    },
}

impl PlayerStatusEvent {
    fn from_payload(payload: PlayerStatusEventPayload) -> Option<Self> {
        match payload.event.as_str() {
            "status_changed" => Some(Self::StatusChanged {
                status: PlayerStatus::from_raw(payload.status?),
                error_message: payload.error_message,
            }),
            "time_control_status_changed" => Some(Self::TimeControlStatusChanged {
                time_control_status: PlayerTimeControlStatus::from_raw(
                    payload.time_control_status?,
                ),
                reason_for_waiting_to_play: payload.reason_for_waiting_to_play,
            }),
            _ => None,
        }
    }
}

/// Safe wrapper around `AVPlayerItem`.
#[derive(Debug)]
pub struct PlayerItem {
    pub(crate) ptr: *mut c_void,
}

retain_release_wrapper!(PlayerItem, release = ffi::av_player_item_release);

impl PlayerItem {
    /// Create a player item from a file path.
    pub fn from_file_path(path: impl AsRef<Path>) -> Result<Self, AVPlayerError> {
        let path = path
            .as_ref()
            .to_str()
            .ok_or_else(|| AVPlayerError::InvalidArgument("path is not valid UTF-8".into()))?;
        Self::from_url_internal(path, true)
    }

    /// Create a player item from a remote URL.
    pub fn from_remote_url(url: impl AsRef<str>) -> Result<Self, AVPlayerError> {
        Self::from_url_internal(url.as_ref(), false)
    }

    /// Create a player item from an existing asset.
    pub fn from_asset(asset: &Asset) -> Result<Self, AVPlayerError> {
        let keys_json =
            CString::new("[\"duration\",\"tracks\",\"metadata\"]").map_err(|error| {
                AVPlayerError::InvalidArgument(format!("asset-key JSON contains NUL byte: {error}"))
            })?;
        let mut err: *mut c_char = ptr::null_mut();
        // SAFETY: `asset.ptr` is a valid borrowed AVAsset handle, `keys_json` is a
        // NUL-terminated string, and `err` points to writable storage for the bridge.
        let ptr = unsafe {
            ffi::av_player_item_create_with_asset(asset.ptr, keys_json.as_ptr(), &raw mut err)
        };
        if ptr.is_null() {
            // SAFETY: On failure the bridge initializes `err` to an owned Swift error
            // payload that `from_swift` consumes.
            return Err(unsafe { from_swift(ffi::status::PLAYER_CREATE_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn from_url_internal(url: &str, is_file_url: bool) -> Result<Self, AVPlayerError> {
        let url = CString::new(url).map_err(|error| {
            AVPlayerError::InvalidArgument(format!("URL contains NUL byte: {error}"))
        })?;
        let keys_json =
            CString::new("[\"duration\",\"tracks\",\"metadata\"]").map_err(|error| {
                AVPlayerError::InvalidArgument(format!("asset-key JSON contains NUL byte: {error}"))
            })?;
        let mut err: *mut c_char = ptr::null_mut();
        // SAFETY: `url` and `keys_json` are NUL-terminated strings owned by this
        // frame, and `err` points to writable storage for the bridge.
        let ptr = unsafe {
            ffi::av_player_item_create_with_url(
                url.as_ptr(),
                is_file_url,
                keys_json.as_ptr(),
                &raw mut err,
            )
        };
        if ptr.is_null() {
            // SAFETY: On failure the bridge initializes `err` to an owned Swift error
            // payload that `from_swift` consumes.
            return Err(unsafe { from_swift(ffi::status::PLAYER_CREATE_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn info(&self) -> Result<PlayerItemInfoPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        // SAFETY: `self.ptr` is a valid AVPlayerItem handle and `err` points to
        // writable storage for the bridge.
        let json_ptr = unsafe { ffi::av_player_item_info_json(self.ptr, &raw mut err) };
        if json_ptr.is_null() {
            // SAFETY: On failure the bridge initializes `err` to an owned Swift error
            // payload that `from_swift` consumes.
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Calls the `AVPlayer` framework counterpart for `status`.
    pub fn status(&self) -> Result<PlayerItemStatus, AVPlayerError> {
        Ok(PlayerItemStatus::from_raw(self.info()?.status))
    }

    /// Calls the `AVPlayer` framework counterpart for `error`.
    pub fn error(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.error_message)
    }

    /// Calls the `AVPlayer` framework counterpart for `duration`.
    pub fn duration(&self) -> Result<Time, AVPlayerError> {
        Ok(self.info()?.duration)
    }

    /// Calls the `AVPlayer` framework counterpart for `presentation_size`.
    pub fn presentation_size(&self) -> Result<Size, AVPlayerError> {
        Ok(self.info()?.presentation_size)
    }

    /// The current macOS SDK does not expose `externalMetadata`; this returns
    /// the underlying asset metadata instead.
    pub fn metadata(&self) -> Result<Vec<MetadataItem>, AVPlayerError> {
        Ok(self.info()?.metadata)
    }

    /// Calls the `AVPlayer` framework counterpart for `observe`.
    pub fn observe<F>(&self, callback: F) -> Result<PlayerItemObserver, AVPlayerError>
    where
        F: Fn(PlayerItemEvent) + Send + Sync + 'static,
    {
        let handler: Handler<PlayerItemEvent> = Box::new(callback);
        let inner = Registration::new(
            handler,
            ffi::av_player_item_observer_release,
            |userdata, drop_userdata, err| unsafe {
                ffi::av_player_item_add_observer(
                    self.ptr,
                    Some(player_item_event_trampoline),
                    userdata,
                    drop_userdata,
                    err,
                )
            },
        )?;
        Ok(PlayerItemObserver { _inner: inner })
    }
}

/// KVO + notification observer for `AVPlayerItem`.
#[derive(Debug)]
pub struct PlayerItemObserver {
    _inner: Registration,
}

/// Safe wrapper around `AVPlayer`.
#[derive(Debug)]
pub struct Player {
    pub(crate) ptr: *mut c_void,
}

retain_release_wrapper!(Player, release = ffi::av_player_release);

impl Player {
    /// Create a player from a file path.
    pub fn from_file_path(path: impl AsRef<Path>) -> Result<Self, AVPlayerError> {
        let path = path
            .as_ref()
            .to_str()
            .ok_or_else(|| AVPlayerError::InvalidArgument("path is not valid UTF-8".into()))?;
        Self::from_url_internal(path, true)
    }

    /// Create a player from a remote URL.
    pub fn from_remote_url(url: impl AsRef<str>) -> Result<Self, AVPlayerError> {
        Self::from_url_internal(url.as_ref(), false)
    }

    /// Create a player that plays the supplied asset.
    pub fn from_asset(asset: &Asset) -> Result<Self, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        // SAFETY: `asset.ptr` is a valid borrowed AVAsset handle and `err` points
        // to writable storage for the bridge.
        let ptr = unsafe { ffi::av_player_create_with_asset(asset.ptr, &raw mut err) };
        if ptr.is_null() {
            // SAFETY: On failure the bridge initializes `err` to an owned Swift error
            // payload that `from_swift` consumes.
            return Err(unsafe { from_swift(ffi::status::PLAYER_CREATE_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    /// Create a player from an already-configured item.
    pub fn from_item(item: &PlayerItem) -> Result<Self, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        // SAFETY: `item.ptr` is a valid borrowed AVPlayerItem handle and `err`
        // points to writable storage for the bridge.
        let ptr = unsafe { ffi::av_player_create_with_item(item.ptr, &raw mut err) };
        if ptr.is_null() {
            // SAFETY: On failure the bridge initializes `err` to an owned Swift error
            // payload that `from_swift` consumes.
            return Err(unsafe { from_swift(ffi::status::PLAYER_CREATE_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn from_url_internal(url: &str, is_file_url: bool) -> Result<Self, AVPlayerError> {
        let url = CString::new(url).map_err(|error| {
            AVPlayerError::InvalidArgument(format!("URL contains NUL byte: {error}"))
        })?;
        let mut err: *mut c_char = ptr::null_mut();
        // SAFETY: `url` is a NUL-terminated string owned by this frame and `err`
        // points to writable storage for the bridge.
        let ptr =
            unsafe { ffi::av_player_create_with_url(url.as_ptr(), is_file_url, &raw mut err) };
        if ptr.is_null() {
            // SAFETY: On failure the bridge initializes `err` to an owned Swift error
            // payload that `from_swift` consumes.
            return Err(unsafe { from_swift(ffi::status::PLAYER_CREATE_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn info(&self) -> Result<PlayerInfoPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        // SAFETY: `self.ptr` is a valid AVPlayer handle and `err` points to
        // writable storage for the bridge.
        let json_ptr = unsafe { ffi::av_player_info_json(self.ptr, &raw mut err) };
        if json_ptr.is_null() {
            // SAFETY: On failure the bridge initializes `err` to an owned Swift error
            // payload that `from_swift` consumes.
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Calls the `AVPlayer` framework counterpart for `status`.
    pub fn status(&self) -> Result<PlayerStatus, AVPlayerError> {
        Ok(PlayerStatus::from_raw(self.info()?.status))
    }

    /// Calls the `AVPlayer` framework counterpart for `error`.
    pub fn error(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.error_message)
    }

    /// Calls the `AVPlayer` framework counterpart for `rate`.
    pub fn rate(&self) -> Result<f32, AVPlayerError> {
        Ok(self.info()?.rate)
    }

    /// Calls the `AVPlayer` framework counterpart for `current_time`.
    pub fn current_time(&self) -> Result<Time, AVPlayerError> {
        Ok(self.info()?.current_time)
    }

    /// Calls the `AVPlayer` framework counterpart for `duration`.
    pub fn duration(&self) -> Result<Time, AVPlayerError> {
        Ok(self.info()?.duration)
    }

    /// Calls the `AVPlayer` framework counterpart for `current_item`.
    pub fn current_item(&self) -> Option<PlayerItem> {
        // SAFETY: `self.ptr` is a valid AVPlayer handle; the bridge returns either
        // a retained current item or null when no item is set.
        let ptr = unsafe { ffi::av_player_copy_current_item(self.ptr) };
        if ptr.is_null() {
            None
        } else {
            Some(PlayerItem { ptr })
        }
    }

    /// Calls the `AVPlayer` framework counterpart for `play`.
    pub fn play(&self) {
        // SAFETY: `self.ptr` is a valid, non-null handle returned by the corresponding
        // ffi create function and has not been released.
        unsafe { ffi::av_player_play(self.ptr) };
    }

    /// Calls the `AVPlayer` framework counterpart for `pause`.
    pub fn pause(&self) {
        // SAFETY: `self.ptr` is a valid, non-null handle returned by the corresponding
        // ffi create function and has not been released.
        unsafe { ffi::av_player_pause(self.ptr) };
    }

    /// Calls the `AVPlayer` framework counterpart for `set_rate`.
    pub fn set_rate(&self, rate: f32) {
        // SAFETY: `self.ptr` is a valid, non-null handle returned by the corresponding
        // ffi create function and has not been released.
        unsafe { ffi::av_player_set_rate(self.ptr, rate) };
    }

    /// Calls the `AVPlayer` framework counterpart for `seek_to`.
    pub fn seek_to(&self, time: Time) -> Result<(), AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let (value, timescale, kind) = time.to_raw();
        // SAFETY: `self.ptr` is a valid AVPlayer handle and `err` points to
        // writable storage for the bridge.
        let status = unsafe { ffi::av_player_seek(self.ptr, value, timescale, kind, &raw mut err) };
        if status != ffi::status::OK {
            // SAFETY: On failure the bridge initializes `err` to an owned Swift error
            // payload that `from_swift` consumes.
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn seek_to_with_tolerance(
        &self,
        time: Time,
        tolerance_before: Time,
        tolerance_after: Time,
    ) -> Result<(), AVPlayerError> {
        validate_seek_time(time, "seek time")?;
        validate_tolerance(tolerance_before, "tolerance before")?;
        validate_tolerance(tolerance_after, "tolerance after")?;
        let (value, timescale, kind) = time.to_raw();
        let (before_value, before_timescale, before_kind) = tolerance_before.to_raw();
        let (after_value, after_timescale, after_kind) = tolerance_after.to_raw();
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::av_player_seek_with_tolerance(
                self.ptr,
                value,
                timescale,
                kind,
                before_value,
                before_timescale,
                before_kind,
                after_value,
                after_timescale,
                after_kind,
                &raw mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn set_rate_at_host_time(
        &self,
        rate: f32,
        item_time: Time,
        host_time: Time,
    ) -> Result<(), AVPlayerError> {
        let (item_value, item_timescale, item_kind) = item_time.to_raw();
        let (host_value, host_timescale, host_kind) = host_time.to_raw();
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::av_player_set_rate_at_host_time(
                self.ptr,
                rate,
                item_value,
                item_timescale,
                item_kind,
                host_value,
                host_timescale,
                host_kind,
                &raw mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn observe_status<F>(&self, callback: F) -> Result<PlayerStatusObserver, AVPlayerError>
    where
        F: Fn(PlayerStatusEvent) + Send + Sync + 'static,
    {
        let handler: Handler<PlayerStatusEvent> = Box::new(callback);
        let inner = Registration::new(
            handler,
            ffi::av_player_status_observer_release,
            |userdata, drop_userdata, err| unsafe {
                ffi::av_player_add_status_observer(
                    self.ptr,
                    Some(player_status_event_trampoline),
                    userdata,
                    drop_userdata,
                    err,
                )
            },
        )?;
        Ok(PlayerStatusObserver { _inner: inner })
    }

    /// Calls the `AVPlayer` framework counterpart for `add_periodic_time_observer`.
    pub fn add_periodic_time_observer<F>(
        &self,
        interval: Time,
        queue_label: Option<&str>,
        callback: F,
    ) -> Result<PeriodicTimeObserver, AVPlayerError>
    where
        F: FnMut(Time) + Send + 'static,
    {
        let queue_label = queue_label_cstring(queue_label)?;
        let (value, timescale, kind) = interval.to_raw();
        let handler: PeriodicTimeHandler = Mutex::new(Box::new(callback));
        let inner = Registration::new(
            handler,
            ffi::av_player_time_observer_release,
            |userdata, drop_userdata, err| unsafe {
                ffi::av_player_add_periodic_time_observer(
                    self.ptr,
                    value,
                    timescale,
                    kind,
                    queue_label
                        .as_ref()
                        .map_or(ptr::null(), |label| label.as_ptr()),
                    Some(periodic_time_observer_trampoline),
                    userdata,
                    drop_userdata,
                    err,
                )
            },
        )?;
        Ok(PeriodicTimeObserver { _inner: inner })
    }

    /// Calls the `AVPlayer` framework counterpart for `add_boundary_time_observer`.
    pub fn add_boundary_time_observer<F>(
        &self,
        times: &[Time],
        queue_label: Option<&str>,
        callback: F,
    ) -> Result<BoundaryTimeObserver, AVPlayerError>
    where
        F: FnMut() + Send + 'static,
    {
        let queue_label = queue_label_cstring(queue_label)?;
        let times_json = serde_json::to_string(times).map_err(|error| {
            AVPlayerError::InvalidArgument(format!("failed to encode boundary times: {error}"))
        })?;
        let times_json = CString::new(times_json).map_err(|error| {
            AVPlayerError::InvalidArgument(format!(
                "boundary times JSON contains NUL byte: {error}"
            ))
        })?;
        let handler: BoundaryTimeHandler = Mutex::new(Box::new(callback));
        let inner = Registration::new(
            handler,
            ffi::av_player_time_observer_release,
            |userdata, drop_userdata, err| unsafe {
                ffi::av_player_add_boundary_time_observer(
                    self.ptr,
                    times_json.as_ptr(),
                    queue_label
                        .as_ref()
                        .map_or(ptr::null(), |label| label.as_ptr()),
                    Some(boundary_time_observer_trampoline),
                    userdata,
                    drop_userdata,
                    err,
                )
            },
        )?;
        Ok(BoundaryTimeObserver { _inner: inner })
    }
}

/// RAII token for `addPeriodicTimeObserver`.
#[derive(Debug)]
pub struct PeriodicTimeObserver {
    _inner: Registration,
}

/// RAII token for `addBoundaryTimeObserver`.
#[derive(Debug)]
pub struct BoundaryTimeObserver {
    _inner: Registration,
}

#[derive(Debug)]
pub struct PlayerStatusObserver {
    _inner: Registration,
}

// SAFETY: AVPlayer / AVPlayerItem ObjC handles and observer tokens are safe to
// transfer across thread boundaries; method calls are internally dispatched
// safely.
unsafe impl Send for PlayerItem {}
unsafe impl Send for PlayerItemObserver {}
unsafe impl Send for Player {}
unsafe impl Send for PeriodicTimeObserver {}
unsafe impl Send for BoundaryTimeObserver {}
unsafe impl Send for PlayerStatusObserver {}

unsafe extern "C" fn player_item_event_trampoline(
    userdata: *mut c_void,
    payload_json: *const c_char,
) {
    let Some(payload) = (unsafe { json_payload::<PlayerItemEventPayload>(payload_json) }) else {
        return;
    };

    let event = match payload.event.as_str() {
        "status_changed" => PlayerItemEvent::StatusChanged {
            status: PlayerItemStatus::from_raw(payload.status.unwrap_or_default()),
            error_message: payload.error_message,
        },
        "presentation_size_changed" => match payload.presentation_size {
            Some(size) => PlayerItemEvent::PresentationSizeChanged(size),
            None => return,
        },
        "time_jumped" => PlayerItemEvent::TimeJumped {
            has_originating_participant: payload.has_originating_participant.unwrap_or(false),
        },
        "did_play_to_end" => PlayerItemEvent::DidPlayToEnd,
        "failed_to_play_to_end" => PlayerItemEvent::FailedToPlayToEnd {
            error_message: payload.error_message,
        },
        "playback_stalled" => PlayerItemEvent::PlaybackStalled,
        "new_access_log_entry" => PlayerItemEvent::NewAccessLogEntry,
        "new_error_log_entry" => PlayerItemEvent::NewErrorLogEntry,
        "recommended_time_offset_from_live_did_change" => {
            match payload.recommended_time_offset_from_live {
                Some(time) => PlayerItemEvent::RecommendedTimeOffsetFromLiveDidChange(time),
                None => return,
            }
        }
        "media_selection_changed" => PlayerItemEvent::MediaSelectionChanged,
        _ => return,
    };

    unsafe { deliver(userdata, "player_item_event_trampoline", event) };
}

unsafe extern "C" fn player_status_event_trampoline(
    userdata: *mut c_void,
    payload_json: *const c_char,
) {
    let Some(event) = (unsafe { json_payload::<PlayerStatusEventPayload>(payload_json) })
        .and_then(PlayerStatusEvent::from_payload)
    else {
        return;
    };
    unsafe { deliver(userdata, "player_status_event_trampoline", event) };
}

unsafe extern "C" fn periodic_time_observer_trampoline(
    userdata: *mut c_void,
    value: i64,
    timescale: i32,
    kind: i32,
) {
    let time = Time::from_raw(value, timescale, kind);
    let _ = unsafe {
        CallbackContext::<PeriodicTimeHandler>::with(
            userdata,
            "periodic_time_observer_trampoline",
            |handler| {
                let mut callback = handler.lock().unwrap_or_else(PoisonError::into_inner);
                callback(time);
            },
        )
    };
}

unsafe extern "C" fn boundary_time_observer_trampoline(userdata: *mut c_void) {
    let _ = unsafe {
        CallbackContext::<BoundaryTimeHandler>::with(
            userdata,
            "boundary_time_observer_trampoline",
            |handler| {
                let mut callback = handler.lock().unwrap_or_else(PoisonError::into_inner);
                callback();
            },
        )
    };
}

fn queue_label_cstring(queue_label: Option<&str>) -> Result<Option<CString>, AVPlayerError> {
    queue_label
        .map(|label| {
            CString::new(label).map_err(|error| {
                AVPlayerError::InvalidArgument(format!("queue label contains NUL byte: {error}"))
            })
        })
        .transpose()
}

fn parse_json_and_free<T: DeserializeOwned>(json_ptr: *mut c_char) -> Result<T, AVPlayerError> {
    // SAFETY: `json_ptr` is a non-null, NUL-terminated C string returned by the
    // bridge; it remains valid until freed with `avp_string_free`.
    let json = unsafe { CStr::from_ptr(json_ptr) }
        .to_string_lossy()
        .into_owned();
    // SAFETY: `json_ptr` was returned by the FFI and has not been freed yet.
    unsafe { ffi::avp_string_free(json_ptr) };
    serde_json::from_str::<T>(&json).map_err(|error| {
        AVPlayerError::OperationFailed(format!("failed to decode bridge JSON: {error}"))
    })
}
