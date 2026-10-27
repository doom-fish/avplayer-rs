#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use crate::ffi::AVPValue;
use core::ffi::c_void;
use core::ptr;

use apple_cf::cm::CMSampleBuffer;

use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::player::PlayerItem;
use crate::player_item_output::PlayerItemOutput;
use crate::retained::retain_release_wrapper;
use crate::time::Time;
use crate::util::{
    borrowed_objects, deliver, parse_value_and_free, to_cstring, value_buffer, value_payload,
    Handler, Registration,
};

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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
        native_sample_buffers: Vec<CMSampleBuffer>,
    },
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerItemLegibleOutput`.
#[derive(Debug)]
pub struct PlayerItemLegibleOutput {
    pub(crate) ptr: *mut c_void,
}

retain_release_wrapper!(
    PlayerItemLegibleOutput,
    release = ffi::av_player_item_output_release
);

impl PlayerItemLegibleOutput {
    /// Calls the `AVPlayer` framework counterpart for `new`.
    pub fn new(native_representation_subtypes: Option<&[u32]>) -> Result<Self, AVPlayerError> {
        let native_representation_subtypes = native_representation_subtypes
            .map(|subtypes| value_buffer(subtypes, "native legible output subtypes"))
            .transpose()?;
        let mut err: *mut AVPValue = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_player_item_legible_output_create(
                native_representation_subtypes
                    .as_ref()
                    .map_or(ptr::null(), super::binary::InputValue::as_ptr),
                &raw mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn info(&self) -> Result<LegibleOutputInfoPayload, AVPlayerError> {
        let mut err: *mut AVPValue = ptr::null_mut();
        let value_ptr =
            unsafe { ffi::av_player_item_legible_output_info_value(self.ptr, &raw mut err) };
        if value_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_value_and_free(value_ptr)
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
        F: Fn(PlayerItemLegibleOutputEvent) + Send + Sync + 'static,
    {
        let queue_label = queue_label
            .map(|label| to_cstring(label, "legible output queue label"))
            .transpose()?;
        let handler: Handler<PlayerItemLegibleOutputEvent> = Box::new(callback);
        let inner = Registration::new(
            handler,
            ffi::av_player_item_legible_output_observer_release,
            |userdata, drop_userdata, err| unsafe {
                ffi::av_player_item_legible_output_add_observer(
                    self.ptr,
                    queue_label
                        .as_ref()
                        .map_or(ptr::null(), super::binary::InputString::as_ptr),
                    Some(legible_output_event_trampoline),
                    userdata,
                    drop_userdata,
                    err,
                )
            },
        )?;
        Ok(PlayerItemLegibleOutputObserver { _inner: inner })
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
        let mut err: *mut AVPValue = ptr::null_mut();
        let status = unsafe {
            ffi::av_player_item_legible_output_set_text_styling_resolution(
                self.ptr,
                resolution.as_ptr(),
                &raw mut err,
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
    _inner: Registration,
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
        let mut err: *mut AVPValue = ptr::null_mut();
        let status = unsafe { ffi::av_player_item_add_output(self.ptr, output.ptr, &raw mut err) };
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
    payload_value: *const AVPValue,
    objects: *const *mut c_void,
    count: usize,
) {
    let Some(payload) = (unsafe { value_payload::<LegibleOutputEventPayload>(payload_value) })
    else {
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
            native_sample_buffers: unsafe {
                borrowed_objects(objects, count, |object| {
                    CMSampleBuffer::from_raw_borrowed(object)
                })
            },
        },
        _ => return,
    };

    unsafe { deliver(userdata, "legible_output_event_trampoline", event) };
}

impl crate::binary::FromValue for LegibleOutputInfoPayload {
    fn from_value(value: &crate::binary::Value) -> Result<Self, AVPlayerError> {
        Ok(Self {
            suppresses_player_rendering: value.field(0)?,
            advance_interval_for_delegate_invocation: value.field(1)?,
            native_representation_subtypes: value.field(2)?,
            has_delegate: value.field(3)?,
            text_styling_resolution: value.field(4)?,
        })
    }
}

impl crate::binary::FromValue for LegibleOutputEventPayload {
    fn from_value(value: &crate::binary::Value) -> Result<Self, AVPlayerError> {
        Ok(Self {
            event: value.field(0)?,
            item_time: value.field(1)?,
            strings: value.field(2)?,
            native_sample_buffer_count: value.field(3)?,
        })
    }
}
