#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::marker::PhantomData;
use core::ptr;
use std::ffi::CStr;

use apple_cf::cm::CMSampleBuffer;
use apple_cf::cv::CVPixelBuffer;
use doom_fish_utils::stream::{BoundedAsyncStream, NextItem};
use serde::Deserialize;

use crate::asset::{AssetTrack, MediaType};
use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::metadata_groups::TimedMetadataGroupHandle;
use crate::reader::{AssetReader, AssetReaderTrackOutput};
use crate::time::TimeRange;
use crate::util::{catch_cb_panic, json_cstring, parse_json_and_free};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptionGroupInfo {
    pub time_range: TimeRange,
    pub captions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptionValidationEvent {
    pub caption_text: String,
    pub syntax_elements: Vec<String>,
}

struct CaptionValidationObserverState {
    callback: Box<dyn Fn(CaptionValidationEvent) + Send + 'static>,
}

/// Borrowed `AVAssetReaderOutput` view.
#[derive(Debug, Clone, Copy)]
pub struct AssetReaderOutput<'a> {
    ptr: *mut c_void,
    _marker: PhantomData<&'a c_void>,
}

impl AssetReaderOutput<'_> {
    pub(crate) const fn from_ptr(ptr: *mut c_void) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    pub fn media_type(&self) -> Result<MediaType, AVPlayerError> {
        let raw = unsafe { ffi::av_reader_output_media_type(self.ptr) };
        if raw.is_null() {
            return Err(AVPlayerError::OperationFailed(
                "reader output did not return a media type".into(),
            ));
        }
        let media_type = unsafe { CStr::from_ptr(raw) }
            .to_string_lossy()
            .into_owned();
        unsafe { ffi::avp_string_free(raw) };
        Ok(MediaType::from_raw(&media_type))
    }

    pub fn set_always_copies_sample_data(&self, always_copies: bool) {
        unsafe { ffi::av_reader_output_set_always_copies_sample_data(self.ptr, always_copies) };
    }

    pub fn supports_random_access(&self) -> bool {
        unsafe { ffi::av_reader_output_supports_random_access(self.ptr) }
    }

    pub fn set_supports_random_access(&self, supports_random_access: bool) {
        unsafe {
            ffi::av_reader_output_set_supports_random_access(self.ptr, supports_random_access);
        };
    }

    pub fn reset_for_reading_time_ranges(
        &self,
        time_ranges: &[TimeRange],
    ) -> Result<(), AVPlayerError> {
        let time_ranges = json_cstring(time_ranges, "reader output time ranges")?;
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::av_reader_output_reset_for_time_ranges_json(
                self.ptr,
                time_ranges.as_ptr(),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn mark_configuration_as_final(&self) {
        unsafe { ffi::av_reader_output_mark_configuration_as_final(self.ptr) };
    }

    pub fn copy_next_sample_buffer(&self) -> Option<CMSampleBuffer> {
        let ptr = unsafe { ffi::av_reader_output_copy_next_sample_buffer(self.ptr) };
        CMSampleBuffer::from_raw(ptr)
    }

    pub fn copy_next_video_pixel_buffer(&self) -> Option<CVPixelBuffer> {
        let ptr = unsafe { ffi::av_reader_output_copy_next_video_pixel_buffer(self.ptr) };
        CVPixelBuffer::from_raw(ptr)
    }
}

/// Mirrors `AVAssetReaderSampleReferenceOutput`.
#[derive(Debug)]
pub struct AssetReaderSampleReferenceOutput {
    pub(crate) ptr: *mut c_void,
}

impl Drop for AssetReaderSampleReferenceOutput {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_reader_output_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl AssetReaderSampleReferenceOutput {
    pub fn new(track: &AssetTrack) -> Result<Self, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::av_reader_sample_reference_output_create(track.ptr, &mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    pub const fn as_output(&self) -> AssetReaderOutput<'_> {
        AssetReaderOutput::from_ptr(self.ptr)
    }

    pub fn track(&self) -> Option<AssetTrack> {
        let ptr = unsafe { ffi::av_reader_sample_reference_output_copy_track(self.ptr) };
        (!ptr.is_null()).then_some(AssetTrack { ptr })
    }
}

/// Mirrors `AVAssetReaderOutputMetadataAdaptor`.
#[derive(Debug)]
pub struct AssetReaderOutputMetadataAdaptor {
    ptr: *mut c_void,
}

impl Drop for AssetReaderOutputMetadataAdaptor {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl AssetReaderOutputMetadataAdaptor {
    pub fn new(track_output: &AssetReaderTrackOutput) -> Result<Self, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr =
            unsafe { ffi::av_reader_output_metadata_adaptor_create(track_output.ptr, &mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    pub fn track_output(&self) -> Option<AssetReaderTrackOutput> {
        let ptr = unsafe { ffi::av_reader_output_metadata_adaptor_copy_track_output(self.ptr) };
        (!ptr.is_null()).then_some(AssetReaderTrackOutput { ptr })
    }

    pub fn next_timed_metadata_group(
        &self,
    ) -> Result<Option<TimedMetadataGroupHandle>, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_reader_output_metadata_adaptor_copy_next_timed_metadata_group(
                self.ptr, &mut err,
            )
        };
        if ptr.is_null() {
            if err.is_null() {
                return Ok(None);
            }
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Some(TimedMetadataGroupHandle::from_ptr(ptr)))
    }
}

/// Mirrors `AVAssetReaderOutputCaptionAdaptor`.
#[derive(Debug)]
pub struct AssetReaderOutputCaptionAdaptor {
    ptr: *mut c_void,
}

impl Drop for AssetReaderOutputCaptionAdaptor {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl AssetReaderOutputCaptionAdaptor {
    pub fn new(track_output: &AssetReaderTrackOutput) -> Result<Self, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr =
            unsafe { ffi::av_reader_output_caption_adaptor_create(track_output.ptr, &mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    pub fn track_output(&self) -> Option<AssetReaderTrackOutput> {
        let ptr = unsafe { ffi::av_reader_output_caption_adaptor_copy_track_output(self.ptr) };
        (!ptr.is_null()).then_some(AssetReaderTrackOutput { ptr })
    }

    pub fn next_caption_group(&self) -> Result<Option<CaptionGroupInfo>, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::av_reader_output_caption_adaptor_next_caption_group_json(self.ptr, &mut err)
        };
        if json_ptr.is_null() {
            if err.is_null() {
                return Ok(None);
            }
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr).map(Some)
    }

    pub fn observe_validation<F>(
        &self,
        callback: F,
    ) -> Result<CaptionValidationObserver, AVPlayerError>
    where
        F: Fn(CaptionValidationEvent) + Send + 'static,
    {
        let state = Box::new(CaptionValidationObserverState {
            callback: Box::new(callback),
        });
        let userdata = Box::into_raw(state).cast::<c_void>();
        let mut err: *mut c_char = ptr::null_mut();
        let token = unsafe {
            ffi::av_reader_output_caption_adaptor_add_validation_observer(
                self.ptr,
                Some(caption_validation_event_trampoline),
                userdata,
                Some(caption_validation_observer_drop),
                &mut err,
            )
        };
        if token.is_null() {
            unsafe { caption_validation_observer_drop(userdata) };
            return Err(unsafe { from_swift(ffi::status::OBSERVER_FAILED, err) });
        }
        Ok(CaptionValidationObserver { token })
    }

    /// Returns an async stream of caption-validation callbacks.
    pub fn observe_validation_events(
        &self,
        capacity: usize,
    ) -> Result<CaptionValidationEventStream, AVPlayerError> {
        let (inner, sender) = BoundedAsyncStream::new(capacity);
        let observer = self.observe_validation(move |event| {
            sender.push(event);
        })?;
        Ok(CaptionValidationEventStream {
            inner,
            _observer: observer,
        })
    }

    /// Returns a caption-validation stream.
    pub fn validation_event_stream(
        &self,
        capacity: usize,
    ) -> Result<CaptionValidationEventStream, AVPlayerError> {
        self.observe_validation_events(capacity)
    }
}

#[derive(Debug)]
pub struct CaptionValidationObserver {
    token: *mut c_void,
}

#[derive(Debug)]
/// Async stream of validation callbacks sourced from `AVAssetReaderOutputCaptionAdaptor`.
pub struct CaptionValidationEventStream {
    inner: BoundedAsyncStream<CaptionValidationEvent>,
    _observer: CaptionValidationObserver,
}

impl CaptionValidationEventStream {
    #[must_use]
    /// Returns the next buffered validation event.
    pub const fn next(&self) -> NextItem<'_, CaptionValidationEvent> {
        self.inner.next()
    }

    #[must_use]
    /// Returns the next buffered validation event if one is available.
    pub fn try_next(&self) -> Option<CaptionValidationEvent> {
        self.inner.try_next()
    }

    #[must_use]
    /// Returns the number of currently buffered validation events.
    pub fn buffered_count(&self) -> usize {
        self.inner.buffered_count()
    }

    /// Drops all currently buffered validation events without closing the stream.
    pub fn clear_buffer(&self) {
        self.inner.clear_buffer();
    }

    #[must_use]
    /// Returns whether the stream has been closed.
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }
}

impl Drop for CaptionValidationObserver {
    fn drop(&mut self) {
        if !self.token.is_null() {
            unsafe { ffi::av_reader_output_caption_validation_observer_release(self.token) };
            self.token = ptr::null_mut();
        }
    }
}

unsafe impl Send for AssetReaderSampleReferenceOutput {}
unsafe impl Send for AssetReaderOutputMetadataAdaptor {}
unsafe impl Send for AssetReaderOutputCaptionAdaptor {}
unsafe impl Send for CaptionValidationObserver {}

impl AssetReader {
    pub fn can_add_sample_reference_output(
        &self,
        output: &AssetReaderSampleReferenceOutput,
    ) -> bool {
        unsafe { ffi::av_reader_can_add_output(self.ptr, output.ptr) }
    }

    pub fn add_sample_reference_output(
        &self,
        output: &AssetReaderSampleReferenceOutput,
    ) -> Result<(), AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe { ffi::av_reader_add_output(self.ptr, output.ptr, &mut err) };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }
}

impl AssetReaderTrackOutput {
    pub const fn as_output(&self) -> AssetReaderOutput<'_> {
        AssetReaderOutput::from_ptr(self.ptr)
    }
}

unsafe extern "C" fn caption_validation_event_trampoline(
    userdata: *mut c_void,
    payload_json: *const c_char,
) {
    if userdata.is_null() || payload_json.is_null() {
        return;
    }

    let callback = unsafe { &*userdata.cast::<CaptionValidationObserverState>() };
    let Ok(payload) = unsafe { CStr::from_ptr(payload_json) }.to_str() else {
        return;
    };
    let Ok(payload) = serde_json::from_str::<CaptionValidationEvent>(payload) else {
        return;
    };

    catch_cb_panic("caption_validation_event_trampoline", || {
        (callback.callback)(payload);
    });
}

unsafe extern "C" fn caption_validation_observer_drop(userdata: *mut c_void) {
    if !userdata.is_null() {
        unsafe {
            drop(Box::from_raw(
                userdata.cast::<CaptionValidationObserverState>(),
            ));
        };
    }
}
