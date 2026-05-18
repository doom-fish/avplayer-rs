#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use std::ffi::CStr;

use serde::Deserialize;

use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::player::Player;
use crate::util::{json_cstring, parse_json_and_free, to_cstring};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CriteriaPayload {
    preferred_languages: Option<Vec<String>>,
    preferred_media_characteristics: Option<Vec<String>>,
    principal_media_characteristics: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerInfoPayload {
    time_control_status: Option<i32>,
    reason_for_waiting_to_play: Option<String>,
    action_at_item_end: Option<i32>,
    volume: Option<f32>,
    muted: Option<bool>,
    automatically_waits_to_minimize_stalling: Option<bool>,
    applies_media_selection_criteria_automatically: Option<bool>,
    eligible_for_hdr_playback: Option<bool>,
    audiovisual_background_playback_policy: Option<i32>,
    network_resource_priority: Option<i32>,
}

/// Mirrors the `AVPlayer` framework counterpart for `MediaCharacteristic`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MediaCharacteristic {
/// Mirrors the `AVPlayer` framework case `Audible`.
    Audible,
/// Mirrors the `AVPlayer` framework case `Legible`.
    Legible,
/// Mirrors the `AVPlayer` framework case `Visual`.
    Visual,
/// Mirrors the `AVPlayer` framework case `ContainsOnlyForcedSubtitles`.
    ContainsOnlyForcedSubtitles,
/// Mirrors the `AVPlayer` framework case `TranscribesSpokenDialogForAccessibility`.
    TranscribesSpokenDialogForAccessibility,
/// Mirrors the `AVPlayer` framework case `DescribesMusicAndSoundForAccessibility`.
    DescribesMusicAndSoundForAccessibility,
/// Mirrors the `AVPlayer` framework case `DescribesVideoForAccessibility`.
    DescribesVideoForAccessibility,
/// Mirrors the `AVPlayer` framework case `EasyToRead`.
    EasyToRead,
/// Mirrors the `AVPlayer` framework case `LanguageTranslation`.
    LanguageTranslation,
/// Mirrors the `AVPlayer` framework case `DubbedTranslation`.
    DubbedTranslation,
/// Mirrors the `AVPlayer` framework case `VoiceOverTranslation`.
    VoiceOverTranslation,
/// Mirrors the `AVPlayer` framework case `IsOriginalContent`.
    IsOriginalContent,
/// Mirrors the `AVPlayer` framework case `Unknown`.
    Unknown(String),
}

impl MediaCharacteristic {
/// Calls the `AVPlayer` framework counterpart for `as_raw`.
    #[must_use]
    pub fn as_raw(&self) -> &str {
        match self {
            Self::Audible => "audible",
            Self::Legible => "legible",
            Self::Visual => "visual",
            Self::ContainsOnlyForcedSubtitles => "contains_only_forced_subtitles",
            Self::TranscribesSpokenDialogForAccessibility => {
                "transcribes_spoken_dialog_for_accessibility"
            }
            Self::DescribesMusicAndSoundForAccessibility => {
                "describes_music_and_sound_for_accessibility"
            }
            Self::DescribesVideoForAccessibility => "describes_video_for_accessibility",
            Self::EasyToRead => "easy_to_read",
            Self::LanguageTranslation => "language_translation",
            Self::DubbedTranslation => "dubbed_translation",
            Self::VoiceOverTranslation => "voice_over_translation",
            Self::IsOriginalContent => "is_original_content",
            Self::Unknown(raw) => raw,
        }
    }

/// Calls the `AVPlayer` framework counterpart for `from_raw`.
    #[must_use]
    pub fn from_raw(raw: &str) -> Self {
        match raw {
            "audible" => Self::Audible,
            "legible" => Self::Legible,
            "visual" => Self::Visual,
            "contains_only_forced_subtitles" => Self::ContainsOnlyForcedSubtitles,
            "transcribes_spoken_dialog_for_accessibility" => {
                Self::TranscribesSpokenDialogForAccessibility
            }
            "describes_music_and_sound_for_accessibility" => {
                Self::DescribesMusicAndSoundForAccessibility
            }
            "describes_video_for_accessibility" => Self::DescribesVideoForAccessibility,
            "easy_to_read" => Self::EasyToRead,
            "language_translation" => Self::LanguageTranslation,
            "dubbed_translation" => Self::DubbedTranslation,
            "voice_over_translation" => Self::VoiceOverTranslation,
            "is_original_content" => Self::IsOriginalContent,
            other => Self::Unknown(other.to_owned()),
        }
    }
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerActionAtItemEnd`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PlayerActionAtItemEnd {
/// Mirrors the `AVPlayer` framework case `Advance`.
    Advance,
/// Mirrors the `AVPlayer` framework case `Pause`.
    Pause,
/// Mirrors the `AVPlayer` framework case `None`.
    None,
}

impl PlayerActionAtItemEnd {
/// Mirrors the `AVPlayer` framework constant `fn`.
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::Advance,
            2 => Self::None,
            _ => Self::Pause,
        }
    }

/// Mirrors the `AVPlayer` framework constant `fn`.
    #[must_use]
    pub const fn as_raw(self) -> i32 {
        match self {
            Self::Advance => 0,
            Self::Pause => 1,
            Self::None => 2,
        }
    }
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerTimeControlStatus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PlayerTimeControlStatus {
/// Mirrors the `AVPlayer` framework case `Paused`.
    Paused,
/// Mirrors the `AVPlayer` framework case `WaitingToPlayAtSpecifiedRate`.
    WaitingToPlayAtSpecifiedRate,
/// Mirrors the `AVPlayer` framework case `Playing`.
    Playing,
/// Mirrors the `AVPlayer` framework case `Unknown`.
    Unknown(i32),
}

impl PlayerTimeControlStatus {
/// Mirrors the `AVPlayer` framework constant `fn`.
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::Paused,
            1 => Self::WaitingToPlayAtSpecifiedRate,
            2 => Self::Playing,
            other => Self::Unknown(other),
        }
    }
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerWaitingReason`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PlayerWaitingReason {
/// Mirrors the `AVPlayer` framework case `ToMinimizeStalls`.
    ToMinimizeStalls,
/// Mirrors the `AVPlayer` framework case `WhileEvaluatingBufferingRate`.
    WhileEvaluatingBufferingRate,
/// Mirrors the `AVPlayer` framework case `WithNoItemToPlay`.
    WithNoItemToPlay,
/// Mirrors the `AVPlayer` framework case `ForCoordinatedPlayback`.
    ForCoordinatedPlayback,
/// Mirrors the `AVPlayer` framework case `Unknown`.
    Unknown(String),
}

impl PlayerWaitingReason {
    fn from_raw(raw: &str) -> Self {
        match raw {
            "AVPlayerWaitingToMinimizeStallsReason" => Self::ToMinimizeStalls,
            "AVPlayerWaitingWhileEvaluatingBufferingRateReason" => {
                Self::WhileEvaluatingBufferingRate
            }
            "AVPlayerWaitingWithNoItemToPlayReason" => Self::WithNoItemToPlay,
            "AVPlayerWaitingForCoordinatedPlaybackReason" => Self::ForCoordinatedPlayback,
            other => Self::Unknown(other.to_owned()),
        }
    }
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerAudiovisualBackgroundPlaybackPolicy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PlayerAudiovisualBackgroundPlaybackPolicy {
/// Mirrors the `AVPlayer` framework case `Automatic`.
    Automatic,
/// Mirrors the `AVPlayer` framework case `Pauses`.
    Pauses,
/// Mirrors the `AVPlayer` framework case `ContinuesIfPossible`.
    ContinuesIfPossible,
/// Mirrors the `AVPlayer` framework case `Unknown`.
    Unknown(i32),
}

impl PlayerAudiovisualBackgroundPlaybackPolicy {
    const fn from_raw(raw: i32) -> Self {
        match raw {
            1 => Self::Automatic,
            2 => Self::Pauses,
            3 => Self::ContinuesIfPossible,
            other => Self::Unknown(other),
        }
    }

    const fn as_raw(self) -> i32 {
        match self {
            Self::Automatic => 1,
            Self::Pauses => 2,
            Self::ContinuesIfPossible => 3,
            Self::Unknown(raw) => raw,
        }
    }
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerNetworkResourcePriority`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PlayerNetworkResourcePriority {
/// Mirrors the `AVPlayer` framework case `Default`.
    Default,
/// Mirrors the `AVPlayer` framework case `Low`.
    Low,
/// Mirrors the `AVPlayer` framework case `High`.
    High,
/// Mirrors the `AVPlayer` framework case `Unknown`.
    Unknown(i32),
}

impl PlayerNetworkResourcePriority {
    const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::Default,
            1 => Self::Low,
            2 => Self::High,
            other => Self::Unknown(other),
        }
    }

    const fn as_raw(self) -> i32 {
        match self {
            Self::Default => 0,
            Self::Low => 1,
            Self::High => 2,
            Self::Unknown(raw) => raw,
        }
    }
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerRateDidChangeReason`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PlayerRateDidChangeReason {
/// Mirrors the `AVPlayer` framework case `SetRateCalled`.
    SetRateCalled,
/// Mirrors the `AVPlayer` framework case `SetRateFailed`.
    SetRateFailed,
/// Mirrors the `AVPlayer` framework case `AudioSessionInterrupted`.
    AudioSessionInterrupted,
/// Mirrors the `AVPlayer` framework case `AppBackgrounded`.
    AppBackgrounded,
/// Mirrors the `AVPlayer` framework case `Unknown`.
    Unknown(String),
}

impl PlayerRateDidChangeReason {
    fn from_raw(raw: &str) -> Self {
        match raw {
            "AVPlayerRateDidChangeReasonSetRateCalled" => Self::SetRateCalled,
            "AVPlayerRateDidChangeReasonSetRateFailed" => Self::SetRateFailed,
            "AVPlayerRateDidChangeReasonAudioSessionInterrupted" => Self::AudioSessionInterrupted,
            "AVPlayerRateDidChangeReasonAppBackgrounded" => Self::AppBackgrounded,
            other => Self::Unknown(other.to_owned()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerRateDidChangeEventPayload {
    rate: f32,
    reason: Option<String>,
    has_originating_participant: bool,
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerRateDidChangeEvent`.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerRateDidChangeEvent {
/// Mirrors the `AVPlayer` framework property for `rate`.
    pub rate: f32,
/// Mirrors the `AVPlayer` framework property for `reason`.
    pub reason: Option<PlayerRateDidChangeReason>,
/// Mirrors the `AVPlayer` framework property for `has_originating_participant`.
    pub has_originating_participant: bool,
}

struct PlayerRateObserverState {
    callback: Box<dyn Fn(PlayerRateDidChangeEvent) + Send + 'static>,
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerMediaSelectionCriteria`.
#[derive(Debug)]
pub struct PlayerMediaSelectionCriteria {
    pub(crate) ptr: *mut c_void,
}

impl Drop for PlayerMediaSelectionCriteria {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_player_media_selection_criteria_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl PlayerMediaSelectionCriteria {
/// Calls the `AVPlayer` framework counterpart for `new`.
    pub fn new(
        preferred_languages: &[impl AsRef<str>],
        preferred_media_characteristics: &[MediaCharacteristic],
    ) -> Result<Self, AVPlayerError> {
        Self::with_principal_media_characteristics(
            &[],
            preferred_languages,
            preferred_media_characteristics,
        )
    }

/// Calls the `AVPlayer` framework counterpart for `with_principal_media_characteristics`.
    pub fn with_principal_media_characteristics(
        principal_media_characteristics: &[MediaCharacteristic],
        preferred_languages: &[impl AsRef<str>],
        preferred_media_characteristics: &[MediaCharacteristic],
    ) -> Result<Self, AVPlayerError> {
        let preferred_languages = preferred_languages
            .iter()
            .map(|language| language.as_ref().to_owned())
            .collect::<Vec<_>>();
        let preferred_media_characteristics = preferred_media_characteristics
            .iter()
            .map(MediaCharacteristic::as_raw)
            .collect::<Vec<_>>();
        let principal_media_characteristics = principal_media_characteristics
            .iter()
            .map(MediaCharacteristic::as_raw)
            .collect::<Vec<_>>();

        let preferred_languages = json_cstring(&preferred_languages, "preferred languages")?;
        let preferred_media_characteristics = json_cstring(
            &preferred_media_characteristics,
            "preferred media characteristics",
        )?;
        let principal_media_characteristics = if principal_media_characteristics.is_empty() {
            None
        } else {
            Some(json_cstring(
                &principal_media_characteristics,
                "principal media characteristics",
            )?)
        };

        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_player_media_selection_criteria_create(
                preferred_languages.as_ptr(),
                preferred_media_characteristics.as_ptr(),
                principal_media_characteristics
                    .as_ref()
                    .map_or(ptr::null(), |value| value.as_ptr()),
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::PLAYER_CREATE_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn info(&self) -> Result<CriteriaPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::av_player_media_selection_criteria_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

/// Calls the `AVPlayer` framework counterpart for `preferred_languages`.
    pub fn preferred_languages(&self) -> Result<Vec<String>, AVPlayerError> {
        Ok(self.info()?.preferred_languages.unwrap_or_default())
    }

/// Calls the `AVPlayer` framework counterpart for `preferred_media_characteristics`.
    pub fn preferred_media_characteristics(
        &self,
    ) -> Result<Vec<MediaCharacteristic>, AVPlayerError> {
        Ok(self
            .info()?
            .preferred_media_characteristics
            .unwrap_or_default()
            .into_iter()
            .map(|raw| MediaCharacteristic::from_raw(&raw))
            .collect())
    }

/// Calls the `AVPlayer` framework counterpart for `principal_media_characteristics`.
    pub fn principal_media_characteristics(
        &self,
    ) -> Result<Vec<MediaCharacteristic>, AVPlayerError> {
        Ok(self
            .info()?
            .principal_media_characteristics
            .unwrap_or_default()
            .into_iter()
            .map(|raw| MediaCharacteristic::from_raw(&raw))
            .collect())
    }
}

impl Player {
    fn info_for_media_selection(&self) -> Result<PlayerInfoPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_player_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

/// Calls the `AVPlayer` framework counterpart for `time_control_status`.
    pub fn time_control_status(&self) -> Result<PlayerTimeControlStatus, AVPlayerError> {
        Ok(PlayerTimeControlStatus::from_raw(
            self.info_for_media_selection()?
                .time_control_status
                .unwrap_or_default(),
        ))
    }

/// Calls the `AVPlayer` framework counterpart for `reason_for_waiting_to_play`.
    pub fn reason_for_waiting_to_play(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info_for_media_selection()?.reason_for_waiting_to_play)
    }

/// Calls the `AVPlayer` framework counterpart for `waiting_reason`.
    pub fn waiting_reason(&self) -> Result<Option<PlayerWaitingReason>, AVPlayerError> {
        Ok(self
            .info_for_media_selection()?
            .reason_for_waiting_to_play
            .as_deref()
            .map(PlayerWaitingReason::from_raw))
    }

/// Calls the `AVPlayer` framework counterpart for `action_at_item_end`.
    pub fn action_at_item_end(&self) -> Result<PlayerActionAtItemEnd, AVPlayerError> {
        Ok(PlayerActionAtItemEnd::from_raw(
            self.info_for_media_selection()?
                .action_at_item_end
                .unwrap_or(1),
        ))
    }

/// Calls the `AVPlayer` framework counterpart for `set_action_at_item_end`.
    pub fn set_action_at_item_end(
        &self,
        action: PlayerActionAtItemEnd,
    ) -> Result<(), AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status =
            unsafe { ffi::av_player_set_action_at_item_end(self.ptr, action.as_raw(), &mut err) };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

/// Calls the `AVPlayer` framework counterpart for `volume`.
    pub fn volume(&self) -> Result<f32, AVPlayerError> {
        Ok(self.info_for_media_selection()?.volume.unwrap_or(1.0))
    }

/// Calls the `AVPlayer` framework counterpart for `set_volume`.
    pub fn set_volume(&self, volume: f32) {
        unsafe { ffi::av_player_set_volume(self.ptr, volume) };
    }

/// Calls the `AVPlayer` framework counterpart for `is_muted`.
    pub fn is_muted(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info_for_media_selection()?.muted.unwrap_or(false))
    }

/// Calls the `AVPlayer` framework counterpart for `set_muted`.
    pub fn set_muted(&self, muted: bool) {
        unsafe { ffi::av_player_set_muted(self.ptr, muted) };
    }

/// Calls the `AVPlayer` framework counterpart for `automatically_waits_to_minimize_stalling`.
    pub fn automatically_waits_to_minimize_stalling(&self) -> Result<bool, AVPlayerError> {
        Ok(self
            .info_for_media_selection()?
            .automatically_waits_to_minimize_stalling
            .unwrap_or(false))
    }

/// Calls the `AVPlayer` framework counterpart for `set_automatically_waits_to_minimize_stalling`.
    pub fn set_automatically_waits_to_minimize_stalling(&self, enabled: bool) {
        unsafe { ffi::av_player_set_automatically_waits_to_minimize_stalling(self.ptr, enabled) };
    }

/// Calls the `AVPlayer` framework counterpart for `applies_media_selection_criteria_automatically`.
    pub fn applies_media_selection_criteria_automatically(&self) -> Result<bool, AVPlayerError> {
        Ok(self
            .info_for_media_selection()?
            .applies_media_selection_criteria_automatically
            .unwrap_or(false))
    }

/// Calls the `AVPlayer` framework counterpart for `set_applies_media_selection_criteria_automatically`.
    pub fn set_applies_media_selection_criteria_automatically(&self, enabled: bool) {
        unsafe {
            ffi::av_player_set_applies_media_selection_criteria_automatically(self.ptr, enabled);
        }
    }

/// Calls the `AVPlayer` framework counterpart for `eligible_for_hdr_playback`.
    pub fn eligible_for_hdr_playback(&self) -> Result<bool, AVPlayerError> {
        self.info_for_media_selection()?
            .eligible_for_hdr_playback
            .ok_or_else(|| availability_error("AVPlayer.eligibleForHDRPlayback", "10.15"))
    }

/// Calls the `AVPlayer` framework counterpart for `audiovisual_background_playback_policy`.
    pub fn audiovisual_background_playback_policy(
        &self,
    ) -> Result<PlayerAudiovisualBackgroundPlaybackPolicy, AVPlayerError> {
        Ok(PlayerAudiovisualBackgroundPlaybackPolicy::from_raw(
            self.info_for_media_selection()?
                .audiovisual_background_playback_policy
                .ok_or_else(|| {
                    availability_error("AVPlayer.audiovisualBackgroundPlaybackPolicy", "12.0")
                })?,
        ))
    }

/// Calls the `AVPlayer` framework counterpart for `set_audiovisual_background_playback_policy`.
    pub fn set_audiovisual_background_playback_policy(
        &self,
        policy: PlayerAudiovisualBackgroundPlaybackPolicy,
    ) -> Result<(), AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::av_player_set_audiovisual_background_playback_policy(
                self.ptr,
                policy.as_raw(),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

/// Calls the `AVPlayer` framework counterpart for `network_resource_priority`.
    pub fn network_resource_priority(
        &self,
    ) -> Result<PlayerNetworkResourcePriority, AVPlayerError> {
        Ok(PlayerNetworkResourcePriority::from_raw(
            self.info_for_media_selection()?
                .network_resource_priority
                .ok_or_else(|| availability_error("AVPlayer.networkResourcePriority", "26.0"))?,
        ))
    }

/// Calls the `AVPlayer` framework counterpart for `set_network_resource_priority`.
    pub fn set_network_resource_priority(
        &self,
        priority: PlayerNetworkResourcePriority,
    ) -> Result<(), AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::av_player_set_network_resource_priority(self.ptr, priority.as_raw(), &mut err)
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

/// Calls the `AVPlayer` framework counterpart for `observe_rate_changes`.
    pub fn observe_rate_changes<F>(
        &self,
        queue_label: Option<&str>,
        callback: F,
    ) -> Result<PlayerRateDidChangeObserver, AVPlayerError>
    where
        F: Fn(PlayerRateDidChangeEvent) + Send + 'static,
    {
        let queue_label = queue_label
            .map(|label| to_cstring(label, "player rate observer queue label"))
            .transpose()?;
        let state = Box::new(PlayerRateObserverState {
            callback: Box::new(callback),
        });
        let userdata = Box::into_raw(state).cast::<c_void>();
        let mut err: *mut c_char = ptr::null_mut();
        let token = unsafe {
            ffi::av_player_add_rate_observer(
                self.ptr,
                queue_label
                    .as_ref()
                    .map_or(ptr::null(), |label| label.as_ptr()),
                Some(player_rate_event_trampoline),
                userdata,
                Some(player_rate_observer_drop),
                &mut err,
            )
        };
        if token.is_null() {
            unsafe { player_rate_observer_drop(userdata) };
            return Err(unsafe { from_swift(ffi::status::OBSERVER_FAILED, err) });
        }
        Ok(PlayerRateDidChangeObserver { token })
    }

/// Calls the `AVPlayer` framework counterpart for `set_media_selection_criteria`.
    pub fn set_media_selection_criteria(
        &self,
        media_characteristic: &MediaCharacteristic,
        criteria: Option<&PlayerMediaSelectionCriteria>,
    ) -> Result<(), AVPlayerError> {
        let media_characteristic =
            to_cstring(media_characteristic.as_raw(), "media characteristic")?;
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::av_player_set_media_selection_criteria(
                self.ptr,
                media_characteristic.as_ptr(),
                criteria.map_or(ptr::null_mut(), |criteria| criteria.ptr),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

/// Calls the `AVPlayer` framework counterpart for `media_selection_criteria`.
    pub fn media_selection_criteria(
        &self,
        media_characteristic: &MediaCharacteristic,
    ) -> Result<Option<PlayerMediaSelectionCriteria>, AVPlayerError> {
        let media_characteristic =
            to_cstring(media_characteristic.as_raw(), "media characteristic")?;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_player_copy_media_selection_criteria(
                self.ptr,
                media_characteristic.as_ptr(),
                &mut err,
            )
        };
        if ptr.is_null() {
            if err.is_null() {
                return Ok(None);
            }
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Some(PlayerMediaSelectionCriteria { ptr }))
    }
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerRateDidChangeObserver`.
#[derive(Debug)]
pub struct PlayerRateDidChangeObserver {
    token: *mut c_void,
}

impl Drop for PlayerRateDidChangeObserver {
    fn drop(&mut self) {
        if !self.token.is_null() {
            unsafe { ffi::av_player_rate_observer_release(self.token) };
            self.token = ptr::null_mut();
        }
    }
}

// SAFETY: These media-selection handles are safe to transfer across thread
// boundaries; method calls are internally dispatched safely.
unsafe impl Send for PlayerMediaSelectionCriteria {}
unsafe impl Send for PlayerRateDidChangeObserver {}

/// Calls the `AVPlayer` framework counterpart for `player_eligible_for_hdr_playback_did_change_notification`.
pub fn player_eligible_for_hdr_playback_did_change_notification() -> Result<String, AVPlayerError> {
    let mut err: *mut c_char = ptr::null_mut();
    let string_ptr =
        unsafe { ffi::av_player_eligible_for_hdr_playback_did_change_notification_name(&mut err) };
    if string_ptr.is_null() {
        return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
    }
    let value = unsafe { CStr::from_ptr(string_ptr) }
        .to_string_lossy()
        .into_owned();
    unsafe { ffi::avp_string_free(string_ptr) };
    Ok(value)
}

unsafe extern "C" fn player_rate_event_trampoline(
    userdata: *mut c_void,
    payload_json: *const c_char,
) {
    if userdata.is_null() || payload_json.is_null() {
        return;
    }

    let callback = &*userdata.cast::<PlayerRateObserverState>();
    let Ok(payload) = CStr::from_ptr(payload_json).to_str() else {
        return;
    };
    let Ok(payload) = serde_json::from_str::<PlayerRateDidChangeEventPayload>(payload) else {
        return;
    };

    crate::util::catch_cb_panic("player_rate_event_trampoline", || {
        (callback.callback)(PlayerRateDidChangeEvent {
            rate: payload.rate,
            reason: payload
                .reason
                .as_deref()
                .map(PlayerRateDidChangeReason::from_raw),
            has_originating_participant: payload.has_originating_participant,
        });
    });
}

unsafe extern "C" fn player_rate_observer_drop(userdata: *mut c_void) {
    if !userdata.is_null() {
        drop(Box::from_raw(userdata.cast::<PlayerRateObserverState>()));
    }
}

fn availability_error(symbol: &str, macos_version: &str) -> AVPlayerError {
    AVPlayerError::OperationFailed(format!("{symbol} requires macOS {macos_version}+"))
}
