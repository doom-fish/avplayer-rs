#![allow(
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::struct_excessive_bools
)]

use core::ffi::c_char;
use core::ops::{BitOr, BitOrAssign};
use core::ptr;
use std::ffi::CString;

use serde::Deserialize;

use crate::asset::Size;
use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::player::PlayerItem;
use crate::time::TimeRange;
use crate::util::parse_json_and_free;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExtendedPlayerItemInfoPayload {
    automatically_loaded_asset_keys: Vec<String>,
    seekable_time_ranges: Vec<TimeRange>,
    loaded_time_ranges: Vec<TimeRange>,
    can_use_network_resources_for_live_streaming_while_paused: bool,
    preferred_forward_buffer_duration: f64,
    preferred_peak_bit_rate: f64,
    preferred_peak_bit_rate_for_expensive_networks: f64,
    preferred_maximum_resolution: Size,
    preferred_maximum_resolution_for_expensive_networks: Size,
    audio_time_pitch_algorithm: String,
    output_count: usize,
    track_count: usize,
    variant_preferences: Option<u64>,
    authorization_required_for_playback: bool,
    application_authorized_for_playback: bool,
    content_authorized_for_playback: bool,
    content_authorization_request_status: i32,
    custom_video_compositor: Option<PlayerItemVideoCompositorPayload>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AudioTimePitchAlgorithm {
    Spectral,
    TimeDomain,
    Varispeed,
    LowQualityZeroLatency,
    Unknown(String),
}

impl AudioTimePitchAlgorithm {
    #[must_use]
    pub fn from_raw(raw: &str) -> Self {
        match raw {
            "spectral" => Self::Spectral,
            "time_domain" => Self::TimeDomain,
            "varispeed" => Self::Varispeed,
            "low_quality_zero_latency" => Self::LowQualityZeroLatency,
            other => Self::Unknown(other.to_owned()),
        }
    }

    #[must_use]
    pub fn as_raw(&self) -> &str {
        match self {
            Self::Spectral => "spectral",
            Self::TimeDomain => "time_domain",
            Self::Varispeed => "varispeed",
            Self::LowQualityZeroLatency => "low_quality_zero_latency",
            Self::Unknown(raw) => raw,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct VariantPreferences(u64);

impl VariantPreferences {
    pub const NONE: Self = Self(0);
    pub const SCALABILITY_TO_LOSSLESS_AUDIO: Self = Self(1 << 0);

    #[must_use]
    pub const fn bits(self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    const fn from_bits(bits: u64) -> Self {
        Self(bits)
    }
}

impl BitOr for VariantPreferences {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for VariantPreferences {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ContentAuthorizationStatus {
    Unknown,
    Completed,
    Cancelled,
    TimedOut,
    Busy,
    NotAvailable,
    NotPossible,
    Other(i32),
}

impl ContentAuthorizationStatus {
    const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::Unknown,
            1 => Self::Completed,
            2 => Self::Cancelled,
            3 => Self::TimedOut,
            4 => Self::Busy,
            5 => Self::NotAvailable,
            6 => Self::NotPossible,
            other => Self::Other(other),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerItemVideoCompositorPayload {
    class_name: String,
    supports_wide_color_source_frames: Option<bool>,
    supports_hdr_source_frames: Option<bool>,
    supports_source_tagged_buffers: Option<bool>,
    can_conform_color_of_source_frames: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerItemVideoCompositorInfo {
    pub class_name: String,
    pub supports_wide_color_source_frames: Option<bool>,
    pub supports_hdr_source_frames: Option<bool>,
    pub supports_source_tagged_buffers: Option<bool>,
    pub can_conform_color_of_source_frames: Option<bool>,
}

impl From<PlayerItemVideoCompositorPayload> for PlayerItemVideoCompositorInfo {
    fn from(payload: PlayerItemVideoCompositorPayload) -> Self {
        Self {
            class_name: payload.class_name,
            supports_wide_color_source_frames: payload.supports_wide_color_source_frames,
            supports_hdr_source_frames: payload.supports_hdr_source_frames,
            supports_source_tagged_buffers: payload.supports_source_tagged_buffers,
            can_conform_color_of_source_frames: payload.can_conform_color_of_source_frames,
        }
    }
}

impl PlayerItem {
    fn extended_info(&self) -> Result<ExtendedPlayerItemInfoPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_player_item_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn automatically_loaded_asset_keys(&self) -> Result<Vec<String>, AVPlayerError> {
        Ok(self.extended_info()?.automatically_loaded_asset_keys)
    }

    pub fn seekable_time_ranges(&self) -> Result<Vec<TimeRange>, AVPlayerError> {
        Ok(self.extended_info()?.seekable_time_ranges)
    }

    pub fn loaded_time_ranges(&self) -> Result<Vec<TimeRange>, AVPlayerError> {
        Ok(self.extended_info()?.loaded_time_ranges)
    }

    pub fn can_use_network_resources_for_live_streaming_while_paused(
        &self,
    ) -> Result<bool, AVPlayerError> {
        Ok(self
            .extended_info()?
            .can_use_network_resources_for_live_streaming_while_paused)
    }

    pub fn set_can_use_network_resources_for_live_streaming_while_paused(&self, enabled: bool) {
        unsafe {
            ffi::av_player_item_set_can_use_network_resources_for_live_streaming_while_paused(
                self.ptr, enabled,
            );
        }
    }

    pub fn preferred_forward_buffer_duration(&self) -> Result<f64, AVPlayerError> {
        Ok(self.extended_info()?.preferred_forward_buffer_duration)
    }

    pub fn set_preferred_forward_buffer_duration(&self, duration: f64) {
        unsafe { ffi::av_player_item_set_preferred_forward_buffer_duration(self.ptr, duration) };
    }

    pub fn preferred_peak_bit_rate(&self) -> Result<f64, AVPlayerError> {
        Ok(self.extended_info()?.preferred_peak_bit_rate)
    }

    pub fn set_preferred_peak_bit_rate(&self, value: f64) {
        unsafe { ffi::av_player_item_set_preferred_peak_bit_rate(self.ptr, value) };
    }

    pub fn preferred_peak_bit_rate_for_expensive_networks(&self) -> Result<f64, AVPlayerError> {
        Ok(self
            .extended_info()?
            .preferred_peak_bit_rate_for_expensive_networks)
    }

    pub fn set_preferred_peak_bit_rate_for_expensive_networks(&self, value: f64) {
        unsafe {
            ffi::av_player_item_set_preferred_peak_bit_rate_for_expensive_networks(self.ptr, value);
        }
    }

    pub fn preferred_maximum_resolution(&self) -> Result<Size, AVPlayerError> {
        Ok(self.extended_info()?.preferred_maximum_resolution)
    }

    pub fn set_preferred_maximum_resolution(&self, value: Size) {
        unsafe {
            ffi::av_player_item_set_preferred_maximum_resolution(
                self.ptr,
                value.width,
                value.height,
            );
        }
    }

    pub fn preferred_maximum_resolution_for_expensive_networks(
        &self,
    ) -> Result<Size, AVPlayerError> {
        Ok(self
            .extended_info()?
            .preferred_maximum_resolution_for_expensive_networks)
    }

    pub fn set_preferred_maximum_resolution_for_expensive_networks(&self, value: Size) {
        unsafe {
            ffi::av_player_item_set_preferred_maximum_resolution_for_expensive_networks(
                self.ptr,
                value.width,
                value.height,
            );
        }
    }

    pub fn audio_time_pitch_algorithm(&self) -> Result<AudioTimePitchAlgorithm, AVPlayerError> {
        Ok(AudioTimePitchAlgorithm::from_raw(
            &self.extended_info()?.audio_time_pitch_algorithm,
        ))
    }

    pub fn set_audio_time_pitch_algorithm(
        &self,
        algorithm: &AudioTimePitchAlgorithm,
    ) -> Result<(), AVPlayerError> {
        let algorithm = CString::new(algorithm.as_raw()).map_err(|error| {
            AVPlayerError::InvalidArgument(format!(
                "audio time pitch algorithm contains NUL byte: {error}"
            ))
        })?;
        unsafe { ffi::av_player_item_set_audio_time_pitch_algorithm(self.ptr, algorithm.as_ptr()) };
        Ok(())
    }

    pub fn output_count(&self) -> Result<usize, AVPlayerError> {
        Ok(self.extended_info()?.output_count)
    }

    pub fn track_count(&self) -> Result<usize, AVPlayerError> {
        Ok(self.extended_info()?.track_count)
    }

    pub fn variant_preferences(&self) -> Result<VariantPreferences, AVPlayerError> {
        Ok(VariantPreferences::from_bits(
            self.extended_info()?
                .variant_preferences
                .ok_or_else(|| availability_error("AVPlayerItem.variantPreferences", "11.3"))?,
        ))
    }

    pub fn set_variant_preferences(&self, preferences: VariantPreferences) -> Result<(), AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::av_player_item_set_variant_preferences(self.ptr, preferences.bits(), &mut err)
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn authorization_required_for_playback(&self) -> Result<bool, AVPlayerError> {
        Ok(self.extended_info()?.authorization_required_for_playback)
    }

    pub fn application_authorized_for_playback(&self) -> Result<bool, AVPlayerError> {
        Ok(self.extended_info()?.application_authorized_for_playback)
    }

    pub fn content_authorized_for_playback(&self) -> Result<bool, AVPlayerError> {
        Ok(self.extended_info()?.content_authorized_for_playback)
    }

    pub fn content_authorization_request_status(
        &self,
    ) -> Result<ContentAuthorizationStatus, AVPlayerError> {
        Ok(ContentAuthorizationStatus::from_raw(
            self.extended_info()?.content_authorization_request_status,
        ))
    }

    pub fn custom_video_compositor(
        &self,
    ) -> Result<Option<PlayerItemVideoCompositorInfo>, AVPlayerError> {
        Ok(self
            .extended_info()?
            .custom_video_compositor
            .map(PlayerItemVideoCompositorInfo::from))
    }
}

fn availability_error(symbol: &str, macos_version: &str) -> AVPlayerError {
    AVPlayerError::OperationFailed(format!("{symbol} requires macOS {macos_version}+"))
}
