#![allow(
    clippy::derive_partial_eq_without_eq,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::struct_excessive_bools
)]

use core::ffi::{c_char, c_void};
use core::ops::{BitOr, BitOrAssign};
use core::ptr;

use serde::Deserialize;
use serde_json::Value;

use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::player::{Player, PlayerItem};
use crate::time::Time;
use crate::util::{parse_json_and_free, to_cstring};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct PlayerInterstitialEventRestrictions(u64);

impl PlayerInterstitialEventRestrictions {
    pub const NONE: Self = Self(0);
    pub const CONSTRAINS_SEEKING_FORWARD_IN_PRIMARY_CONTENT: Self = Self(1 << 0);
    pub const REQUIRES_PLAYBACK_AT_PREFERRED_RATE_FOR_ADVANCEMENT: Self = Self(1 << 2);
    pub const DEFAULT_POLICY: Self = Self::NONE;

    pub const fn bits(self) -> u64 {
        self.0
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    const fn from_bits(bits: u64) -> Self {
        Self(bits)
    }
}

impl BitOr for PlayerInterstitialEventRestrictions {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for PlayerInterstitialEventRestrictions {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PlayerInterstitialEventCue {
    NoCue,
    JoinCue,
    LeaveCue,
    Unknown(String),
}

impl PlayerInterstitialEventCue {
    fn from_raw(raw: &str) -> Self {
        match raw {
            "no_cue" => Self::NoCue,
            "join_cue" => Self::JoinCue,
            "leave_cue" => Self::LeaveCue,
            other => Self::Unknown(other.to_owned()),
        }
    }

    fn as_raw(&self) -> &str {
        match self {
            Self::NoCue => "no_cue",
            Self::JoinCue => "join_cue",
            Self::LeaveCue => "leave_cue",
            Self::Unknown(raw) => raw,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PlayerInterstitialEventTimelineOccupancy {
    SinglePoint,
    Fill,
    Unknown(i32),
}

impl PlayerInterstitialEventTimelineOccupancy {
    const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::SinglePoint,
            1 => Self::Fill,
            other => Self::Unknown(other),
        }
    }

    const fn raw(self) -> i32 {
        match self {
            Self::SinglePoint => 0,
            Self::Fill => 1,
            Self::Unknown(raw) => raw,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PlayerInterstitialEventAssetListResponseStatus {
    Available,
    Cleared,
    Unavailable,
    Unknown(i32),
}

impl PlayerInterstitialEventAssetListResponseStatus {
    const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::Available,
            1 => Self::Cleared,
            2 => Self::Unavailable,
            other => Self::Unknown(other),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PlayerInterstitialEventSkippableEventState {
    NotSkippable,
    NotYetEligible,
    Eligible,
    NoLongerEligible,
    Unknown(i32),
}

impl PlayerInterstitialEventSkippableEventState {
    const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::NotSkippable,
            1 => Self::NotYetEligible,
            2 => Self::Eligible,
            3 => Self::NoLongerEligible,
            other => Self::Unknown(other),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerInterstitialEventInfoPayload {
    identifier: String,
    time: Time,
    date: Option<String>,
    template_item_count: usize,
    restrictions: u64,
    resumption_offset: Time,
    playout_limit: Time,
    aligns_start_with_primary_segment_boundary: bool,
    aligns_resumption_with_primary_segment_boundary: bool,
    cue: Option<String>,
    will_play_once: bool,
    user_defined_attributes_json: Option<String>,
    asset_list_response_json: Option<String>,
    timeline_occupancy_raw: Option<i32>,
    supplements_primary_content: Option<bool>,
    content_may_vary: Option<bool>,
    has_primary_item: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerInterstitialEventInfo {
    pub identifier: String,
    pub time: Time,
    pub date: Option<String>,
    pub template_item_count: usize,
    pub restrictions: PlayerInterstitialEventRestrictions,
    pub resumption_offset: Time,
    pub playout_limit: Time,
    pub aligns_start_with_primary_segment_boundary: bool,
    pub aligns_resumption_with_primary_segment_boundary: bool,
    pub cue: Option<PlayerInterstitialEventCue>,
    pub will_play_once: bool,
    pub user_defined_attributes: Option<Value>,
    pub asset_list_response: Option<Value>,
    pub timeline_occupancy: Option<PlayerInterstitialEventTimelineOccupancy>,
    pub supplements_primary_content: Option<bool>,
    pub content_may_vary: Option<bool>,
    pub has_primary_item: bool,
}

impl TryFrom<PlayerInterstitialEventInfoPayload> for PlayerInterstitialEventInfo {
    type Error = AVPlayerError;

    fn try_from(payload: PlayerInterstitialEventInfoPayload) -> Result<Self, Self::Error> {
        Ok(Self {
            identifier: payload.identifier,
            time: payload.time,
            date: payload.date,
            template_item_count: payload.template_item_count,
            restrictions: PlayerInterstitialEventRestrictions::from_bits(payload.restrictions),
            resumption_offset: payload.resumption_offset,
            playout_limit: payload.playout_limit,
            aligns_start_with_primary_segment_boundary: payload.aligns_start_with_primary_segment_boundary,
            aligns_resumption_with_primary_segment_boundary: payload.aligns_resumption_with_primary_segment_boundary,
            cue: payload.cue.as_deref().map(PlayerInterstitialEventCue::from_raw),
            will_play_once: payload.will_play_once,
            user_defined_attributes: parse_json_value(payload.user_defined_attributes_json)?,
            asset_list_response: parse_json_value(payload.asset_list_response_json)?,
            timeline_occupancy: payload
                .timeline_occupancy_raw
                .map(PlayerInterstitialEventTimelineOccupancy::from_raw),
            supplements_primary_content: payload.supplements_primary_content,
            content_may_vary: payload.content_may_vary,
            has_primary_item: payload.has_primary_item,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerInterstitialMonitorStatePayload {
    events: Vec<PlayerInterstitialEventInfoPayload>,
    current_event: Option<PlayerInterstitialEventInfoPayload>,
    current_event_skippable_state_raw: Option<i32>,
    current_event_skip_control_label: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerInterstitialEventMonitorState {
    pub events: Vec<PlayerInterstitialEventInfo>,
    pub current_event: Option<PlayerInterstitialEventInfo>,
    pub current_event_skippable_state: Option<PlayerInterstitialEventSkippableEventState>,
    pub current_event_skip_control_label: Option<String>,
}

impl TryFrom<PlayerInterstitialMonitorStatePayload> for PlayerInterstitialEventMonitorState {
    type Error = AVPlayerError;

    fn try_from(payload: PlayerInterstitialMonitorStatePayload) -> Result<Self, Self::Error> {
        Ok(Self {
            events: payload
                .events
                .into_iter()
                .map(PlayerInterstitialEventInfo::try_from)
                .collect::<Result<_, _>>()?,
            current_event: payload.current_event.map(PlayerInterstitialEventInfo::try_from).transpose()?,
            current_event_skippable_state: payload
                .current_event_skippable_state_raw
                .map(PlayerInterstitialEventSkippableEventState::from_raw),
            current_event_skip_control_label: payload.current_event_skip_control_label,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerInterstitialMonitorEventPayload {
    event: String,
    interstitial_event: Option<PlayerInterstitialEventInfoPayload>,
    asset_list_response_status_raw: Option<i32>,
    skippable_state_raw: Option<i32>,
    skip_control_label: Option<String>,
    error_message: Option<String>,
    playout_time: Option<Time>,
    did_play_entire_event: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum PlayerInterstitialEventMonitorEvent {
    EventsDidChange,
    CurrentEventDidChange,
    AssetListResponseStatusDidChange {
        interstitial_event: Option<PlayerInterstitialEventInfo>,
        status: PlayerInterstitialEventAssetListResponseStatus,
        error_message: Option<String>,
    },
    CurrentEventSkippableStateDidChange {
        interstitial_event: Option<PlayerInterstitialEventInfo>,
        state: PlayerInterstitialEventSkippableEventState,
        skip_control_label: Option<String>,
    },
    CurrentEventSkipped {
        interstitial_event: Option<PlayerInterstitialEventInfo>,
    },
    InterstitialEventWasUnscheduled {
        interstitial_event: Option<PlayerInterstitialEventInfo>,
        error_message: Option<String>,
    },
    InterstitialEventDidFinish {
        interstitial_event: Option<PlayerInterstitialEventInfo>,
        playout_time: Option<Time>,
        did_play_entire_event: Option<bool>,
    },
}

struct PlayerInterstitialEventMonitorObserverState {
    callback: Box<dyn Fn(PlayerInterstitialEventMonitorEvent) + Send + 'static>,
}

pub struct PlayerInterstitialEvent {
    pub(crate) ptr: *mut c_void,
}

impl Drop for PlayerInterstitialEvent {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_player_interstitial_event_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl PlayerInterstitialEvent {
    pub fn new(primary_item: &PlayerItem, time: Time) -> Result<Self, AVPlayerError> {
        let (value, timescale, kind) = time.to_raw();
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_player_interstitial_event_create_with_time(
                primary_item.ptr,
                value,
                timescale,
                kind,
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    pub fn info(&self) -> Result<PlayerInterstitialEventInfo, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_player_interstitial_event_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        PlayerInterstitialEventInfo::try_from(parse_json_and_free::<PlayerInterstitialEventInfoPayload>(json_ptr)?)
    }

    pub fn set_identifier(&self, identifier: &str) -> Result<(), AVPlayerError> {
        let identifier = to_cstring(identifier, "interstitial event identifier")?;
        unsafe { ffi::av_player_interstitial_event_set_identifier(self.ptr, identifier.as_ptr()) };
        Ok(())
    }

    pub fn set_restrictions(&self, restrictions: PlayerInterstitialEventRestrictions) {
        unsafe { ffi::av_player_interstitial_event_set_restrictions(self.ptr, restrictions.bits()) };
    }

    pub fn set_resumption_offset(&self, value: Time) {
        let (time_value, timescale, kind) = value.to_raw();
        unsafe {
            ffi::av_player_interstitial_event_set_resumption_offset(
                self.ptr,
                time_value,
                timescale,
                kind,
            );
        }
    }

    pub fn set_playout_limit(&self, value: Time) {
        let (time_value, timescale, kind) = value.to_raw();
        unsafe {
            ffi::av_player_interstitial_event_set_playout_limit(
                self.ptr,
                time_value,
                timescale,
                kind,
            );
        }
    }

    pub fn set_aligns_start_with_primary_segment_boundary(&self, enabled: bool) {
        unsafe {
            ffi::av_player_interstitial_event_set_aligns_start_with_primary_segment_boundary(
                self.ptr, enabled,
            );
        }
    }

    pub fn set_aligns_resumption_with_primary_segment_boundary(&self, enabled: bool) {
        unsafe {
            ffi::av_player_interstitial_event_set_aligns_resumption_with_primary_segment_boundary(
                self.ptr, enabled,
            );
        }
    }

    pub fn set_cue(&self, cue: &PlayerInterstitialEventCue) -> Result<(), AVPlayerError> {
        let cue = to_cstring(cue.as_raw(), "interstitial event cue")?;
        unsafe { ffi::av_player_interstitial_event_set_cue(self.ptr, cue.as_ptr()) };
        Ok(())
    }

    pub fn set_will_play_once(&self, enabled: bool) {
        unsafe { ffi::av_player_interstitial_event_set_will_play_once(self.ptr, enabled) };
    }

    pub fn set_timeline_occupancy(&self, occupancy: PlayerInterstitialEventTimelineOccupancy) {
        unsafe { ffi::av_player_interstitial_event_set_timeline_occupancy(self.ptr, occupancy.raw()) };
    }

    pub fn set_supplements_primary_content(&self, enabled: bool) {
        unsafe { ffi::av_player_interstitial_event_set_supplements_primary_content(self.ptr, enabled) };
    }

    pub fn set_content_may_vary(&self, enabled: bool) {
        unsafe { ffi::av_player_interstitial_event_set_content_may_vary(self.ptr, enabled) };
    }
}

pub struct PlayerInterstitialEventMonitor {
    pub(crate) ptr: *mut c_void,
}

impl Drop for PlayerInterstitialEventMonitor {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_player_interstitial_event_monitor_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl PlayerInterstitialEventMonitor {
    pub fn new(player: &Player) -> Result<Self, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::av_player_interstitial_event_monitor_create(player.ptr, &mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    pub fn state(&self) -> Result<PlayerInterstitialEventMonitorState, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_player_interstitial_event_monitor_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        PlayerInterstitialEventMonitorState::try_from(parse_json_and_free::<PlayerInterstitialMonitorStatePayload>(json_ptr)?)
    }

    pub fn observe<F>(&self, callback: F) -> Result<PlayerInterstitialEventMonitorObserver, AVPlayerError>
    where
        F: Fn(PlayerInterstitialEventMonitorEvent) + Send + 'static,
    {
        let state = Box::new(PlayerInterstitialEventMonitorObserverState {
            callback: Box::new(callback),
        });
        let userdata = Box::into_raw(state).cast::<c_void>();
        let mut err: *mut c_char = ptr::null_mut();
        let token = unsafe {
            ffi::av_player_interstitial_event_monitor_add_observer(
                self.ptr,
                Some(player_interstitial_event_monitor_event_trampoline),
                userdata,
                Some(player_interstitial_event_monitor_observer_drop),
                &mut err,
            )
        };
        if token.is_null() {
            unsafe { player_interstitial_event_monitor_observer_drop(userdata) };
            return Err(unsafe { from_swift(ffi::status::OBSERVER_FAILED, err) });
        }
        Ok(PlayerInterstitialEventMonitorObserver { token })
    }
}

pub struct PlayerInterstitialEventController {
    pub(crate) ptr: *mut c_void,
}

impl Drop for PlayerInterstitialEventController {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_player_interstitial_event_controller_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl PlayerInterstitialEventController {
    pub fn new(player: &Player) -> Result<Self, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::av_player_interstitial_event_controller_create(player.ptr, &mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    pub fn state(&self) -> Result<PlayerInterstitialEventMonitorState, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_player_interstitial_event_controller_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        PlayerInterstitialEventMonitorState::try_from(parse_json_and_free::<PlayerInterstitialMonitorStatePayload>(json_ptr)?)
    }

    pub fn set_events(&self, events: &[&PlayerInterstitialEvent]) -> Result<(), AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let event_ptrs = events.iter().map(|event| event.ptr).collect::<Vec<_>>();
        let status = unsafe {
            ffi::av_player_interstitial_event_controller_set_events(
                self.ptr,
                event_ptrs.as_ptr(),
                event_ptrs.len(),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn cancel_current_event_with_resumption_offset(&self, value: Time) {
        let (time_value, timescale, kind) = value.to_raw();
        unsafe {
            ffi::av_player_interstitial_event_controller_cancel_current_event_with_resumption_offset(
                self.ptr,
                time_value,
                timescale,
                kind,
            );
        }
    }

    pub fn skip_current_event(&self) {
        unsafe { ffi::av_player_interstitial_event_controller_skip_current_event(self.ptr) };
    }
}

pub struct PlayerInterstitialEventMonitorObserver {
    token: *mut c_void,
}

impl Drop for PlayerInterstitialEventMonitorObserver {
    fn drop(&mut self) {
        if !self.token.is_null() {
            unsafe { ffi::av_player_interstitial_event_monitor_observer_release(self.token) };
            self.token = ptr::null_mut();
        }
    }
}

pub fn player_waiting_during_interstitial_event_reason() -> Result<String, AVPlayerError> {
    let mut err: *mut c_char = ptr::null_mut();
    let string_ptr = unsafe { ffi::av_player_waiting_during_interstitial_event_reason(&mut err) };
    if string_ptr.is_null() {
        return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
    }
    parse_json_and_free::<String>(string_ptr)
}

fn parse_json_value(value: Option<String>) -> Result<Option<Value>, AVPlayerError> {
    value
        .map(|value| {
            serde_json::from_str::<Value>(&value).map_err(|error| {
                AVPlayerError::OperationFailed(format!(
                    "failed to decode interstitial event JSON payload: {error}"
                ))
            })
        })
        .transpose()
}

unsafe extern "C" fn player_interstitial_event_monitor_event_trampoline(
    userdata: *mut c_void,
    payload_json: *const c_char,
) {
    if userdata.is_null() || payload_json.is_null() {
        return;
    }

    let callback = &*userdata.cast::<PlayerInterstitialEventMonitorObserverState>();
    let Ok(payload) = core::ffi::CStr::from_ptr(payload_json).to_str() else {
        return;
    };
    let Ok(payload) = serde_json::from_str::<PlayerInterstitialMonitorEventPayload>(payload) else {
        return;
    };

    let interstitial_event = payload
        .interstitial_event
        .map(PlayerInterstitialEventInfo::try_from)
        .transpose()
        .ok()
        .flatten();

    let event = match payload.event.as_str() {
        "events_did_change" => PlayerInterstitialEventMonitorEvent::EventsDidChange,
        "current_event_did_change" => PlayerInterstitialEventMonitorEvent::CurrentEventDidChange,
        "asset_list_response_status_did_change" => {
            PlayerInterstitialEventMonitorEvent::AssetListResponseStatusDidChange {
                interstitial_event,
                status: PlayerInterstitialEventAssetListResponseStatus::from_raw(
                    payload.asset_list_response_status_raw.unwrap_or_default(),
                ),
                error_message: payload.error_message,
            }
        }
        "current_event_skippable_state_did_change" => {
            PlayerInterstitialEventMonitorEvent::CurrentEventSkippableStateDidChange {
                interstitial_event,
                state: PlayerInterstitialEventSkippableEventState::from_raw(
                    payload.skippable_state_raw.unwrap_or_default(),
                ),
                skip_control_label: payload.skip_control_label,
            }
        }
        "current_event_skipped" => {
            PlayerInterstitialEventMonitorEvent::CurrentEventSkipped { interstitial_event }
        }
        "interstitial_event_was_unscheduled" => {
            PlayerInterstitialEventMonitorEvent::InterstitialEventWasUnscheduled {
                interstitial_event,
                error_message: payload.error_message,
            }
        }
        "interstitial_event_did_finish" => {
            PlayerInterstitialEventMonitorEvent::InterstitialEventDidFinish {
                interstitial_event,
                playout_time: payload.playout_time,
                did_play_entire_event: payload.did_play_entire_event,
            }
        }
        _ => return,
    };

    (callback.callback)(event);
}

unsafe extern "C" fn player_interstitial_event_monitor_observer_drop(userdata: *mut c_void) {
    if !userdata.is_null() {
        drop(Box::from_raw(
            userdata.cast::<PlayerInterstitialEventMonitorObserverState>(),
        ));
    }
}
