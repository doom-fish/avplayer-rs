#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;

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
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MediaCharacteristic {
    Audible,
    Legible,
    Visual,
    ContainsOnlyForcedSubtitles,
    TranscribesSpokenDialogForAccessibility,
    DescribesMusicAndSoundForAccessibility,
    DescribesVideoForAccessibility,
    EasyToRead,
    LanguageTranslation,
    DubbedTranslation,
    VoiceOverTranslation,
    IsOriginalContent,
    Unknown(String),
}

impl MediaCharacteristic {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PlayerActionAtItemEnd {
    Advance,
    Pause,
    None,
}

impl PlayerActionAtItemEnd {
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::Advance,
            2 => Self::None,
            _ => Self::Pause,
        }
    }

    #[must_use]
    pub const fn as_raw(self) -> i32 {
        match self {
            Self::Advance => 0,
            Self::Pause => 1,
            Self::None => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PlayerTimeControlStatus {
    Paused,
    WaitingToPlayAtSpecifiedRate,
    Playing,
    Unknown(i32),
}

impl PlayerTimeControlStatus {
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

    pub fn preferred_languages(&self) -> Result<Vec<String>, AVPlayerError> {
        Ok(self.info()?.preferred_languages.unwrap_or_default())
    }

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

    pub fn time_control_status(&self) -> Result<PlayerTimeControlStatus, AVPlayerError> {
        Ok(PlayerTimeControlStatus::from_raw(
            self.info_for_media_selection()?
                .time_control_status
                .unwrap_or_default(),
        ))
    }

    pub fn reason_for_waiting_to_play(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info_for_media_selection()?.reason_for_waiting_to_play)
    }

    pub fn action_at_item_end(&self) -> Result<PlayerActionAtItemEnd, AVPlayerError> {
        Ok(PlayerActionAtItemEnd::from_raw(
            self.info_for_media_selection()?
                .action_at_item_end
                .unwrap_or(1),
        ))
    }

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

    pub fn volume(&self) -> Result<f32, AVPlayerError> {
        Ok(self.info_for_media_selection()?.volume.unwrap_or(1.0))
    }

    pub fn set_volume(&self, volume: f32) {
        unsafe { ffi::av_player_set_volume(self.ptr, volume) };
    }

    pub fn is_muted(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info_for_media_selection()?.muted.unwrap_or(false))
    }

    pub fn set_muted(&self, muted: bool) {
        unsafe { ffi::av_player_set_muted(self.ptr, muted) };
    }

    pub fn automatically_waits_to_minimize_stalling(&self) -> Result<bool, AVPlayerError> {
        Ok(self
            .info_for_media_selection()?
            .automatically_waits_to_minimize_stalling
            .unwrap_or(false))
    }

    pub fn set_automatically_waits_to_minimize_stalling(&self, enabled: bool) {
        unsafe { ffi::av_player_set_automatically_waits_to_minimize_stalling(self.ptr, enabled) };
    }

    pub fn applies_media_selection_criteria_automatically(&self) -> Result<bool, AVPlayerError> {
        Ok(self
            .info_for_media_selection()?
            .applies_media_selection_criteria_automatically
            .unwrap_or(false))
    }

    pub fn set_applies_media_selection_criteria_automatically(&self, enabled: bool) {
        unsafe {
            ffi::av_player_set_applies_media_selection_criteria_automatically(self.ptr, enabled);
        }
    }

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
