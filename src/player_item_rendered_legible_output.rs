#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;

use apple_cf::cv::CVPixelBuffer;
use serde::Deserialize;

use crate::asset::Size;
use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::player::PlayerItem;
use crate::retained::retain_release_wrapper;
use crate::time::Time;
use crate::util::{
    borrowed_objects, deliver, json_payload, parse_json_and_free, Handler, Registration,
};

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

/// Mirrors the `AVPlayer` framework counterpart for `RenderedCaptionImage`.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderedCaptionImage {
    /// Mirrors the `AVPlayer` framework property for `x`.
    pub x: f64,
    /// Mirrors the `AVPlayer` framework property for `y`.
    pub y: f64,
    /// Mirrors the `AVPlayer` framework property for `width`.
    pub width: usize,
    /// Mirrors the `AVPlayer` framework property for `height`.
    pub height: usize,
    pub pixel_buffer: CVPixelBuffer,
}

impl RenderedCaptionImage {
    const fn from_payload(
        payload: &RenderedCaptionImagePayload,
        pixel_buffer: CVPixelBuffer,
    ) -> Self {
        Self {
            x: payload.x,
            y: payload.y,
            width: payload.width,
            height: payload.height,
            pixel_buffer,
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

/// Mirrors the `AVPlayer` framework counterpart for `RenderedLegibleOutputEvent`.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum RenderedLegibleOutputEvent {
    /// Mirrors the `AVPlayer` framework case `SequenceWasFlushed`.
    SequenceWasFlushed,
    /// Mirrors the `AVPlayer` framework case `RenderedCaptionImages`.
    RenderedCaptionImages {
        item_time: Time,
        caption_images: Vec<RenderedCaptionImage>,
    },
}

/// Mirrors the `AVPlayer` framework counterpart for `PlayerItemRenderedLegibleOutput`.
#[derive(Debug)]
pub struct PlayerItemRenderedLegibleOutput {
    pub(crate) ptr: *mut c_void,
}

retain_release_wrapper!(
    PlayerItemRenderedLegibleOutput,
    release = ffi::av_player_item_output_release
);

impl PlayerItemRenderedLegibleOutput {
    /// Calls the `AVPlayer` framework counterpart for `new`.
    pub fn new(video_display_size: Size) -> Result<Self, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_player_item_rendered_legible_output_create(
                video_display_size.width,
                video_display_size.height,
                &raw mut err,
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
            ffi::av_player_item_rendered_legible_output_info_json(self.ptr, &raw mut err)
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Calls the `AVPlayer` framework counterpart for `suppresses_player_rendering`.
    pub fn suppresses_player_rendering(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.suppresses_player_rendering)
    }

    /// Calls the `AVPlayer` framework counterpart for `set_suppresses_player_rendering`.
    pub fn set_suppresses_player_rendering(&self, suppresses: bool) {
        unsafe { ffi::av_player_item_output_set_suppresses_player_rendering(self.ptr, suppresses) };
    }

    /// Calls the `AVPlayer` framework counterpart for `advance_interval_for_delegate_invocation`.
    pub fn advance_interval_for_delegate_invocation(&self) -> Result<f64, AVPlayerError> {
        Ok(self.info()?.advance_interval_for_delegate_invocation)
    }

    /// Calls the `AVPlayer` framework counterpart for `set_advance_interval_for_delegate_invocation`.
    pub fn set_advance_interval_for_delegate_invocation(&self, interval: f64) {
        unsafe {
            ffi::av_player_item_rendered_legible_output_set_advance_interval(self.ptr, interval);
        }
    }

    /// Calls the `AVPlayer` framework counterpart for `video_display_size`.
    pub fn video_display_size(&self) -> Result<Size, AVPlayerError> {
        Ok(self.info()?.video_display_size)
    }

    /// Calls the `AVPlayer` framework counterpart for `set_video_display_size`.
    pub fn set_video_display_size(&self, value: Size) {
        unsafe {
            ffi::av_player_item_rendered_legible_output_set_video_display_size(
                self.ptr,
                value.width,
                value.height,
            );
        }
    }

    /// Calls the `AVPlayer` framework counterpart for `observe`.
    pub fn observe<F>(
        &self,
        queue_label: Option<&str>,
        callback: F,
    ) -> Result<RenderedLegibleOutputObserver, AVPlayerError>
    where
        F: Fn(RenderedLegibleOutputEvent) + Send + Sync + 'static,
    {
        let queue_label = queue_label
            .map(|label| crate::util::to_cstring(label, "rendered legible output queue label"))
            .transpose()?;
        let handler: Handler<RenderedLegibleOutputEvent> = Box::new(callback);
        let inner = Registration::new(
            handler,
            ffi::av_player_item_rendered_legible_output_observer_release,
            |userdata, drop_userdata, err| unsafe {
                ffi::av_player_item_rendered_legible_output_add_observer(
                    self.ptr,
                    queue_label
                        .as_ref()
                        .map_or(ptr::null(), |label| label.as_ptr()),
                    Some(rendered_legible_output_event_trampoline),
                    userdata,
                    drop_userdata,
                    err,
                )
            },
        )?;
        Ok(RenderedLegibleOutputObserver { _inner: inner })
    }
}

/// Mirrors the `AVPlayer` framework counterpart for `RenderedLegibleOutputObserver`.
#[derive(Debug)]
pub struct RenderedLegibleOutputObserver {
    _inner: Registration,
}

// SAFETY: These rendered-legible-output handles are safe to transfer across
// thread boundaries; method calls are internally dispatched safely.
unsafe impl Send for PlayerItemRenderedLegibleOutput {}
unsafe impl Send for RenderedLegibleOutputObserver {}

impl PlayerItem {
    /// Calls the `AVPlayer` framework counterpart for `add_rendered_legible_output`.
    pub fn add_rendered_legible_output(
        &self,
        output: &PlayerItemRenderedLegibleOutput,
    ) -> Result<(), AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe { ffi::av_player_item_add_output(self.ptr, output.ptr, &raw mut err) };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Calls the `AVPlayer` framework counterpart for `remove_rendered_legible_output`.
    pub fn remove_rendered_legible_output(&self, output: &PlayerItemRenderedLegibleOutput) {
        unsafe { ffi::av_player_item_remove_output(self.ptr, output.ptr) };
    }
}

unsafe extern "C" fn rendered_legible_output_event_trampoline(
    userdata: *mut c_void,
    payload_json: *const c_char,
    objects: *const *mut c_void,
    count: usize,
) {
    let Some(payload) =
        (unsafe { json_payload::<RenderedLegibleOutputEventPayload>(payload_json) })
    else {
        return;
    };

    let event = match payload.event.as_str() {
        "sequence_was_flushed" => RenderedLegibleOutputEvent::SequenceWasFlushed,
        "rendered_caption_images" => {
            let Some(item_time) = payload.item_time else {
                return;
            };
            let pixel_buffers = unsafe {
                borrowed_objects(objects, count, |object| {
                    CVPixelBuffer::from_raw_borrowed(object)
                })
            };
            let caption_images = payload
                .caption_images
                .unwrap_or_default()
                .into_iter()
                .zip(pixel_buffers)
                .map(|(image, pixel_buffer)| {
                    RenderedCaptionImage::from_payload(&image, pixel_buffer)
                })
                .collect();
            RenderedLegibleOutputEvent::RenderedCaptionImages {
                item_time,
                caption_images,
            }
        }
        _ => return,
    };

    unsafe { deliver(userdata, "rendered_legible_output_event_trampoline", event) };
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;
    use std::sync::{Arc, Mutex};

    use doom_fish_utils::callback_context::CallbackContext;

    use super::*;

    const BGRA: u32 = u32::from_be_bytes(*b"BGRA");

    #[test]
    fn caption_images_carry_their_pixel_buffers() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&events);
        let handler: Handler<RenderedLegibleOutputEvent> =
            Box::new(move |event| sink.lock().unwrap().push(event));
        let context = CallbackContext::new(handler);
        let pixel_buffer = CVPixelBuffer::create(16, 8, BGRA).unwrap();
        let payload = CString::new(
            r#"{"event":"rendered_caption_images","itemTime":{"kind":"numeric","value":3,"timescale":1},"captionImages":[{"x":1.0,"y":2.0,"width":16,"height":8}]}"#,
        )
        .unwrap();
        let objects = [pixel_buffer.as_ptr()];

        unsafe {
            rendered_legible_output_event_trampoline(
                context.as_ptr(),
                payload.as_ptr(),
                objects.as_ptr(),
                objects.len(),
            );
        }

        let events = std::mem::take(&mut *events.lock().unwrap());
        let [RenderedLegibleOutputEvent::RenderedCaptionImages {
            item_time,
            caption_images,
        }] = events.as_slice()
        else {
            panic!("unexpected events: {events:?}");
        };
        assert_eq!(*item_time, Time::new(3, 1));
        assert_eq!(caption_images.len(), 1);
        assert_eq!(caption_images[0].pixel_buffer, pixel_buffer);
        assert_eq!(caption_images[0].pixel_buffer.width(), 16);
        assert_eq!(caption_images[0].pixel_buffer.height(), 8);
        assert!((caption_images[0].x - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn inactive_contexts_receive_nothing() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&events);
        let handler: Handler<RenderedLegibleOutputEvent> =
            Box::new(move |event| sink.lock().unwrap().push(event));
        let context = CallbackContext::new(handler);
        context.deactivate();
        let payload = CString::new(r#"{"event":"sequence_was_flushed"}"#).unwrap();
        unsafe {
            rendered_legible_output_event_trampoline(
                context.as_ptr(),
                payload.as_ptr(),
                core::ptr::null(),
                0,
            );
        }
        assert!(events.lock().unwrap().is_empty());
    }
}
