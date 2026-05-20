#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;

use serde::Deserialize;

use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::player::PlayerItem;
use crate::player_item_output::PlayerItemOutput;
use crate::time::Time;
use crate::util::{json_cstring, parse_json_and_free, to_cstring};

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegibleOutputInfoPayload {
    suppresses_player_rendering: bool,
    advance_interval_for_delegate_invocation: f64,
    native_representation_subtypes: Vec<u32>,
    has_delegate: bool,
    text_styling_resolution: String,
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerItemLegibleOutputTextStylingResolution`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PlayerItemLegibleOutputTextStylingResolution {
    /// Mirrors the `AVPlayer` framework case `Default`.
    Default,
    /// Mirrors the `AVPlayer` framework case `SourceAndRulesOnly`.
    SourceAndRulesOnly,
    /// Mirrors the `AVPlayer` framework case `Unknown`.
    Unknown(String),
}

impl PlayerItemLegibleOutputTextStylingResolution {
    fn from_raw(raw: &str) -> Self {
        match raw {
            "default" => Self::Default,
            "source_and_rules_only" => Self::SourceAndRulesOnly,
            other => Self::Unknown(other.to_owned()),
        }
    }

    fn as_raw(&self) -> &str {
        match self {
            Self::Default => "default",
            Self::SourceAndRulesOnly => "source_and_rules_only",
            Self::Unknown(raw) => raw,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegibleOutputEventPayload {
    event: String,
    item_time: Option<Time>,
    strings: Vec<String>,
    native_sample_buffer_count: usize,
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerItemLegibleOutputEvent`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerItemLegibleOutputEvent {
    /// Mirrors the `AVPlayer` framework case `SequenceWasFlushed`.
    SequenceWasFlushed,
    /// Mirrors the `AVPlayer` framework case `AttributedStrings`.
    AttributedStrings {
        item_time: Time,
        strings: Vec<String>,
        native_sample_buffer_count: usize,
    },
}

struct LegibleOutputObserverState {
    callback: Box<dyn Fn(PlayerItemLegibleOutputEvent) + Send + 'static>,
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerItemLegibleOutput`.
#[derive(Debug)]
pub struct PlayerItemLegibleOutput {
    pub(crate) ptr: *mut c_void,
}

impl Drop for PlayerItemLegibleOutput {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_player_item_output_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl PlayerItemLegibleOutput {
    /// Calls the `AVPlayer` framework counterpart for `new`.
    pub fn new(native_representation_subtypes: Option<&[u32]>) -> Result<Self, AVPlayerError> {
        let native_representation_subtypes = native_representation_subtypes
            .map(|subtypes| json_cstring(subtypes, "native legible output subtypes"))
            .transpose()?;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_player_item_legible_output_create(
                native_representation_subtypes
                    .as_ref()
                    .map_or(ptr::null(), |subtypes| subtypes.as_ptr()),
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn info(&self) -> Result<LegibleOutputInfoPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_player_item_legible_output_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Calls the `AVPlayer` framework counterpart for `suppresses_player_rendering`.
    pub fn suppresses_player_rendering(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.suppresses_player_rendering)
    }

    /// Calls the `AVPlayer` framework counterpart for `has_delegate`.
    pub fn has_delegate(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.has_delegate)
    }

    /// Mirrors the `AVPlayer` framework constant `fn`.
    pub const fn as_output(&self) -> PlayerItemOutput<'_> {
        PlayerItemOutput::from_ptr(self.ptr)
    }

    /// Calls the `AVPlayer` framework counterpart for `set_suppresses_player_rendering`.
    pub fn set_suppresses_player_rendering(&self, suppresses: bool) {
        unsafe { ffi::av_player_item_output_set_suppresses_player_rendering(self.ptr, suppresses) };
    }

    /// Calls the `AVPlayer` framework counterpart for `observe`.
    pub fn observe<F>(
        &self,
        queue_label: Option<&str>,
        callback: F,
    ) -> Result<PlayerItemLegibleOutputObserver, AVPlayerError>
    where
        F: Fn(PlayerItemLegibleOutputEvent) + Send + 'static,
    {
        let queue_label = queue_label
            .map(|label| to_cstring(label, "legible output queue label"))
            .transpose()?;
        let state = Box::new(LegibleOutputObserverState {
            callback: Box::new(callback),
        });
        let userdata = Box::into_raw(state).cast::<c_void>();
        let mut err: *mut c_char = ptr::null_mut();
        let token = unsafe {
            ffi::av_player_item_legible_output_add_observer(
                self.ptr,
                queue_label
                    .as_ref()
                    .map_or(ptr::null(), |label| label.as_ptr()),
                Some(legible_output_event_trampoline),
                userdata,
                Some(legible_output_observer_drop),
                &mut err,
            )
        };
        if token.is_null() {
            unsafe { legible_output_observer_drop(userdata) };
            return Err(unsafe { from_swift(ffi::status::OBSERVER_FAILED, err) });
        }
        Ok(PlayerItemLegibleOutputObserver { token })
    }

    /// Calls the `AVPlayer` framework counterpart for `advance_interval_for_delegate_invocation`.
    pub fn advance_interval_for_delegate_invocation(&self) -> Result<f64, AVPlayerError> {
        Ok(self.info()?.advance_interval_for_delegate_invocation)
    }

    /// Calls the `AVPlayer` framework counterpart for `set_advance_interval_for_delegate_invocation`.
    pub fn set_advance_interval_for_delegate_invocation(&self, interval: f64) {
        unsafe { ffi::av_player_item_legible_output_set_advance_interval(self.ptr, interval) };
    }

    /// Calls the `AVPlayer` framework counterpart for `native_representation_subtypes`.
    pub fn native_representation_subtypes(&self) -> Result<Vec<u32>, AVPlayerError> {
        Ok(self.info()?.native_representation_subtypes)
    }

    /// Calls the `AVPlayer` framework counterpart for `text_styling_resolution`.
    pub fn text_styling_resolution(
        &self,
    ) -> Result<PlayerItemLegibleOutputTextStylingResolution, AVPlayerError> {
        Ok(PlayerItemLegibleOutputTextStylingResolution::from_raw(
            &self.info()?.text_styling_resolution,
        ))
    }

    /// Calls the `AVPlayer` framework counterpart for `set_text_styling_resolution`.
    pub fn set_text_styling_resolution(
        &self,
        resolution: &PlayerItemLegibleOutputTextStylingResolution,
    ) -> Result<(), AVPlayerError> {
        let resolution = to_cstring(
            resolution.as_raw(),
            "legible output text styling resolution",
        )?;
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::av_player_item_legible_output_set_text_styling_resolution(
                self.ptr,
                resolution.as_ptr(),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerItemLegibleOutputObserver`.
#[derive(Debug)]
pub struct PlayerItemLegibleOutputObserver {
    token: *mut c_void,
}

impl Drop for PlayerItemLegibleOutputObserver {
    fn drop(&mut self) {
        if !self.token.is_null() {
            unsafe { ffi::av_player_item_legible_output_observer_release(self.token) };
            self.token = ptr::null_mut();
        }
    }
}

// SAFETY: These legible-output handles are safe to transfer across thread
// boundaries; method calls are internally dispatched safely.
unsafe impl Send for PlayerItemLegibleOutput {}
unsafe impl Send for PlayerItemLegibleOutputObserver {}

impl PlayerItem {
    /// Calls the `AVPlayer` framework counterpart for `add_legible_output`.
    pub fn add_legible_output(
        &self,
        output: &PlayerItemLegibleOutput,
    ) -> Result<(), AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe { ffi::av_player_item_add_output(self.ptr, output.ptr, &mut err) };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Calls the `AVPlayer` framework counterpart for `remove_legible_output`.
    pub fn remove_legible_output(&self, output: &PlayerItemLegibleOutput) {
        unsafe { ffi::av_player_item_remove_output(self.ptr, output.ptr) };
    }
}

unsafe extern "C" fn legible_output_event_trampoline(
    userdata: *mut c_void,
    payload_json: *const c_char,
) {
    if userdata.is_null() || payload_json.is_null() {
        return;
    }

    let callback = &*userdata.cast::<LegibleOutputObserverState>();
    let Ok(payload) = core::ffi::CStr::from_ptr(payload_json).to_str() else {
        return;
    };
    let Ok(payload) = serde_json::from_str::<LegibleOutputEventPayload>(payload) else {
        return;
    };

    let event = match payload.event.as_str() {
        "sequence_was_flushed" => PlayerItemLegibleOutputEvent::SequenceWasFlushed,
        "attributed_strings" => PlayerItemLegibleOutputEvent::AttributedStrings {
            item_time: match payload.item_time {
                Some(item_time) => item_time,
                None => return,
            },
            strings: payload.strings,
            native_sample_buffer_count: payload.native_sample_buffer_count,
        },
        _ => return,
    };

    crate::util::catch_cb_panic("legible_output_event_trampoline", || {
        (callback.callback)(event);
    });
}

unsafe extern "C" fn legible_output_observer_drop(userdata: *mut c_void) {
    if !userdata.is_null() {
        drop(Box::from_raw(userdata.cast::<LegibleOutputObserverState>()));
    }
}
