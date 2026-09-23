#![allow(
    clippy::derive_partial_eq_without_eq,
    clippy::missing_errors_doc,
    clippy::must_use_candidate
)]

use core::ffi::{c_char, c_void};
use core::ptr;

use serde::Deserialize;

use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::player::PlayerItem;
use crate::player_interstitial_event::{
    PlayerInterstitialEventInfo, PlayerInterstitialEventInfoPayload,
};
use crate::retained::retain_release_wrapper;
use crate::time::{Time, TimeRange};
use crate::util::{
    deliver, json_payload, parse_json_and_free, to_cstring, validate_seek_time, validate_tolerance,
    Handler, Registration,
};

/// Mirrors the `AVPlayer` framework counterpart for `PlayerItemSegmentType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PlayerItemSegmentType {
    /// Mirrors the `AVPlayer` framework case `Primary`.
    Primary,
    /// Mirrors the `AVPlayer` framework case `Interstitial`.
    Interstitial,
    /// Mirrors the `AVPlayer` framework case `Unknown`.
    Unknown(i32),
}

impl PlayerItemSegmentType {
    const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::Primary,
            1 => Self::Interstitial,
            other => Self::Unknown(other),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerItemIntegratedTimelineInfoPayload {
    current_time: Time,
    current_date: Option<String>,
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerItemIntegratedTimelineInfo`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerItemIntegratedTimelineInfo {
    /// Mirrors the `AVPlayer` framework property for `current_time`.
    pub current_time: Time,
    /// Mirrors the `AVPlayer` framework property for `current_date`.
    pub current_date: Option<String>,
}

impl From<PlayerItemIntegratedTimelineInfoPayload> for PlayerItemIntegratedTimelineInfo {
    fn from(payload: PlayerItemIntegratedTimelineInfoPayload) -> Self {
        Self {
            current_time: payload.current_time,
            current_date: payload.current_date,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerItemIntegratedTimelineSegmentPayload {
    segment_type_raw: i32,
    time_mapping_source: TimeRange,
    time_mapping_target: TimeRange,
    loaded_time_ranges: Vec<TimeRange>,
    start_date: Option<String>,
    interstitial_event: Option<PlayerInterstitialEventInfoPayload>,
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerItemIntegratedTimelineSegmentInfo`.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerItemIntegratedTimelineSegmentInfo {
    /// Mirrors the `AVPlayer` framework property for `segment_type`.
    pub segment_type: PlayerItemSegmentType,
    /// Mirrors the `AVPlayer` framework property for `time_mapping_source`.
    pub time_mapping_source: TimeRange,
    /// Mirrors the `AVPlayer` framework property for `time_mapping_target`.
    pub time_mapping_target: TimeRange,
    /// Mirrors the `AVPlayer` framework property for `loaded_time_ranges`.
    pub loaded_time_ranges: Vec<TimeRange>,
    /// Mirrors the `AVPlayer` framework property for `start_date`.
    pub start_date: Option<String>,
    /// Mirrors the `AVPlayer` framework property for `interstitial_event`.
    pub interstitial_event: Option<PlayerInterstitialEventInfo>,
}

impl TryFrom<PlayerItemIntegratedTimelineSegmentPayload>
    for PlayerItemIntegratedTimelineSegmentInfo
{
    type Error = AVPlayerError;

    fn try_from(payload: PlayerItemIntegratedTimelineSegmentPayload) -> Result<Self, Self::Error> {
        Ok(Self {
            segment_type: PlayerItemSegmentType::from_raw(payload.segment_type_raw),
            time_mapping_source: payload.time_mapping_source,
            time_mapping_target: payload.time_mapping_target,
            loaded_time_ranges: payload.loaded_time_ranges,
            start_date: payload.start_date,
            interstitial_event: payload
                .interstitial_event
                .map(PlayerInterstitialEventInfo::try_from)
                .transpose()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerItemIntegratedTimelineSnapshotPayload {
    duration: Time,
    current_time: Time,
    current_date: Option<String>,
    current_segment_index: Option<usize>,
    segments: Vec<PlayerItemIntegratedTimelineSegmentPayload>,
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerItemIntegratedTimelineSnapshotInfo`.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerItemIntegratedTimelineSnapshotInfo {
    /// Mirrors the `AVPlayer` framework property for `duration`.
    pub duration: Time,
    /// Mirrors the `AVPlayer` framework property for `current_time`.
    pub current_time: Time,
    /// Mirrors the `AVPlayer` framework property for `current_date`.
    pub current_date: Option<String>,
    /// Mirrors the `AVPlayer` framework property for `current_segment_index`.
    pub current_segment_index: Option<usize>,
    /// Mirrors the `AVPlayer` framework property for `current_segment`.
    pub current_segment: Option<PlayerItemIntegratedTimelineSegmentInfo>,
    /// Mirrors the `AVPlayer` framework property for `segments`.
    pub segments: Vec<PlayerItemIntegratedTimelineSegmentInfo>,
}

impl TryFrom<PlayerItemIntegratedTimelineSnapshotPayload>
    for PlayerItemIntegratedTimelineSnapshotInfo
{
    type Error = AVPlayerError;

    fn try_from(payload: PlayerItemIntegratedTimelineSnapshotPayload) -> Result<Self, Self::Error> {
        let segments = payload
            .segments
            .into_iter()
            .map(PlayerItemIntegratedTimelineSegmentInfo::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        let current_segment = payload
            .current_segment_index
            .and_then(|index| segments.get(index).cloned());
        Ok(Self {
            duration: payload.duration,
            current_time: payload.current_time,
            current_date: payload.current_date,
            current_segment_index: payload.current_segment_index,
            current_segment,
            segments,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerItemIntegratedTimelineSegmentOffsetPayload {
    segment: PlayerItemIntegratedTimelineSegmentPayload,
    offset: Time,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerIntegratedTimelineOutOfSyncPayload {
    reason: String,
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerIntegratedTimelineSnapshotsOutOfSyncReason`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PlayerIntegratedTimelineSnapshotsOutOfSyncReason {
    /// Mirrors the `AVPlayer` framework case `SegmentsChanged`.
    SegmentsChanged,
    /// Mirrors the `AVPlayer` framework case `CurrentSegmentChanged`.
    CurrentSegmentChanged,
    /// Mirrors the `AVPlayer` framework case `LoadedTimeRangesChanged`.
    LoadedTimeRangesChanged,
    /// Mirrors the `AVPlayer` framework case `Unknown`.
    Unknown(String),
}

impl PlayerIntegratedTimelineSnapshotsOutOfSyncReason {
    fn from_raw(raw: &str) -> Self {
        match raw {
            "AVPlayerIntegratedTimelineSnapshotsOutOfSyncReasonSegmentsChanged" => {
                Self::SegmentsChanged
            }
            "AVPlayerIntegratedTimelineSnapshotsOutOfSyncReasonCurrentSegmentChanged" => {
                Self::CurrentSegmentChanged
            }
            "AVPlayerIntegratedTimelineSnapshotsOutOfSyncReasonLoadedTimeRangesChanged" => {
                Self::LoadedTimeRangesChanged
            }
            other => Self::Unknown(other.to_owned()),
        }
    }
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerIntegratedTimelineOutOfSyncEvent`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlayerIntegratedTimelineOutOfSyncEvent {
    /// Mirrors the `AVPlayer` framework property for `reason`.
    pub reason: PlayerIntegratedTimelineSnapshotsOutOfSyncReason,
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerItemIntegratedTimeline`.
#[derive(Debug)]
pub struct PlayerItemIntegratedTimeline {
    pub(crate) ptr: *mut c_void,
}

retain_release_wrapper!(
    PlayerItemIntegratedTimeline,
    release = ffi::av_player_item_integrated_timeline_release
);

impl PlayerItemIntegratedTimeline {
    /// Calls the `AVPlayer` framework counterpart for `info`.
    pub fn info(&self) -> Result<PlayerItemIntegratedTimelineInfo, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::av_player_item_integrated_timeline_info_json(self.ptr, &raw mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(parse_json_and_free::<PlayerItemIntegratedTimelineInfoPayload>(json_ptr)?.into())
    }

    /// Calls the `AVPlayer` framework counterpart for `current_snapshot`.
    pub fn current_snapshot(&self) -> Result<PlayerItemIntegratedTimelineSnapshot, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_player_item_integrated_timeline_copy_current_snapshot(self.ptr, &raw mut err)
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(PlayerItemIntegratedTimelineSnapshot { ptr })
    }

    /// Calls the `AVPlayer` framework counterpart for `seek_to_time`.
    pub fn seek_to_time(
        &self,
        time: Time,
        tolerance_before: Time,
        tolerance_after: Time,
    ) -> Result<bool, AVPlayerError> {
        validate_seek_time(time, "seek time")?;
        validate_tolerance(tolerance_before, "tolerance before")?;
        validate_tolerance(tolerance_after, "tolerance after")?;
        let (time_value, time_timescale, time_kind) = time.to_raw();
        let (before_value, before_timescale, before_kind) = tolerance_before.to_raw();
        let (after_value, after_timescale, after_kind) = tolerance_after.to_raw();
        let mut err: *mut c_char = ptr::null_mut();
        let mut success = false;
        let status = unsafe {
            ffi::av_player_item_integrated_timeline_seek_to_time(
                self.ptr,
                time_value,
                time_timescale,
                time_kind,
                before_value,
                before_timescale,
                before_kind,
                after_value,
                after_timescale,
                after_kind,
                &raw mut success,
                &raw mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(success)
    }

    /// Calls the `AVPlayer` framework counterpart for `seek_to_date`.
    pub fn seek_to_date(&self, date: &str) -> Result<bool, AVPlayerError> {
        let date = to_cstring(date, "integrated timeline date")?;
        let mut err: *mut c_char = ptr::null_mut();
        let mut success = false;
        let status = unsafe {
            ffi::av_player_item_integrated_timeline_seek_to_date(
                self.ptr,
                date.as_ptr(),
                &raw mut success,
                &raw mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(success)
    }

    /// Calls the `AVPlayer` framework counterpart for `observe_periodic_times`.
    pub fn observe_periodic_times<F>(
        &self,
        interval: Time,
        callback: F,
    ) -> Result<PlayerItemIntegratedTimelineObserver, AVPlayerError>
    where
        F: Fn(Time) + Send + Sync + 'static,
    {
        let (value, timescale, kind) = interval.to_raw();
        let handler: Handler<Time> = Box::new(callback);
        let inner = Registration::new(
            handler,
            ffi::av_player_item_integrated_timeline_observer_release,
            |userdata, drop_userdata, err| unsafe {
                ffi::av_player_item_integrated_timeline_add_periodic_time_observer(
                    self.ptr,
                    value,
                    timescale,
                    kind,
                    Some(player_item_integrated_timeline_time_trampoline),
                    userdata,
                    drop_userdata,
                    err,
                )
            },
        )?;
        Ok(PlayerItemIntegratedTimelineObserver { _inner: inner })
    }

    /// Calls the `AVPlayer` framework counterpart for `observe_boundary_times`.
    pub fn observe_boundary_times<F>(
        &self,
        segment: &PlayerItemIntegratedTimelineSegment,
        offsets_into_segment: &[Time],
        callback: F,
    ) -> Result<PlayerItemIntegratedTimelineObserver, AVPlayerError>
    where
        F: Fn(Time) + Send + Sync + 'static,
    {
        let offsets_json = serde_json::to_string(offsets_into_segment).map_err(|error| {
            AVPlayerError::InvalidArgument(format!(
                "failed to encode integrated timeline offsets: {error}"
            ))
        })?;
        let offsets_json = to_cstring(&offsets_json, "integrated timeline boundary offsets")?;
        let handler: Handler<Time> = Box::new(callback);
        let inner = Registration::new(
            handler,
            ffi::av_player_item_integrated_timeline_observer_release,
            |userdata, drop_userdata, err| unsafe {
                ffi::av_player_item_integrated_timeline_add_boundary_time_observer(
                    self.ptr,
                    segment.ptr,
                    offsets_json.as_ptr(),
                    Some(player_item_integrated_timeline_time_trampoline),
                    userdata,
                    drop_userdata,
                    err,
                )
            },
        )?;
        Ok(PlayerItemIntegratedTimelineObserver { _inner: inner })
    }

    /// Calls the `AVPlayer` framework counterpart for `observe_snapshots_out_of_sync`.
    pub fn observe_snapshots_out_of_sync<F>(
        &self,
        callback: F,
    ) -> Result<PlayerItemIntegratedTimelineObserver, AVPlayerError>
    where
        F: Fn(PlayerIntegratedTimelineOutOfSyncEvent) + Send + Sync + 'static,
    {
        let handler: Handler<PlayerIntegratedTimelineOutOfSyncEvent> = Box::new(callback);
        let inner = Registration::new(
            handler,
            ffi::av_player_item_integrated_timeline_observer_release,
            |userdata, drop_userdata, err| unsafe {
                ffi::av_player_item_integrated_timeline_add_out_of_sync_observer(
                    self.ptr,
                    Some(player_item_integrated_timeline_out_of_sync_trampoline),
                    userdata,
                    drop_userdata,
                    err,
                )
            },
        )?;
        Ok(PlayerItemIntegratedTimelineObserver { _inner: inner })
    }
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerItemIntegratedTimelineSnapshot`.
#[derive(Debug)]
pub struct PlayerItemIntegratedTimelineSnapshot {
    pub(crate) ptr: *mut c_void,
}

retain_release_wrapper!(
    PlayerItemIntegratedTimelineSnapshot,
    release = ffi::av_player_item_integrated_timeline_snapshot_release
);

impl PlayerItemIntegratedTimelineSnapshot {
    /// Calls the `AVPlayer` framework counterpart for `info`.
    pub fn info(&self) -> Result<PlayerItemIntegratedTimelineSnapshotInfo, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::av_player_item_integrated_timeline_snapshot_info_json(self.ptr, &raw mut err)
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        PlayerItemIntegratedTimelineSnapshotInfo::try_from(parse_json_and_free::<
            PlayerItemIntegratedTimelineSnapshotPayload,
        >(json_ptr)?)
    }

    /// Calls the `AVPlayer` framework counterpart for `current_segment`.
    pub fn current_segment(&self) -> Option<PlayerItemIntegratedTimelineSegment> {
        let ptr = unsafe {
            ffi::av_player_item_integrated_timeline_snapshot_copy_current_segment(self.ptr)
        };
        if ptr.is_null() {
            None
        } else {
            Some(PlayerItemIntegratedTimelineSegment { ptr })
        }
    }

    /// Calls the `AVPlayer` framework counterpart for `segment_count`.
    pub fn segment_count(&self) -> usize {
        unsafe { ffi::av_player_item_integrated_timeline_snapshot_segment_count(self.ptr) }
    }

    /// Calls the `AVPlayer` framework counterpart for `segment_at_index`.
    pub fn segment_at_index(&self, index: usize) -> Option<PlayerItemIntegratedTimelineSegment> {
        let ptr = unsafe {
            ffi::av_player_item_integrated_timeline_snapshot_copy_segment_at_index(self.ptr, index)
        };
        if ptr.is_null() {
            None
        } else {
            Some(PlayerItemIntegratedTimelineSegment { ptr })
        }
    }

    /// Calls the `AVPlayer` framework counterpart for `segment_and_offset_into_segment`.
    pub fn segment_and_offset_into_segment(
        &self,
        timeline_time: Time,
    ) -> Result<(PlayerItemIntegratedTimelineSegmentInfo, Time), AVPlayerError> {
        let (value, timescale, kind) = timeline_time.to_raw();
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::av_player_item_integrated_timeline_snapshot_segment_and_offset_json(
                self.ptr,
                value,
                timescale,
                kind,
                &raw mut err,
            )
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        let payload =
            parse_json_and_free::<PlayerItemIntegratedTimelineSegmentOffsetPayload>(json_ptr)?;
        Ok((
            PlayerItemIntegratedTimelineSegmentInfo::try_from(payload.segment)?,
            payload.offset,
        ))
    }
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerItemIntegratedTimelineSegment`.
#[derive(Debug)]
pub struct PlayerItemIntegratedTimelineSegment {
    pub(crate) ptr: *mut c_void,
}

retain_release_wrapper!(
    PlayerItemIntegratedTimelineSegment,
    release = ffi::av_player_item_integrated_timeline_segment_release
);

impl PlayerItemIntegratedTimelineSegment {
    /// Calls the `AVPlayer` framework counterpart for `info`.
    pub fn info(&self) -> Result<PlayerItemIntegratedTimelineSegmentInfo, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::av_player_item_integrated_timeline_segment_info_json(self.ptr, &raw mut err)
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        PlayerItemIntegratedTimelineSegmentInfo::try_from(parse_json_and_free::<
            PlayerItemIntegratedTimelineSegmentPayload,
        >(json_ptr)?)
    }
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerItemIntegratedTimelineObserver`.
#[derive(Debug)]
pub struct PlayerItemIntegratedTimelineObserver {
    _inner: Registration,
}

// SAFETY: These integrated-timeline wrapper handles are safe to transfer across
// thread boundaries; method calls are internally dispatched safely.
unsafe impl Send for PlayerItemIntegratedTimeline {}
unsafe impl Send for PlayerItemIntegratedTimelineSnapshot {}
unsafe impl Send for PlayerItemIntegratedTimelineSegment {}
unsafe impl Send for PlayerItemIntegratedTimelineObserver {}

/// Calls the `AVPlayer` framework counterpart for `player_integrated_timeline_snapshots_out_of_sync_notification`.
pub fn player_integrated_timeline_snapshots_out_of_sync_notification(
) -> Result<String, AVPlayerError> {
    let mut err: *mut c_char = ptr::null_mut();
    let ptr = unsafe {
        ffi::av_player_integrated_timeline_snapshots_out_of_sync_notification(&raw mut err)
    };
    if ptr.is_null() {
        return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
    }
    parse_json_and_free::<String>(ptr)
}

/// Calls the `AVPlayer` framework counterpart for `player_integrated_timeline_snapshots_out_of_sync_reason_key`.
pub fn player_integrated_timeline_snapshots_out_of_sync_reason_key() -> Result<String, AVPlayerError>
{
    let mut err: *mut c_char = ptr::null_mut();
    let ptr = unsafe {
        ffi::av_player_integrated_timeline_snapshots_out_of_sync_reason_key(&raw mut err)
    };
    if ptr.is_null() {
        return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
    }
    parse_json_and_free::<String>(ptr)
}

/// Calls the `AVPlayer` framework counterpart for `player_integrated_timeline_snapshots_out_of_sync_reason_segments_changed`.
pub fn player_integrated_timeline_snapshots_out_of_sync_reason_segments_changed(
) -> Result<String, AVPlayerError> {
    let mut err: *mut c_char = ptr::null_mut();
    let ptr = unsafe {
        ffi::av_player_integrated_timeline_snapshots_out_of_sync_reason_segments_changed(
            &raw mut err,
        )
    };
    if ptr.is_null() {
        return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
    }
    parse_json_and_free::<String>(ptr)
}

/// Calls the `AVPlayer` framework counterpart for `player_integrated_timeline_snapshots_out_of_sync_reason_current_segment_changed`.
pub fn player_integrated_timeline_snapshots_out_of_sync_reason_current_segment_changed(
) -> Result<String, AVPlayerError> {
    let mut err: *mut c_char = ptr::null_mut();
    let ptr = unsafe {
        ffi::av_player_integrated_timeline_snapshots_out_of_sync_reason_current_segment_changed(
            &raw mut err,
        )
    };
    if ptr.is_null() {
        return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
    }
    parse_json_and_free::<String>(ptr)
}

/// Calls the `AVPlayer` framework counterpart for `player_integrated_timeline_snapshots_out_of_sync_reason_loaded_time_ranges_changed`.
pub fn player_integrated_timeline_snapshots_out_of_sync_reason_loaded_time_ranges_changed(
) -> Result<String, AVPlayerError> {
    let mut err: *mut c_char = ptr::null_mut();
    let ptr = unsafe {
        ffi::av_player_integrated_timeline_snapshots_out_of_sync_reason_loaded_time_ranges_changed(
            &raw mut err,
        )
    };
    if ptr.is_null() {
        return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
    }
    parse_json_and_free::<String>(ptr)
}

impl PlayerItem {
    /// Calls the `AVPlayer` framework counterpart for `integrated_timeline`.
    pub fn integrated_timeline(&self) -> Result<PlayerItemIntegratedTimeline, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::av_player_item_copy_integrated_timeline(self.ptr, &raw mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(PlayerItemIntegratedTimeline { ptr })
    }
}

unsafe extern "C" fn player_item_integrated_timeline_time_trampoline(
    userdata: *mut c_void,
    value: i64,
    timescale: i32,
    kind: i32,
) {
    unsafe {
        deliver(
            userdata,
            "player_item_integrated_timeline_time_trampoline",
            Time::from_raw(value, timescale, kind),
        );
    }
}

unsafe extern "C" fn player_item_integrated_timeline_out_of_sync_trampoline(
    userdata: *mut c_void,
    payload_json: *const c_char,
) {
    let Some(payload) =
        (unsafe { json_payload::<PlayerIntegratedTimelineOutOfSyncPayload>(payload_json) })
    else {
        return;
    };
    let event = PlayerIntegratedTimelineOutOfSyncEvent {
        reason: PlayerIntegratedTimelineSnapshotsOutOfSyncReason::from_raw(&payload.reason),
    };
    unsafe {
        deliver(
            userdata,
            "player_item_integrated_timeline_out_of_sync_trampoline",
            event,
        );
    }
}
