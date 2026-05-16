#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use std::ffi::{CStr, CString};
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::asset::{Asset, Size};
use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::metadata::MetadataItem;
use crate::time::Time;

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
}

/// `AVPlayerStatus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PlayerStatus {
    Unknown,
    ReadyToPlay,
    Failed,
}

impl PlayerStatus {
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
    Unknown,
    ReadyToPlay,
    Failed,
}

impl PlayerItemStatus {
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
    StatusChanged {
        status: PlayerItemStatus,
        error_message: Option<String>,
    },
    PresentationSizeChanged(Size),
    DidPlayToEnd,
}

struct PlayerItemObserverState {
    callback: Box<dyn Fn(PlayerItemEvent) + Send + 'static>,
}

struct PeriodicTimeObserverState {
    callback: Box<dyn FnMut(Time) + Send + 'static>,
}

struct BoundaryTimeObserverState {
    callback: Box<dyn FnMut() + Send + 'static>,
}

/// Safe wrapper around `AVPlayerItem`.
pub struct PlayerItem {
    ptr: *mut c_void,
}

impl Drop for PlayerItem {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_player_item_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

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
        let keys_json = CString::new("[\"duration\",\"tracks\",\"metadata\"]").map_err(
            |error| AVPlayerError::InvalidArgument(format!("asset-key JSON contains NUL byte: {error}")),
        )?;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_player_item_create_with_asset(asset.ptr, keys_json.as_ptr(), &mut err)
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::PLAYER_CREATE_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn from_url_internal(url: &str, is_file_url: bool) -> Result<Self, AVPlayerError> {
        let url = CString::new(url)
            .map_err(|error| AVPlayerError::InvalidArgument(format!("URL contains NUL byte: {error}")))?;
        let keys_json = CString::new("[\"duration\",\"tracks\",\"metadata\"]").map_err(
            |error| AVPlayerError::InvalidArgument(format!("asset-key JSON contains NUL byte: {error}")),
        )?;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_player_item_create_with_url(
                url.as_ptr(),
                is_file_url,
                keys_json.as_ptr(),
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::PLAYER_CREATE_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn info(&self) -> Result<PlayerItemInfoPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_player_item_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn status(&self) -> Result<PlayerItemStatus, AVPlayerError> {
        Ok(PlayerItemStatus::from_raw(self.info()?.status))
    }

    pub fn error(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.error_message)
    }

    pub fn duration(&self) -> Result<Time, AVPlayerError> {
        Ok(self.info()?.duration)
    }

    pub fn presentation_size(&self) -> Result<Size, AVPlayerError> {
        Ok(self.info()?.presentation_size)
    }

    /// The current macOS SDK does not expose `externalMetadata`; this returns
    /// the underlying asset metadata instead.
    pub fn metadata(&self) -> Result<Vec<MetadataItem>, AVPlayerError> {
        Ok(self.info()?.metadata)
    }

    pub fn observe<F>(&self, callback: F) -> Result<PlayerItemObserver, AVPlayerError>
    where
        F: Fn(PlayerItemEvent) + Send + 'static,
    {
        let state = Box::new(PlayerItemObserverState {
            callback: Box::new(callback),
        });
        let userdata = Box::into_raw(state).cast::<c_void>();
        let mut err: *mut c_char = ptr::null_mut();
        let token = unsafe {
            ffi::av_player_item_add_observer(
                self.ptr,
                Some(player_item_event_trampoline),
                userdata,
                Some(player_item_observer_drop),
                &mut err,
            )
        };
        if token.is_null() {
            unsafe { player_item_observer_drop(userdata) };
            return Err(unsafe { from_swift(ffi::status::OBSERVER_FAILED, err) });
        }
        Ok(PlayerItemObserver { token })
    }
}

/// KVO + notification observer for `AVPlayerItem`.
pub struct PlayerItemObserver {
    token: *mut c_void,
}

impl Drop for PlayerItemObserver {
    fn drop(&mut self) {
        if !self.token.is_null() {
            unsafe { ffi::av_player_item_observer_release(self.token) };
            self.token = ptr::null_mut();
        }
    }
}

/// Safe wrapper around `AVPlayer`.
pub struct Player {
    ptr: *mut c_void,
}

impl Drop for Player {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_player_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

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
        let ptr = unsafe { ffi::av_player_create_with_asset(asset.ptr, &mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::PLAYER_CREATE_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    /// Create a player from an already-configured item.
    pub fn from_item(item: &PlayerItem) -> Result<Self, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::av_player_create_with_item(item.ptr, &mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::PLAYER_CREATE_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn from_url_internal(url: &str, is_file_url: bool) -> Result<Self, AVPlayerError> {
        let url = CString::new(url)
            .map_err(|error| AVPlayerError::InvalidArgument(format!("URL contains NUL byte: {error}")))?;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::av_player_create_with_url(url.as_ptr(), is_file_url, &mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::PLAYER_CREATE_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn info(&self) -> Result<PlayerInfoPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_player_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn status(&self) -> Result<PlayerStatus, AVPlayerError> {
        Ok(PlayerStatus::from_raw(self.info()?.status))
    }

    pub fn error(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.error_message)
    }

    pub fn rate(&self) -> Result<f32, AVPlayerError> {
        Ok(self.info()?.rate)
    }

    pub fn current_time(&self) -> Result<Time, AVPlayerError> {
        Ok(self.info()?.current_time)
    }

    pub fn duration(&self) -> Result<Time, AVPlayerError> {
        Ok(self.info()?.duration)
    }

    pub fn current_item(&self) -> Option<PlayerItem> {
        let ptr = unsafe { ffi::av_player_copy_current_item(self.ptr) };
        if ptr.is_null() {
            None
        } else {
            Some(PlayerItem { ptr })
        }
    }

    pub fn play(&self) {
        unsafe { ffi::av_player_play(self.ptr) };
    }

    pub fn pause(&self) {
        unsafe { ffi::av_player_pause(self.ptr) };
    }

    pub fn set_rate(&self, rate: f32) {
        unsafe { ffi::av_player_set_rate(self.ptr, rate) };
    }

    pub fn seek_to(&self, time: Time) -> Result<(), AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let (value, timescale, kind) = time.to_raw();
        let status = unsafe { ffi::av_player_seek(self.ptr, value, timescale, kind, &mut err) };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

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
        let state = Box::new(PeriodicTimeObserverState {
            callback: Box::new(callback),
        });
        let userdata = Box::into_raw(state).cast::<c_void>();
        let (value, timescale, kind) = interval.to_raw();
        let mut err: *mut c_char = ptr::null_mut();
        let token = unsafe {
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
                Some(periodic_time_observer_drop),
                &mut err,
            )
        };
        if token.is_null() {
            unsafe { periodic_time_observer_drop(userdata) };
            return Err(unsafe { from_swift(ffi::status::OBSERVER_FAILED, err) });
        }
        Ok(PeriodicTimeObserver { token })
    }

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
        let times_json = serde_json::to_string(times)
            .map_err(|error| AVPlayerError::InvalidArgument(format!("failed to encode boundary times: {error}")))?;
        let times_json = CString::new(times_json).map_err(|error| {
            AVPlayerError::InvalidArgument(format!("boundary times JSON contains NUL byte: {error}"))
        })?;
        let state = Box::new(BoundaryTimeObserverState {
            callback: Box::new(callback),
        });
        let userdata = Box::into_raw(state).cast::<c_void>();
        let mut err: *mut c_char = ptr::null_mut();
        let token = unsafe {
            ffi::av_player_add_boundary_time_observer(
                self.ptr,
                times_json.as_ptr(),
                queue_label
                    .as_ref()
                    .map_or(ptr::null(), |label| label.as_ptr()),
                Some(boundary_time_observer_trampoline),
                userdata,
                Some(boundary_time_observer_drop),
                &mut err,
            )
        };
        if token.is_null() {
            unsafe { boundary_time_observer_drop(userdata) };
            return Err(unsafe { from_swift(ffi::status::OBSERVER_FAILED, err) });
        }
        Ok(BoundaryTimeObserver { token })
    }
}

/// RAII token for `addPeriodicTimeObserver`.
pub struct PeriodicTimeObserver {
    token: *mut c_void,
}

impl Drop for PeriodicTimeObserver {
    fn drop(&mut self) {
        if !self.token.is_null() {
            unsafe { ffi::av_player_time_observer_release(self.token) };
            self.token = ptr::null_mut();
        }
    }
}

/// RAII token for `addBoundaryTimeObserver`.
pub struct BoundaryTimeObserver {
    token: *mut c_void,
}

impl Drop for BoundaryTimeObserver {
    fn drop(&mut self) {
        if !self.token.is_null() {
            unsafe { ffi::av_player_time_observer_release(self.token) };
            self.token = ptr::null_mut();
        }
    }
}

unsafe extern "C" fn player_item_event_trampoline(userdata: *mut c_void, payload_json: *const c_char) {
    if userdata.is_null() || payload_json.is_null() {
        return;
    }

    let callback = &*userdata.cast::<PlayerItemObserverState>();
    let Ok(payload) = CStr::from_ptr(payload_json).to_str() else {
        return;
    };
    let Ok(payload) = serde_json::from_str::<PlayerItemEventPayload>(payload) else {
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
        "did_play_to_end" => PlayerItemEvent::DidPlayToEnd,
        _ => return,
    };

    (callback.callback)(event);
}

unsafe extern "C" fn player_item_observer_drop(userdata: *mut c_void) {
    if !userdata.is_null() {
        drop(Box::from_raw(userdata.cast::<PlayerItemObserverState>()));
    }
}

unsafe extern "C" fn periodic_time_observer_trampoline(
    userdata: *mut c_void,
    value: i64,
    timescale: i32,
    kind: i32,
) {
    if userdata.is_null() {
        return;
    }
    let state = &mut *userdata.cast::<PeriodicTimeObserverState>();
    (state.callback)(Time::from_raw(value, timescale, kind));
}

unsafe extern "C" fn periodic_time_observer_drop(userdata: *mut c_void) {
    if !userdata.is_null() {
        drop(Box::from_raw(userdata.cast::<PeriodicTimeObserverState>()));
    }
}

unsafe extern "C" fn boundary_time_observer_trampoline(userdata: *mut c_void) {
    if userdata.is_null() {
        return;
    }
    let state = &mut *userdata.cast::<BoundaryTimeObserverState>();
    (state.callback)();
}

unsafe extern "C" fn boundary_time_observer_drop(userdata: *mut c_void) {
    if !userdata.is_null() {
        drop(Box::from_raw(userdata.cast::<BoundaryTimeObserverState>()));
    }
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
    let json = unsafe { CStr::from_ptr(json_ptr) }
        .to_string_lossy()
        .into_owned();
    unsafe { ffi::avp_string_free(json_ptr) };
    serde_json::from_str::<T>(&json)
        .map_err(|error| AVPlayerError::OperationFailed(format!("failed to decode bridge JSON: {error}")))
}
