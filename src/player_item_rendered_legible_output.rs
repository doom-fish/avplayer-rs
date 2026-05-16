#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;

use serde::Deserialize;

use crate::asset::Size;
use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::player::PlayerItem;
use crate::time::Time;
use crate::util::parse_json_and_free;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RenderedLegibleOutputInfoPayload {
    suppresses_player_rendering: bool,
    advance_interval_for_delegate_invocation: f64,
    video_display_size: Size,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RenderedCaptionImagePayload {
    x: f64,
    y: f64,
    width: usize,
    height: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderedCaptionImage {
    pub x: f64,
    pub y: f64,
    pub width: usize,
    pub height: usize,
}

impl From<RenderedCaptionImagePayload> for RenderedCaptionImage {
    fn from(payload: RenderedCaptionImagePayload) -> Self {
        Self {
            x: payload.x,
            y: payload.y,
            width: payload.width,
            height: payload.height,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RenderedLegibleOutputEventPayload {
    event: String,
    item_time: Option<Time>,
    caption_images: Option<Vec<RenderedCaptionImagePayload>>,
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum RenderedLegibleOutputEvent {
    SequenceWasFlushed,
    RenderedCaptionImages {
        item_time: Time,
        caption_images: Vec<RenderedCaptionImage>,
    },
}

struct RenderedLegibleOutputObserverState {
    callback: Box<dyn Fn(RenderedLegibleOutputEvent) + Send + 'static>,
}

pub struct PlayerItemRenderedLegibleOutput {
    pub(crate) ptr: *mut c_void,
}

impl Drop for PlayerItemRenderedLegibleOutput {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_player_item_output_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl PlayerItemRenderedLegibleOutput {
    pub fn new(video_display_size: Size) -> Result<Self, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_player_item_rendered_legible_output_create(
                video_display_size.width,
                video_display_size.height,
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn info(&self) -> Result<RenderedLegibleOutputInfoPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::av_player_item_rendered_legible_output_info_json(self.ptr, &mut err)
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn suppresses_player_rendering(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.suppresses_player_rendering)
    }

    pub fn set_suppresses_player_rendering(&self, suppresses: bool) {
        unsafe { ffi::av_player_item_output_set_suppresses_player_rendering(self.ptr, suppresses) };
    }

    pub fn advance_interval_for_delegate_invocation(&self) -> Result<f64, AVPlayerError> {
        Ok(self.info()?.advance_interval_for_delegate_invocation)
    }

    pub fn set_advance_interval_for_delegate_invocation(&self, interval: f64) {
        unsafe {
            ffi::av_player_item_rendered_legible_output_set_advance_interval(self.ptr, interval);
        }
    }

    pub fn video_display_size(&self) -> Result<Size, AVPlayerError> {
        Ok(self.info()?.video_display_size)
    }

    pub fn set_video_display_size(&self, value: Size) {
        unsafe {
            ffi::av_player_item_rendered_legible_output_set_video_display_size(
                self.ptr,
                value.width,
                value.height,
            );
        }
    }

    pub fn observe<F>(
        &self,
        queue_label: Option<&str>,
        callback: F,
    ) -> Result<RenderedLegibleOutputObserver, AVPlayerError>
    where
        F: Fn(RenderedLegibleOutputEvent) + Send + 'static,
    {
        let queue_label = queue_label
            .map(|label| crate::util::to_cstring(label, "rendered legible output queue label"))
            .transpose()?;
        let state = Box::new(RenderedLegibleOutputObserverState {
            callback: Box::new(callback),
        });
        let userdata = Box::into_raw(state).cast::<c_void>();
        let mut err: *mut c_char = ptr::null_mut();
        let token = unsafe {
            ffi::av_player_item_rendered_legible_output_add_observer(
                self.ptr,
                queue_label.as_ref().map_or(ptr::null(), |label| label.as_ptr()),
                Some(rendered_legible_output_event_trampoline),
                userdata,
                Some(rendered_legible_output_observer_drop),
                &mut err,
            )
        };
        if token.is_null() {
            unsafe { rendered_legible_output_observer_drop(userdata) };
            return Err(unsafe { from_swift(ffi::status::OBSERVER_FAILED, err) });
        }
        Ok(RenderedLegibleOutputObserver { token })
    }
}

pub struct RenderedLegibleOutputObserver {
    token: *mut c_void,
}

impl Drop for RenderedLegibleOutputObserver {
    fn drop(&mut self) {
        if !self.token.is_null() {
            unsafe { ffi::av_player_item_rendered_legible_output_observer_release(self.token) };
            self.token = ptr::null_mut();
        }
    }
}

impl PlayerItem {
    pub fn add_rendered_legible_output(
        &self,
        output: &PlayerItemRenderedLegibleOutput,
    ) -> Result<(), AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe { ffi::av_player_item_add_output(self.ptr, output.ptr, &mut err) };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn remove_rendered_legible_output(&self, output: &PlayerItemRenderedLegibleOutput) {
        unsafe { ffi::av_player_item_remove_output(self.ptr, output.ptr) };
    }
}

unsafe extern "C" fn rendered_legible_output_event_trampoline(
    userdata: *mut c_void,
    payload_json: *const c_char,
) {
    if userdata.is_null() || payload_json.is_null() {
        return;
    }

    let callback = &*userdata.cast::<RenderedLegibleOutputObserverState>();
    let Ok(payload) = core::ffi::CStr::from_ptr(payload_json).to_str() else {
        return;
    };
    let Ok(payload) = serde_json::from_str::<RenderedLegibleOutputEventPayload>(payload) else {
        return;
    };

    let event = match payload.event.as_str() {
        "sequence_was_flushed" => RenderedLegibleOutputEvent::SequenceWasFlushed,
        "rendered_caption_images" => RenderedLegibleOutputEvent::RenderedCaptionImages {
            item_time: match payload.item_time {
                Some(item_time) => item_time,
                None => return,
            },
            caption_images: payload
                .caption_images
                .unwrap_or_default()
                .into_iter()
                .map(RenderedCaptionImage::from)
                .collect(),
        },
        _ => return,
    };

    (callback.callback)(event);
}

unsafe extern "C" fn rendered_legible_output_observer_drop(userdata: *mut c_void) {
    if !userdata.is_null() {
        drop(Box::from_raw(userdata.cast::<RenderedLegibleOutputObserverState>()));
    }
}
