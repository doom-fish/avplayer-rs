#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use std::ffi::CStr;

use serde::Deserialize;

use crate::asset::{Asset, MediaType, UrlAsset};
use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::metadata::MetadataItem;
use crate::player_media_selection_criteria::MediaCharacteristic;
use crate::util::{parse_json_and_free, to_cstring};

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MediaSelectionOptionPayload {
    media_type: String,
    media_sub_types: Vec<u32>,
    playable: bool,
    extended_language_tag: Option<String>,
    locale_identifier: Option<String>,
    display_name: String,
    common_metadata: Vec<MetadataItem>,
    available_metadata_formats: Vec<String>,
}

/// Safe wrapper around `AVMediaSelection`.
#[derive(Debug)]
pub struct MediaSelection {
    pub(crate) ptr: *mut c_void,
}

impl Drop for MediaSelection {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl MediaSelection {
    pub fn selected_media_option_in_group(
        &self,
        group: &MediaSelectionGroup,
    ) -> Option<MediaSelectionOption> {
        let ptr = unsafe { ffi::av_media_selection_copy_selected_option(self.ptr, group.ptr) };
        if ptr.is_null() {
            None
        } else {
            Some(MediaSelectionOption { ptr })
        }
    }

    pub fn media_selection_criteria_can_be_applied_automatically_to_group(
        &self,
        group: &MediaSelectionGroup,
    ) -> bool {
        unsafe { ffi::av_media_selection_can_apply_automatically(self.ptr, group.ptr) }
    }

    pub fn mutable_copy(&self) -> Result<MutableMediaSelection, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::av_media_selection_mutable_copy(self.ptr, &mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(MutableMediaSelection { ptr })
    }
}

/// Safe wrapper around `AVMutableMediaSelection`.
#[derive(Debug)]
pub struct MutableMediaSelection {
    ptr: *mut c_void,
}

impl Drop for MutableMediaSelection {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl MutableMediaSelection {
    pub fn selected_media_option_in_group(
        &self,
        group: &MediaSelectionGroup,
    ) -> Option<MediaSelectionOption> {
        let ptr = unsafe { ffi::av_media_selection_copy_selected_option(self.ptr, group.ptr) };
        if ptr.is_null() {
            None
        } else {
            Some(MediaSelectionOption { ptr })
        }
    }

    pub fn media_selection_criteria_can_be_applied_automatically_to_group(
        &self,
        group: &MediaSelectionGroup,
    ) -> bool {
        unsafe { ffi::av_media_selection_can_apply_automatically(self.ptr, group.ptr) }
    }

    pub fn select_media_option(
        &self,
        option: Option<&MediaSelectionOption>,
        group: &MediaSelectionGroup,
    ) {
        unsafe {
            ffi::av_mutable_media_selection_select_option(
                self.ptr,
                option.map_or(ptr::null_mut(), |value| value.ptr),
                group.ptr,
            );
        };
    }
}

/// Safe wrapper around `AVMediaSelectionGroup`.
#[derive(Debug)]
pub struct MediaSelectionGroup {
    pub(crate) ptr: *mut c_void,
}

impl Drop for MediaSelectionGroup {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl MediaSelectionGroup {
    pub fn options(&self) -> Result<Vec<MediaSelectionOption>, AVPlayerError> {
        let count = unsafe { ffi::av_media_selection_group_option_count(self.ptr) };
        let count = usize::try_from(count).map_err(|error| {
            AVPlayerError::OperationFailed(format!("invalid media-selection option count: {error}"))
        })?;
        let mut options = Vec::with_capacity(count);
        for index in 0..count {
            let ptr = unsafe {
                ffi::av_media_selection_group_copy_option_at_index(
                    self.ptr,
                    i32::try_from(index).unwrap_or(i32::MAX),
                )
            };
            if ptr.is_null() {
                return Err(AVPlayerError::OperationFailed(format!(
                    "bridge returned null media-selection option at index {index}"
                )));
            }
            options.push(MediaSelectionOption { ptr });
        }
        Ok(options)
    }

    pub fn default_option(&self) -> Option<MediaSelectionOption> {
        let ptr = unsafe { ffi::av_media_selection_group_copy_default_option(self.ptr) };
        if ptr.is_null() {
            None
        } else {
            Some(MediaSelectionOption { ptr })
        }
    }

    pub fn allows_empty_selection(&self) -> bool {
        unsafe { ffi::av_media_selection_group_allows_empty_selection(self.ptr) }
    }

    pub fn custom_media_selection_scheme(
        &self,
    ) -> Result<Option<CustomMediaSelectionScheme>, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_media_selection_group_copy_custom_media_selection_scheme(self.ptr, &mut err)
        };
        if ptr.is_null() {
            if err.is_null() {
                return Ok(None);
            }
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Some(CustomMediaSelectionScheme { ptr }))
    }
}

/// Safe wrapper around `AVMediaSelectionOption`.
#[derive(Debug)]
pub struct MediaSelectionOption {
    pub(crate) ptr: *mut c_void,
}

impl Drop for MediaSelectionOption {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl MediaSelectionOption {
    fn info(&self) -> Result<MediaSelectionOptionPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_media_selection_option_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn media_type(&self) -> Result<MediaType, AVPlayerError> {
        Ok(MediaType::from_raw(&self.info()?.media_type))
    }

    pub fn media_sub_types(&self) -> Result<Vec<u32>, AVPlayerError> {
        Ok(self.info()?.media_sub_types)
    }

    pub fn is_playable(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.playable)
    }

    pub fn extended_language_tag(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.extended_language_tag)
    }

    pub fn locale_identifier(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.locale_identifier)
    }

    pub fn display_name(&self) -> Result<String, AVPlayerError> {
        Ok(self.info()?.display_name)
    }

    pub fn common_metadata(&self) -> Result<Vec<MetadataItem>, AVPlayerError> {
        Ok(self.info()?.common_metadata)
    }

    pub fn available_metadata_formats(&self) -> Result<Vec<String>, AVPlayerError> {
        Ok(self.info()?.available_metadata_formats)
    }

    pub fn has_media_characteristic(
        &self,
        characteristic: &MediaCharacteristic,
    ) -> Result<bool, AVPlayerError> {
        let characteristic = to_cstring(characteristic.as_raw(), "media characteristic")?;
        Ok(unsafe {
            ffi::av_media_selection_option_has_media_characteristic(
                self.ptr,
                characteristic.as_ptr(),
            )
        })
    }

    pub fn associated_media_selection_option_in_group(
        &self,
        group: &MediaSelectionGroup,
    ) -> Option<Self> {
        let ptr =
            unsafe { ffi::av_media_selection_option_copy_associated_option(self.ptr, group.ptr) };
        if ptr.is_null() {
            None
        } else {
            Some(Self { ptr })
        }
    }

    pub fn display_name_with_locale_identifier(
        &self,
        locale_identifier: &str,
    ) -> Result<String, AVPlayerError> {
        let locale_identifier = to_cstring(locale_identifier, "media-selection locale identifier")?;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_media_selection_option_display_name_for_locale_identifier(
                self.ptr,
                locale_identifier.as_ptr(),
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        let value = unsafe { CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned();
        unsafe { ffi::avp_string_free(ptr) };
        Ok(value)
    }
}

/// Safe wrapper around `AVCustomMediaSelectionScheme`.
#[derive(Debug)]
pub struct CustomMediaSelectionScheme {
    ptr: *mut c_void,
}

impl Drop for CustomMediaSelectionScheme {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl CustomMediaSelectionScheme {
    pub fn should_offer_language_selection(&self) -> Result<bool, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let value = unsafe {
            ffi::av_custom_media_selection_scheme_should_offer_language_selection(
                self.ptr, &mut err,
            )
        };
        if !err.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(value)
    }

    pub fn available_languages(&self) -> Result<Vec<String>, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::av_custom_media_selection_scheme_available_languages_json(self.ptr, &mut err)
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn selectors(&self) -> Result<Vec<MediaPresentationSelector>, AVPlayerError> {
        let count = count_with_error(
            ffi::av_custom_media_selection_scheme_selector_count,
            self.ptr,
        )?;
        let mut selectors = Vec::with_capacity(count);
        for index in 0..count {
            let mut err: *mut c_char = ptr::null_mut();
            let ptr = unsafe {
                ffi::av_custom_media_selection_scheme_copy_selector_at_index(
                    self.ptr,
                    i32::try_from(index).unwrap_or(i32::MAX),
                    &mut err,
                )
            };
            if ptr.is_null() {
                return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
            }
            selectors.push(MediaPresentationSelector { ptr });
        }
        Ok(selectors)
    }
}

/// Safe wrapper around `AVMediaPresentationSelector`.
#[derive(Debug)]
pub struct MediaPresentationSelector {
    ptr: *mut c_void,
}

impl Drop for MediaPresentationSelector {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl MediaPresentationSelector {
    pub fn identifier(&self) -> Result<String, AVPlayerError> {
        string_with_error(ffi::av_media_presentation_selector_identifier, self.ptr)
    }

    pub fn display_name_for_locale_identifier(
        &self,
        locale_identifier: &str,
    ) -> Result<String, AVPlayerError> {
        let locale_identifier =
            to_cstring(locale_identifier, "presentation selector locale identifier")?;
        string_with_input(
            ffi::av_media_presentation_selector_display_name_for_locale_identifier,
            self.ptr,
            locale_identifier.as_ptr(),
        )
    }

    pub fn settings(&self) -> Result<Vec<MediaPresentationSetting>, AVPlayerError> {
        let count = count_with_error(ffi::av_media_presentation_selector_setting_count, self.ptr)?;
        let mut settings = Vec::with_capacity(count);
        for index in 0..count {
            let mut err: *mut c_char = ptr::null_mut();
            let ptr = unsafe {
                ffi::av_media_presentation_selector_copy_setting_at_index(
                    self.ptr,
                    i32::try_from(index).unwrap_or(i32::MAX),
                    &mut err,
                )
            };
            if ptr.is_null() {
                return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
            }
            settings.push(MediaPresentationSetting { ptr });
        }
        Ok(settings)
    }
}

/// Safe wrapper around `AVMediaPresentationSetting`.
#[derive(Debug)]
pub struct MediaPresentationSetting {
    ptr: *mut c_void,
}

impl Drop for MediaPresentationSetting {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl MediaPresentationSetting {
    pub fn media_characteristic(&self) -> Result<MediaCharacteristic, AVPlayerError> {
        let raw = string_with_error(
            ffi::av_media_presentation_setting_media_characteristic,
            self.ptr,
        )?;
        Ok(MediaCharacteristic::from_raw(&raw))
    }

    pub fn display_name_for_locale_identifier(
        &self,
        locale_identifier: &str,
    ) -> Result<String, AVPlayerError> {
        let locale_identifier =
            to_cstring(locale_identifier, "presentation setting locale identifier")?;
        string_with_input(
            ffi::av_media_presentation_setting_display_name_for_locale_identifier,
            self.ptr,
            locale_identifier.as_ptr(),
        )
    }
}

impl Asset {
    pub fn available_media_characteristics_with_media_selection_options(
        &self,
    ) -> Result<Vec<MediaCharacteristic>, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::av_asset_available_media_selection_characteristics_json(self.ptr, &mut err)
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(parse_json_and_free::<Vec<String>>(json_ptr)?
            .into_iter()
            .map(|raw| MediaCharacteristic::from_raw(&raw))
            .collect())
    }

    pub fn media_selection_group_for_media_characteristic(
        &self,
        media_characteristic: &MediaCharacteristic,
    ) -> Result<Option<MediaSelectionGroup>, AVPlayerError> {
        let media_characteristic = to_cstring(
            media_characteristic.as_raw(),
            "media-selection characteristic",
        )?;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_asset_copy_media_selection_group_for_characteristic(
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
        Ok(Some(MediaSelectionGroup { ptr }))
    }

    pub fn preferred_media_selection(&self) -> Result<MediaSelection, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::av_asset_copy_preferred_media_selection(self.ptr, &mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(MediaSelection { ptr })
    }

    pub fn all_media_selections(&self) -> Result<Vec<MediaSelection>, AVPlayerError> {
        let count = unsafe { ffi::av_asset_media_selection_count(self.ptr) };
        let count = usize::try_from(count).map_err(|error| {
            AVPlayerError::OperationFailed(format!("invalid media-selection count: {error}"))
        })?;
        let mut selections = Vec::with_capacity(count);
        for index in 0..count {
            let ptr = unsafe {
                ffi::av_asset_copy_media_selection_at_index(
                    self.ptr,
                    i32::try_from(index).unwrap_or(i32::MAX),
                )
            };
            if ptr.is_null() {
                return Err(AVPlayerError::OperationFailed(format!(
                    "bridge returned null media selection at index {index}"
                )));
            }
            selections.push(MediaSelection { ptr });
        }
        Ok(selections)
    }
}

impl UrlAsset {
    pub fn available_media_characteristics_with_media_selection_options(
        &self,
    ) -> Result<Vec<MediaCharacteristic>, AVPlayerError> {
        self.as_asset()
            .available_media_characteristics_with_media_selection_options()
    }

    pub fn media_selection_group_for_media_characteristic(
        &self,
        media_characteristic: &MediaCharacteristic,
    ) -> Result<Option<MediaSelectionGroup>, AVPlayerError> {
        self.as_asset()
            .media_selection_group_for_media_characteristic(media_characteristic)
    }

    pub fn preferred_media_selection(&self) -> Result<MediaSelection, AVPlayerError> {
        self.as_asset().preferred_media_selection()
    }

    pub fn all_media_selections(&self) -> Result<Vec<MediaSelection>, AVPlayerError> {
        self.as_asset().all_media_selections()
    }
}

fn count_with_error(
    func: unsafe extern "C" fn(*mut c_void, *mut *mut c_char) -> i32,
    ptr: *mut c_void,
) -> Result<usize, AVPlayerError> {
    let mut err: *mut c_char = ptr::null_mut();
    let count = unsafe { func(ptr, &mut err) };
    if !err.is_null() {
        return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
    }
    usize::try_from(count)
        .map_err(|error| AVPlayerError::OperationFailed(format!("invalid count: {error}")))
}

fn string_with_error(
    func: unsafe extern "C" fn(*mut c_void, *mut *mut c_char) -> *mut c_char,
    ptr: *mut c_void,
) -> Result<String, AVPlayerError> {
    let mut err: *mut c_char = ptr::null_mut();
    let value_ptr = unsafe { func(ptr, &mut err) };
    if value_ptr.is_null() {
        return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
    }
    let value = unsafe { CStr::from_ptr(value_ptr) }
        .to_string_lossy()
        .into_owned();
    unsafe { ffi::avp_string_free(value_ptr) };
    Ok(value)
}

fn string_with_input(
    func: unsafe extern "C" fn(*mut c_void, *const c_char, *mut *mut c_char) -> *mut c_char,
    ptr: *mut c_void,
    input: *const c_char,
) -> Result<String, AVPlayerError> {
    let mut err: *mut c_char = ptr::null_mut();
    let value = unsafe { func(ptr, input, &mut err) };
    if value.is_null() {
        return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
    }
    let value_string = unsafe { CStr::from_ptr(value) }
        .to_string_lossy()
        .into_owned();
    unsafe { ffi::avp_string_free(value) };
    Ok(value_string)
}

// SAFETY: These Objective-C media-selection handles are safe to transfer across
// thread boundaries; method calls are internally dispatched safely.
unsafe impl Send for MediaSelection {}
unsafe impl Send for MutableMediaSelection {}
unsafe impl Send for MediaSelectionGroup {}
unsafe impl Send for MediaSelectionOption {}
unsafe impl Send for CustomMediaSelectionScheme {}
unsafe impl Send for MediaPresentationSelector {}
unsafe impl Send for MediaPresentationSetting {}
