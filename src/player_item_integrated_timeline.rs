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
use crate::time::{Time, TimeRange};
use crate::util::{parse_json_and_free, to_cstring};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PlayerItemSegmentType {
    Primary,
    Interstitial,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerItemIntegratedTimelineInfo {
    pub current_time: Time,
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

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerItemIntegratedTimelineSegmentInfo {
    pub segment_type: PlayerItemSegmentType,
    pub time_mapping_source: TimeRange,
    pub time_mapping_target: TimeRange,
    pub loaded_time_ranges: Vec<TimeRange>,
    pub start_date: Option<String>,
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

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerItemIntegratedTimelineSnapshotInfo {
    pub duration: Time,
    pub current_time: Time,
    pub current_date: Option<String>,
    pub current_segment_index: Option<usize>,
    pub current_segment: Option<PlayerItemIntegratedTimelineSegmentInfo>,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PlayerIntegratedTimelineSnapshotsOutOfSyncReason {
    SegmentsChanged,
    CurrentSegmentChanged,
    LoadedTimeRangesChanged,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlayerIntegratedTimelineOutOfSyncEvent {
    pub reason: PlayerIntegratedTimelineSnapshotsOutOfSyncReason,
}

struct TimelineTimeObserverState {
    callback: Box<dyn Fn(Time) + Send + 'static>,
}

struct TimelineOutOfSyncObserverState {
    callback: Box<dyn Fn(PlayerIntegratedTimelineOutOfSyncEvent) + Send + 'static>,
}

pub struct PlayerItemIntegratedTimeline {
    pub(crate) ptr: *mut c_void,
}

impl Drop for PlayerItemIntegratedTimeline {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_player_item_integrated_timeline_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl PlayerItemIntegratedTimeline {
    pub fn info(&self) -> Result<PlayerItemIntegratedTimelineInfo, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::av_player_item_integrated_timeline_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(parse_json_and_free::<PlayerItemIntegratedTimelineInfoPayload>(json_ptr)?.into())
    }

    pub fn current_snapshot(&self) -> Result<PlayerItemIntegratedTimelineSnapshot, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_player_item_integrated_timeline_copy_current_snapshot(self.ptr, &mut err)
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(PlayerItemIntegratedTimelineSnapshot { ptr })
    }

    pub fn seek_to_time(
        &self,
        time: Time,
        tolerance_before: Time,
        tolerance_after: Time,
    ) -> Result<bool, AVPlayerError> {
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
                &mut success,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(success)
    }

    pub fn seek_to_date(&self, date: &str) -> Result<bool, AVPlayerError> {
        let date = to_cstring(date, "integrated timeline date")?;
        let mut err: *mut c_char = ptr::null_mut();
        let mut success = false;
        let status = unsafe {
            ffi::av_player_item_integrated_timeline_seek_to_date(
                self.ptr,
                date.as_ptr(),
                &mut success,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(success)
    }

    pub fn observe_periodic_times<F>(
        &self,
        interval: Time,
        callback: F,
    ) -> Result<PlayerItemIntegratedTimelineObserver, AVPlayerError>
    where
        F: Fn(Time) + Send + 'static,
    {
        let state = Box::new(TimelineTimeObserverState {
            callback: Box::new(callback),
        });
        let userdata = Box::into_raw(state).cast::<c_void>();
        let (value, timescale, kind) = interval.to_raw();
        let mut err: *mut c_char = ptr::null_mut();
        let token = unsafe {
            ffi::av_player_item_integrated_timeline_add_periodic_time_observer(
                self.ptr,
                value,
                timescale,
                kind,
                Some(player_item_integrated_timeline_time_trampoline),
                userdata,
                Some(player_item_integrated_timeline_time_observer_drop),
                &mut err,
            )
        };
        if token.is_null() {
            unsafe { player_item_integrated_timeline_time_observer_drop(userdata) };
            return Err(unsafe { from_swift(ffi::status::OBSERVER_FAILED, err) });
        }
        Ok(PlayerItemIntegratedTimelineObserver { token })
    }

    pub fn observe_boundary_times<F>(
        &self,
        segment: &PlayerItemIntegratedTimelineSegment,
        offsets_into_segment: &[Time],
        callback: F,
    ) -> Result<PlayerItemIntegratedTimelineObserver, AVPlayerError>
    where
        F: Fn(Time) + Send + 'static,
    {
        let state = Box::new(TimelineTimeObserverState {
            callback: Box::new(callback),
        });
        let userdata = Box::into_raw(state).cast::<c_void>();
        let offsets_json = serde_json::to_string(offsets_into_segment).map_err(|error| {
            AVPlayerError::InvalidArgument(format!(
                "failed to encode integrated timeline offsets: {error}"
            ))
        })?;
        let offsets_json = to_cstring(&offsets_json, "integrated timeline boundary offsets")?;
        let mut err: *mut c_char = ptr::null_mut();
        let token = unsafe {
            ffi::av_player_item_integrated_timeline_add_boundary_time_observer(
                self.ptr,
                segment.ptr,
                offsets_json.as_ptr(),
                Some(player_item_integrated_timeline_time_trampoline),
                userdata,
                Some(player_item_integrated_timeline_time_observer_drop),
                &mut err,
            )
        };
        if token.is_null() {
            unsafe { player_item_integrated_timeline_time_observer_drop(userdata) };
            return Err(unsafe { from_swift(ffi::status::OBSERVER_FAILED, err) });
        }
        Ok(PlayerItemIntegratedTimelineObserver { token })
    }

    pub fn observe_snapshots_out_of_sync<F>(
        &self,
        callback: F,
    ) -> Result<PlayerItemIntegratedTimelineObserver, AVPlayerError>
    where
        F: Fn(PlayerIntegratedTimelineOutOfSyncEvent) + Send + 'static,
    {
        let state = Box::new(TimelineOutOfSyncObserverState {
            callback: Box::new(callback),
        });
        let userdata = Box::into_raw(state).cast::<c_void>();
        let mut err: *mut c_char = ptr::null_mut();
        let token = unsafe {
            ffi::av_player_item_integrated_timeline_add_out_of_sync_observer(
                self.ptr,
                Some(player_item_integrated_timeline_out_of_sync_trampoline),
                userdata,
                Some(player_item_integrated_timeline_out_of_sync_observer_drop),
                &mut err,
            )
        };
        if token.is_null() {
            unsafe { player_item_integrated_timeline_out_of_sync_observer_drop(userdata) };
            return Err(unsafe { from_swift(ffi::status::OBSERVER_FAILED, err) });
        }
        Ok(PlayerItemIntegratedTimelineObserver { token })
    }
}

pub struct PlayerItemIntegratedTimelineSnapshot {
    pub(crate) ptr: *mut c_void,
}

impl Drop for PlayerItemIntegratedTimelineSnapshot {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_player_item_integrated_timeline_snapshot_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl PlayerItemIntegratedTimelineSnapshot {
    pub fn info(&self) -> Result<PlayerItemIntegratedTimelineSnapshotInfo, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::av_player_item_integrated_timeline_snapshot_info_json(self.ptr, &mut err)
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        PlayerItemIntegratedTimelineSnapshotInfo::try_from(parse_json_and_free::<
            PlayerItemIntegratedTimelineSnapshotPayload,
        >(json_ptr)?)
    }

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

    pub fn segment_count(&self) -> usize {
        unsafe { ffi::av_player_item_integrated_timeline_snapshot_segment_count(self.ptr) }
    }

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

    pub fn segment_and_offset_into_segment(
        &self,
        timeline_time: Time,
    ) -> Result<(PlayerItemIntegratedTimelineSegmentInfo, Time), AVPlayerError> {
        let (value, timescale, kind) = timeline_time.to_raw();
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::av_player_item_integrated_timeline_snapshot_segment_and_offset_json(
                self.ptr, value, timescale, kind, &mut err,
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

pub struct PlayerItemIntegratedTimelineSegment {
    pub(crate) ptr: *mut c_void,
}

impl Drop for PlayerItemIntegratedTimelineSegment {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_player_item_integrated_timeline_segment_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl PlayerItemIntegratedTimelineSegment {
    pub fn info(&self) -> Result<PlayerItemIntegratedTimelineSegmentInfo, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::av_player_item_integrated_timeline_segment_info_json(self.ptr, &mut err)
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        PlayerItemIntegratedTimelineSegmentInfo::try_from(parse_json_and_free::<
            PlayerItemIntegratedTimelineSegmentPayload,
        >(json_ptr)?)
    }
}

pub struct PlayerItemIntegratedTimelineObserver {
    token: *mut c_void,
}

impl Drop for PlayerItemIntegratedTimelineObserver {
    fn drop(&mut self) {
        if !self.token.is_null() {
            unsafe { ffi::av_player_item_integrated_timeline_observer_release(self.token) };
            self.token = ptr::null_mut();
        }
    }
}

// SAFETY: These integrated-timeline wrapper handles are safe to transfer across
// thread boundaries; method calls are internally dispatched safely.
unsafe impl Send for PlayerItemIntegratedTimeline {}
unsafe impl Send for PlayerItemIntegratedTimelineSnapshot {}
unsafe impl Send for PlayerItemIntegratedTimelineSegment {}
unsafe impl Send for PlayerItemIntegratedTimelineObserver {}

pub fn player_integrated_timeline_snapshots_out_of_sync_notification(
) -> Result<String, AVPlayerError> {
    let mut err: *mut c_char = ptr::null_mut();
    let ptr =
        unsafe { ffi::av_player_integrated_timeline_snapshots_out_of_sync_notification(&mut err) };
    if ptr.is_null() {
        return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
    }
    parse_json_and_free::<String>(ptr)
}

pub fn player_integrated_timeline_snapshots_out_of_sync_reason_key() -> Result<String, AVPlayerError>
{
    let mut err: *mut c_char = ptr::null_mut();
    let ptr =
        unsafe { ffi::av_player_integrated_timeline_snapshots_out_of_sync_reason_key(&mut err) };
    if ptr.is_null() {
        return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
    }
    parse_json_and_free::<String>(ptr)
}

pub fn player_integrated_timeline_snapshots_out_of_sync_reason_segments_changed(
) -> Result<String, AVPlayerError> {
    let mut err: *mut c_char = ptr::null_mut();
    let ptr = unsafe {
        ffi::av_player_integrated_timeline_snapshots_out_of_sync_reason_segments_changed(&mut err)
    };
    if ptr.is_null() {
        return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
    }
    parse_json_and_free::<String>(ptr)
}

pub fn player_integrated_timeline_snapshots_out_of_sync_reason_current_segment_changed(
) -> Result<String, AVPlayerError> {
    let mut err: *mut c_char = ptr::null_mut();
    let ptr = unsafe {
        ffi::av_player_integrated_timeline_snapshots_out_of_sync_reason_current_segment_changed(
            &mut err,
        )
    };
    if ptr.is_null() {
        return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
    }
    parse_json_and_free::<String>(ptr)
}

pub fn player_integrated_timeline_snapshots_out_of_sync_reason_loaded_time_ranges_changed(
) -> Result<String, AVPlayerError> {
    let mut err: *mut c_char = ptr::null_mut();
    let ptr = unsafe {
        ffi::av_player_integrated_timeline_snapshots_out_of_sync_reason_loaded_time_ranges_changed(
            &mut err,
        )
    };
    if ptr.is_null() {
        return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
    }
    parse_json_and_free::<String>(ptr)
}

impl PlayerItem {
    pub fn integrated_timeline(&self) -> Result<PlayerItemIntegratedTimeline, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::av_player_item_copy_integrated_timeline(self.ptr, &mut err) };
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
    if userdata.is_null() {
        return;
    }
    let state = &*userdata.cast::<TimelineTimeObserverState>();
    crate::util::catch_cb_panic("player_item_integrated_timeline_time_trampoline", || {
        (state.callback)(Time::from_raw(value, timescale, kind));
    });
}

unsafe extern "C" fn player_item_integrated_timeline_time_observer_drop(userdata: *mut c_void) {
    if !userdata.is_null() {
        drop(Box::from_raw(userdata.cast::<TimelineTimeObserverState>()));
    }
}

unsafe extern "C" fn player_item_integrated_timeline_out_of_sync_trampoline(
    userdata: *mut c_void,
    payload_json: *const c_char,
) {
    if userdata.is_null() || payload_json.is_null() {
        return;
    }
    let state = &*userdata.cast::<TimelineOutOfSyncObserverState>();
    let Ok(payload) = core::ffi::CStr::from_ptr(payload_json).to_str() else {
        return;
    };
    let Ok(payload) = serde_json::from_str::<PlayerIntegratedTimelineOutOfSyncPayload>(payload)
    else {
        return;
    };
    crate::util::catch_cb_panic(
        "player_item_integrated_timeline_out_of_sync_trampoline",
        || {
            (state.callback)(PlayerIntegratedTimelineOutOfSyncEvent {
                reason: PlayerIntegratedTimelineSnapshotsOutOfSyncReason::from_raw(&payload.reason),
            });
        },
    );
}

unsafe extern "C" fn player_item_integrated_timeline_out_of_sync_observer_drop(
    userdata: *mut c_void,
) {
    if !userdata.is_null() {
        drop(Box::from_raw(
            userdata.cast::<TimelineOutOfSyncObserverState>(),
        ));
    }
}
