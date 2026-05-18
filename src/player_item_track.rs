#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use std::ffi::CString;

use serde::Deserialize;

use crate::asset::AssetTrack;
use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::player::PlayerItem;
use crate::util::parse_json_and_free;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerItemTrackInfoPayload {
    enabled: bool,
    current_video_frame_rate: f32,
    video_field_mode: Option<String>,
    has_asset_track: bool,
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerItemTrackVideoFieldMode`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PlayerItemTrackVideoFieldMode {
/// Mirrors the `AVPlayer` framework case `DeinterlaceFields`.
    DeinterlaceFields,
/// Mirrors the `AVPlayer` framework case `Unknown`.
    Unknown(String),
}

impl PlayerItemTrackVideoFieldMode {
    fn from_raw(raw: &str) -> Self {
        match raw {
            "AVPlayerItemTrackVideoFieldModeDeinterlaceFields" => Self::DeinterlaceFields,
            other => Self::Unknown(other.to_owned()),
        }
    }

    fn as_raw(&self) -> &str {
        match self {
            Self::DeinterlaceFields => "AVPlayerItemTrackVideoFieldModeDeinterlaceFields",
            Self::Unknown(raw) => raw,
        }
    }
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerItemTrack`.
#[derive(Debug)]
pub struct PlayerItemTrack {
    pub(crate) ptr: *mut c_void,
}

impl Drop for PlayerItemTrack {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_player_item_track_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

// SAFETY: AVPlayerItemTrack ObjC handles are safe to transfer across thread
// boundaries; method calls are internally dispatched safely.
unsafe impl Send for PlayerItemTrack {}

impl PlayerItemTrack {
    fn info(&self) -> Result<PlayerItemTrackInfoPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_player_item_track_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

/// Calls the `AVPlayer` framework counterpart for `is_enabled`.
    pub fn is_enabled(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.enabled)
    }

/// Calls the `AVPlayer` framework counterpart for `set_enabled`.
    pub fn set_enabled(&self, enabled: bool) {
        unsafe { ffi::av_player_item_track_set_enabled(self.ptr, enabled) };
    }

/// Calls the `AVPlayer` framework counterpart for `current_video_frame_rate`.
    pub fn current_video_frame_rate(&self) -> Result<f32, AVPlayerError> {
        Ok(self.info()?.current_video_frame_rate)
    }

/// Calls the `AVPlayer` framework counterpart for `video_field_mode`.
    pub fn video_field_mode(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.video_field_mode)
    }

/// Calls the `AVPlayer` framework counterpart for `typed_video_field_mode`.
    pub fn typed_video_field_mode(
        &self,
    ) -> Result<Option<PlayerItemTrackVideoFieldMode>, AVPlayerError> {
        Ok(self
            .info()?
            .video_field_mode
            .as_deref()
            .map(PlayerItemTrackVideoFieldMode::from_raw))
    }

/// Calls the `AVPlayer` framework counterpart for `set_typed_video_field_mode`.
    pub fn set_typed_video_field_mode(
        &self,
        video_field_mode: Option<&PlayerItemTrackVideoFieldMode>,
    ) -> Result<(), AVPlayerError> {
        self.set_video_field_mode(video_field_mode.map(PlayerItemTrackVideoFieldMode::as_raw))
    }

/// Calls the `AVPlayer` framework counterpart for `set_video_field_mode`.
    pub fn set_video_field_mode(
        &self,
        video_field_mode: Option<&str>,
    ) -> Result<(), AVPlayerError> {
        let mode = video_field_mode
            .map(|mode| {
                CString::new(mode).map_err(|error| {
                    AVPlayerError::InvalidArgument(format!(
                        "video field mode contains NUL byte: {error}"
                    ))
                })
            })
            .transpose()?;
        unsafe {
            ffi::av_player_item_track_set_video_field_mode(
                self.ptr,
                mode.as_ref().map_or(ptr::null(), |mode| mode.as_ptr()),
            );
        }
        Ok(())
    }

/// Calls the `AVPlayer` framework counterpart for `asset_track`.
    pub fn asset_track(&self) -> Result<Option<AssetTrack>, AVPlayerError> {
        if !self.info()?.has_asset_track {
            return Ok(None);
        }
        let ptr = unsafe { ffi::av_player_item_track_copy_asset_track(self.ptr) };
        if ptr.is_null() {
            return Ok(None);
        }
        Ok(Some(AssetTrack { ptr }))
    }
}

impl PlayerItem {
/// Calls the `AVPlayer` framework counterpart for `tracks`.
    pub fn tracks(&self) -> Result<Vec<PlayerItemTrack>, AVPlayerError> {
        let count = unsafe { ffi::av_player_item_track_count(self.ptr) };
        if count < 0 {
            return Err(AVPlayerError::OperationFailed(format!(
                "player-item track count unexpectedly negative: {count}"
            )));
        }

        let count = usize::try_from(count).map_err(|error| {
            AVPlayerError::OperationFailed(format!(
                "player-item track count exceeds addressable size: {error}"
            ))
        })?;

        let mut tracks = Vec::with_capacity(count);
        for index in 0..count {
            let ptr = unsafe {
                ffi::av_player_item_copy_track_at_index(
                    self.ptr,
                    i32::try_from(index).map_err(|error| {
                        AVPlayerError::OperationFailed(format!(
                            "player-item track index exceeds bridge range: {error}"
                        ))
                    })?,
                )
            };
            if ptr.is_null() {
                return Err(AVPlayerError::OperationFailed(format!(
                    "bridge returned null player-item track at index {index}"
                )));
            }
            tracks.push(PlayerItemTrack { ptr });
        }
        Ok(tracks)
    }
}
