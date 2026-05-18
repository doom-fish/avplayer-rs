#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;

use serde::Deserialize;

use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::metadata::MetadataItem;
use crate::player::PlayerItem;
use crate::util::{json_cstring, parse_json_and_free};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PlayerItemMediaDataCollectorKind {
    MetadataCollector,
    Unknown(String),
}

impl PlayerItemMediaDataCollectorKind {
    fn from_raw(raw: &str) -> Self {
        match raw {
            "metadata_collector" => Self::MetadataCollector,
            other => Self::Unknown(other.to_owned()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerItemMediaDataCollectorInfoPayload {
    kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerItemMediaDataCollectorInfo {
    pub kind: PlayerItemMediaDataCollectorKind,
}

impl From<PlayerItemMediaDataCollectorInfoPayload> for PlayerItemMediaDataCollectorInfo {
    fn from(payload: PlayerItemMediaDataCollectorInfoPayload) -> Self {
        Self {
            kind: PlayerItemMediaDataCollectorKind::from_raw(&payload.kind),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MetadataCollectorInfoPayload {
    identifiers: Option<Vec<String>>,
    classifying_labels: Option<Vec<String>>,
    has_delegate: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DateRangeMetadataGroup {
    pub start_date: String,
    pub end_date: Option<String>,
    pub classifying_label: Option<String>,
    pub unique_id: Option<String>,
    pub items: Vec<MetadataItem>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MetadataCollectorEventPayload {
    event: String,
    groups: Vec<DateRangeMetadataGroup>,
    new_indices: Vec<usize>,
    modified_indices: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum MetadataCollectorEvent {
    DidCollectDateRangeMetadataGroups {
        groups: Vec<DateRangeMetadataGroup>,
        new_indices: Vec<usize>,
        modified_indices: Vec<usize>,
    },
}

struct MetadataCollectorObserverState {
    callback: Box<dyn Fn(MetadataCollectorEvent) + Send + 'static>,
}

#[derive(Debug)]
pub struct PlayerItemMetadataCollector {
    pub(crate) ptr: *mut c_void,
}

impl Drop for PlayerItemMetadataCollector {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_player_item_metadata_collector_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl PlayerItemMetadataCollector {
    pub fn new(
        identifiers: Option<&[impl AsRef<str>]>,
        classifying_labels: Option<&[impl AsRef<str>]>,
    ) -> Result<Self, AVPlayerError> {
        let identifiers = identifiers.map(|values| {
            values
                .iter()
                .map(|value| value.as_ref().to_owned())
                .collect::<Vec<_>>()
        });
        let classifying_labels = classifying_labels.map(|values| {
            values
                .iter()
                .map(|value| value.as_ref().to_owned())
                .collect::<Vec<_>>()
        });
        let identifiers = identifiers
            .as_ref()
            .map(|values| json_cstring(values, "metadata collector identifiers"))
            .transpose()?;
        let classifying_labels = classifying_labels
            .as_ref()
            .map(|values| json_cstring(values, "metadata collector classifying labels"))
            .transpose()?;

        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_player_item_metadata_collector_create(
                identifiers
                    .as_ref()
                    .map_or(ptr::null(), |values| values.as_ptr()),
                classifying_labels
                    .as_ref()
                    .map_or(ptr::null(), |values| values.as_ptr()),
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn info(&self) -> Result<MetadataCollectorInfoPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::av_player_item_metadata_collector_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn identifiers(&self) -> Result<Vec<String>, AVPlayerError> {
        Ok(self.info()?.identifiers.unwrap_or_default())
    }

    pub fn classifying_labels(&self) -> Result<Vec<String>, AVPlayerError> {
        Ok(self.info()?.classifying_labels.unwrap_or_default())
    }

    pub fn has_delegate(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.has_delegate)
    }

    pub fn observe<F>(
        &self,
        queue_label: Option<&str>,
        callback: F,
    ) -> Result<MetadataCollectorObserver, AVPlayerError>
    where
        F: Fn(MetadataCollectorEvent) + Send + 'static,
    {
        let queue_label = queue_label
            .map(|label| crate::util::to_cstring(label, "metadata collector queue label"))
            .transpose()?;
        let state = Box::new(MetadataCollectorObserverState {
            callback: Box::new(callback),
        });
        let userdata = Box::into_raw(state).cast::<c_void>();
        let mut err: *mut c_char = ptr::null_mut();
        let token = unsafe {
            ffi::av_player_item_metadata_collector_add_observer(
                self.ptr,
                queue_label
                    .as_ref()
                    .map_or(ptr::null(), |label| label.as_ptr()),
                Some(metadata_collector_event_trampoline),
                userdata,
                Some(metadata_collector_observer_drop),
                &mut err,
            )
        };
        if token.is_null() {
            unsafe { metadata_collector_observer_drop(userdata) };
            return Err(unsafe { from_swift(ffi::status::OBSERVER_FAILED, err) });
        }
        Ok(MetadataCollectorObserver { token })
    }
}

#[derive(Debug)]
pub struct MetadataCollectorObserver {
    token: *mut c_void,
}

impl Drop for MetadataCollectorObserver {
    fn drop(&mut self) {
        if !self.token.is_null() {
            unsafe { ffi::av_player_item_metadata_collector_observer_release(self.token) };
            self.token = ptr::null_mut();
        }
    }
}

// SAFETY: These metadata-collector handles are safe to transfer across thread
// boundaries; method calls are internally dispatched safely.
unsafe impl Send for PlayerItemMetadataCollector {}
unsafe impl Send for MetadataCollectorObserver {}

impl PlayerItem {
    pub fn add_metadata_collector(
        &self,
        collector: &PlayerItemMetadataCollector,
    ) -> Result<(), AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::av_player_item_add_media_data_collector(self.ptr, collector.ptr, &mut err)
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn remove_metadata_collector(&self, collector: &PlayerItemMetadataCollector) {
        unsafe { ffi::av_player_item_remove_media_data_collector(self.ptr, collector.ptr) };
    }

    pub fn media_data_collectors(
        &self,
    ) -> Result<Vec<PlayerItemMediaDataCollectorInfo>, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::av_player_item_media_data_collectors_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(
            parse_json_and_free::<Vec<PlayerItemMediaDataCollectorInfoPayload>>(json_ptr)?
                .into_iter()
                .map(PlayerItemMediaDataCollectorInfo::from)
                .collect(),
        )
    }
}

unsafe extern "C" fn metadata_collector_event_trampoline(
    userdata: *mut c_void,
    payload_json: *const c_char,
) {
    if userdata.is_null() || payload_json.is_null() {
        return;
    }

    let callback = &*userdata.cast::<MetadataCollectorObserverState>();
    let Ok(payload) = core::ffi::CStr::from_ptr(payload_json).to_str() else {
        return;
    };
    let Ok(payload) = serde_json::from_str::<MetadataCollectorEventPayload>(payload) else {
        return;
    };

    let event = match payload.event.as_str() {
        "did_collect_date_range_metadata_groups" => {
            MetadataCollectorEvent::DidCollectDateRangeMetadataGroups {
                groups: payload.groups,
                new_indices: payload.new_indices,
                modified_indices: payload.modified_indices,
            }
        }
        _ => return,
    };

    crate::util::catch_cb_panic("metadata_collector_event_trampoline", || {
        (callback.callback)(event);
    });
}

unsafe extern "C" fn metadata_collector_observer_drop(userdata: *mut c_void) {
    if !userdata.is_null() {
        drop(Box::from_raw(
            userdata.cast::<MetadataCollectorObserverState>(),
        ));
    }
}
