#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;

use serde::Deserialize;

use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::metadata::MetadataItem;
use crate::player::PlayerItem;
use crate::player_item_output::PlayerItemOutput;
use crate::retained::retain_release_wrapper;
use crate::time::TimeRange;
use crate::util::{
    deliver, json_cstring, json_payload, parse_json_and_free, to_cstring, Handler, Registration,
};

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MetadataOutputInfoPayload {
    suppresses_player_rendering: bool,
    advance_interval_for_delegate_invocation: f64,
    identifiers: Option<Vec<String>>,
    has_delegate: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TimedMetadataGroupPayload {
    time_range: TimeRange,
    items: Vec<MetadataItem>,
}

/// Mirrors the `AVPlayer` framework counterpart for `TimedMetadataGroup`.
#[derive(Debug, Clone, PartialEq)]
pub struct TimedMetadataGroup {
    /// Mirrors the `AVPlayer` framework property for `time_range`.
    pub time_range: TimeRange,
    /// Mirrors the `AVPlayer` framework property for `items`.
    pub items: Vec<MetadataItem>,
}

impl From<TimedMetadataGroupPayload> for TimedMetadataGroup {
    fn from(payload: TimedMetadataGroupPayload) -> Self {
        Self {
            time_range: payload.time_range,
            items: payload.items,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MetadataOutputEventPayload {
    event: String,
    groups: Vec<TimedMetadataGroupPayload>,
    track_present: bool,
}

/// Mirrors the `AVPlayer` framework counterpart for `MetadataOutputEvent`.
#[derive(Debug, Clone, PartialEq)]
pub enum MetadataOutputEvent {
    /// Mirrors the `AVPlayer` framework case `SequenceWasFlushed`.
    SequenceWasFlushed,
    /// Mirrors the `AVPlayer` framework case `TimedMetadataGroups`.
    TimedMetadataGroups {
        groups: Vec<TimedMetadataGroup>,
        track_present: bool,
    },
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerItemMetadataOutput`.
#[derive(Debug)]
pub struct PlayerItemMetadataOutput {
    pub(crate) ptr: *mut c_void,
}

retain_release_wrapper!(
    PlayerItemMetadataOutput,
    release = ffi::av_player_item_output_release
);

impl PlayerItemMetadataOutput {
    /// Calls the `AVPlayer` framework counterpart for `new`.
    pub fn new(identifiers: Option<&[impl AsRef<str>]>) -> Result<Self, AVPlayerError> {
        let identifiers = identifiers.map(|identifiers| {
            identifiers
                .iter()
                .map(|identifier| identifier.as_ref().to_owned())
                .collect::<Vec<_>>()
        });
        let identifiers = identifiers
            .as_ref()
            .map(|identifiers| json_cstring(identifiers, "metadata output identifiers"))
            .transpose()?;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_player_item_metadata_output_create(
                identifiers
                    .as_ref()
                    .map_or(ptr::null(), |identifiers| identifiers.as_ptr()),
                &raw mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn info(&self) -> Result<MetadataOutputInfoPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::av_player_item_metadata_output_info_json(self.ptr, &raw mut err) };
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
    ) -> Result<MetadataOutputObserver, AVPlayerError>
    where
        F: Fn(MetadataOutputEvent) + Send + Sync + 'static,
    {
        let queue_label = queue_label
            .map(|label| to_cstring(label, "metadata output queue label"))
            .transpose()?;
        let handler: Handler<MetadataOutputEvent> = Box::new(callback);
        let inner = Registration::new(
            handler,
            ffi::av_player_item_metadata_output_observer_release,
            |userdata, drop_userdata, err| unsafe {
                ffi::av_player_item_metadata_output_add_observer(
                    self.ptr,
                    queue_label
                        .as_ref()
                        .map_or(ptr::null(), |label| label.as_ptr()),
                    Some(metadata_output_event_trampoline),
                    userdata,
                    drop_userdata,
                    err,
                )
            },
        )?;
        Ok(MetadataOutputObserver { _inner: inner })
    }

    /// Calls the `AVPlayer` framework counterpart for `advance_interval_for_delegate_invocation`.
    pub fn advance_interval_for_delegate_invocation(&self) -> Result<f64, AVPlayerError> {
        Ok(self.info()?.advance_interval_for_delegate_invocation)
    }

    /// Calls the `AVPlayer` framework counterpart for `set_advance_interval_for_delegate_invocation`.
    pub fn set_advance_interval_for_delegate_invocation(&self, interval: f64) {
        unsafe { ffi::av_player_item_metadata_output_set_advance_interval(self.ptr, interval) };
    }

    /// Calls the `AVPlayer` framework counterpart for `identifiers`.
    pub fn identifiers(&self) -> Result<Vec<String>, AVPlayerError> {
        Ok(self.info()?.identifiers.unwrap_or_default())
    }
}

/// Mirrors the `AVPlayer` framework counterpart for `MetadataOutputObserver`.
#[derive(Debug)]
pub struct MetadataOutputObserver {
    _inner: Registration,
}

// SAFETY: These metadata-output handles are safe to transfer across thread
// boundaries; method calls are internally dispatched safely.
unsafe impl Send for PlayerItemMetadataOutput {}
unsafe impl Send for MetadataOutputObserver {}

impl PlayerItem {
    /// Calls the `AVPlayer` framework counterpart for `add_metadata_output`.
    pub fn add_metadata_output(
        &self,
        output: &PlayerItemMetadataOutput,
    ) -> Result<(), AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe { ffi::av_player_item_add_output(self.ptr, output.ptr, &raw mut err) };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Calls the `AVPlayer` framework counterpart for `remove_metadata_output`.
    pub fn remove_metadata_output(&self, output: &PlayerItemMetadataOutput) {
        unsafe { ffi::av_player_item_remove_output(self.ptr, output.ptr) };
    }
}

unsafe extern "C" fn metadata_output_event_trampoline(
    userdata: *mut c_void,
    payload_json: *const c_char,
) {
    let Some(payload) = (unsafe { json_payload::<MetadataOutputEventPayload>(payload_json) })
    else {
        return;
    };

    let event = match payload.event.as_str() {
        "sequence_was_flushed" => MetadataOutputEvent::SequenceWasFlushed,
        "timed_metadata_groups" => MetadataOutputEvent::TimedMetadataGroups {
            groups: payload
                .groups
                .into_iter()
                .map(TimedMetadataGroup::from)
                .collect(),
            track_present: payload.track_present,
        },
        _ => return,
    };

    unsafe { deliver(userdata, "metadata_output_event_trampoline", event) };
}
