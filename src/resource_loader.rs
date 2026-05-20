#![allow(
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::struct_excessive_bools
)]

use core::ffi::{c_char, c_void};
use core::ptr;
use std::collections::BTreeMap;
use std::ffi::{CStr, CString};

use doom_fish_utils::stream::{BoundedAsyncStream, NextItem};
use serde::Deserialize;

use crate::asset::UrlAsset;
use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::util::{catch_cb_panic, parse_json_and_free, to_cstring};

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssetResourceLoaderInfoPayload {
    has_delegate: bool,
    preloads_eligible_content_keys: bool,
    sends_common_media_client_data_as_http_headers: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssetResourceLoadingRequestPayload {
    request_url: Option<String>,
    request_method: Option<String>,
    request_headers: Option<BTreeMap<String, String>>,
    finished: bool,
    cancelled: bool,
    has_content_information_request: bool,
    has_data_request: bool,
    has_requestor: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssetResourceLoadingContentInformationRequestPayload {
    content_type: Option<String>,
    allowed_content_types: Option<Vec<String>>,
    content_length: i64,
    byte_range_access_supported: bool,
    renewal_date_iso8601: Option<String>,
    entire_length_available_on_demand: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssetResourceLoadingDataRequestPayload {
    requested_offset: i64,
    requested_length: i64,
    requests_all_data_to_end_of_resource: Option<bool>,
    current_offset: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssetResourceLoadingRequestorPayload {
    provides_expired_session_reports: bool,
}

#[derive(Debug)]
pub struct AssetResourceLoader {
    ptr: *mut c_void,
}

impl Drop for AssetResourceLoader {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

#[derive(Debug)]
pub struct AssetResourceLoadingRequest {
    ptr: *mut c_void,
}

impl Drop for AssetResourceLoadingRequest {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

#[derive(Debug)]
pub struct AssetResourceRenewalRequest {
    ptr: *mut c_void,
}

impl Drop for AssetResourceRenewalRequest {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

#[derive(Debug)]
pub struct AssetResourceLoadingContentInformationRequest {
    ptr: *mut c_void,
}

impl Drop for AssetResourceLoadingContentInformationRequest {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

#[derive(Debug)]
pub struct AssetResourceLoadingDataRequest {
    ptr: *mut c_void,
}

impl Drop for AssetResourceLoadingDataRequest {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

#[derive(Debug)]
pub struct AssetResourceLoadingRequestor {
    ptr: *mut c_void,
}

impl Drop for AssetResourceLoadingRequestor {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

#[derive(Debug)]
pub enum AssetResourceLoaderEvent {
    LoadingRequested(AssetResourceLoadingRequest),
    RenewalRequested(AssetResourceRenewalRequest),
    LoadingCancelled(AssetResourceLoadingRequest),
}

struct AssetResourceLoaderObserverState {
    callback: Box<dyn Fn(AssetResourceLoaderEvent) -> bool + Send + 'static>,
}

#[derive(Debug)]
pub struct AssetResourceLoaderObserver {
    token: *mut c_void,
}

impl Drop for AssetResourceLoaderObserver {
    fn drop(&mut self) {
        if !self.token.is_null() {
            unsafe { ffi::av_asset_resource_loader_delegate_release(self.token) };
            self.token = ptr::null_mut();
        }
    }
}

#[derive(Debug)]
/// Async stream of delegate events sourced from `AVAssetResourceLoader`.
pub struct AssetResourceLoaderEventStream {
    inner: BoundedAsyncStream<AssetResourceLoaderEvent>,
    _observer: AssetResourceLoaderObserver,
}

impl AssetResourceLoaderEventStream {
    #[must_use]
    /// Returns the next buffered loader event.
    pub const fn next(&self) -> NextItem<'_, AssetResourceLoaderEvent> {
        self.inner.next()
    }

    #[must_use]
    /// Returns the next buffered loader event if one is available.
    pub fn try_next(&self) -> Option<AssetResourceLoaderEvent> {
        self.inner.try_next()
    }

    #[must_use]
    /// Returns the number of currently buffered loader events.
    pub fn buffered_count(&self) -> usize {
        self.inner.buffered_count()
    }

    /// Drops all currently buffered loader events without closing the stream.
    pub fn clear_buffer(&self) {
        self.inner.clear_buffer();
    }

    #[must_use]
    /// Returns whether the stream has been closed.
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }
}

impl UrlAsset {
    pub fn resource_loader(&self) -> AssetResourceLoader {
        let ptr = unsafe { ffi::av_url_asset_copy_resource_loader(self.asset.ptr) };
        AssetResourceLoader { ptr }
    }
}

impl AssetResourceLoader {
    fn info(&self) -> Result<AssetResourceLoaderInfoPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_asset_resource_loader_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn has_delegate(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.has_delegate)
    }

    pub fn preloads_eligible_content_keys(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.preloads_eligible_content_keys)
    }

    pub fn set_preloads_eligible_content_keys(&self, enabled: bool) {
        unsafe {
            ffi::av_asset_resource_loader_set_preloads_eligible_content_keys(self.ptr, enabled);
        }
    }

    pub fn sends_common_media_client_data_as_http_headers(
        &self,
    ) -> Result<Option<bool>, AVPlayerError> {
        Ok(self.info()?.sends_common_media_client_data_as_http_headers)
    }

    pub fn set_sends_common_media_client_data_as_http_headers(
        &self,
        enabled: bool,
    ) -> Result<(), AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::av_asset_resource_loader_set_sends_common_media_client_data_as_http_headers(
                self.ptr, enabled, &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn observe_loading_requests<F>(
        &self,
        queue_label: Option<&str>,
        callback: F,
    ) -> Result<AssetResourceLoaderObserver, AVPlayerError>
    where
        F: Fn(AssetResourceLoaderEvent) -> bool + Send + 'static,
    {
        let queue_label = queue_label
            .map(|label| to_cstring(label, "resource-loader queue label"))
            .transpose()?;
        let state = Box::new(AssetResourceLoaderObserverState {
            callback: Box::new(callback),
        });
        let userdata = Box::into_raw(state).cast::<c_void>();
        let mut err: *mut c_char = ptr::null_mut();
        let token = unsafe {
            ffi::av_asset_resource_loader_add_delegate(
                self.ptr,
                queue_label
                    .as_ref()
                    .map_or(ptr::null(), |label| label.as_ptr()),
                Some(asset_resource_loader_event_trampoline),
                userdata,
                Some(asset_resource_loader_observer_drop),
                &mut err,
            )
        };
        if token.is_null() {
            unsafe { asset_resource_loader_observer_drop(userdata) };
            return Err(unsafe { from_swift(ffi::status::OBSERVER_FAILED, err) });
        }
        Ok(AssetResourceLoaderObserver { token })
    }

    /// Returns an async stream of loading-request delegate callbacks.
    ///
    /// The underlying delegate returns `true` for loading and renewal request
    /// events so callers can complete those requests asynchronously.
    pub fn observe_loading_request_events(
        &self,
        queue_label: Option<&str>,
        capacity: usize,
    ) -> Result<AssetResourceLoaderEventStream, AVPlayerError> {
        let (inner, sender) = BoundedAsyncStream::new(capacity);
        let observer = self.observe_loading_requests(queue_label, move |event| {
            let should_wait = matches!(
                &event,
                AssetResourceLoaderEvent::LoadingRequested(_)
                    | AssetResourceLoaderEvent::RenewalRequested(_)
            );
            sender.push(event);
            should_wait
        })?;
        Ok(AssetResourceLoaderEventStream {
            inner,
            _observer: observer,
        })
    }

    /// Returns an async loading-request stream on the default resource-loader queue label.
    pub fn loading_request_stream(
        &self,
        capacity: usize,
    ) -> Result<AssetResourceLoaderEventStream, AVPlayerError> {
        self.observe_loading_request_events(Some("avplayer.resource-loader"), capacity)
    }
}

impl AssetResourceLoadingRequest {
    fn info(&self) -> Result<AssetResourceLoadingRequestPayload, AVPlayerError> {
        resource_loading_request_info(self.ptr)
    }

    pub fn request_url(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.request_url)
    }

    pub fn request_method(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.request_method)
    }

    pub fn request_headers(&self) -> Result<Option<BTreeMap<String, String>>, AVPlayerError> {
        Ok(self.info()?.request_headers)
    }

    pub fn is_finished(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.finished)
    }

    pub fn is_cancelled(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.cancelled)
    }

    pub fn has_content_information_request(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.has_content_information_request)
    }

    pub fn has_data_request(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.has_data_request)
    }

    pub fn has_requestor(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.has_requestor)
    }

    pub fn content_information_request(
        &self,
    ) -> Option<AssetResourceLoadingContentInformationRequest> {
        copy_content_information_request(self.ptr)
    }

    pub fn data_request(&self) -> Option<AssetResourceLoadingDataRequest> {
        copy_data_request(self.ptr)
    }

    pub fn requestor(&self) -> Option<AssetResourceLoadingRequestor> {
        copy_requestor(self.ptr)
    }

    pub fn finish_loading(&self) {
        unsafe { ffi::av_asset_resource_loading_request_finish_loading(self.ptr) };
    }

    pub fn finish_loading_with_error(&self, message: impl AsRef<str>) -> Result<(), AVPlayerError> {
        let message = CString::new(message.as_ref()).map_err(|error| {
            AVPlayerError::InvalidArgument(format!(
                "resource-loading error message contains NUL byte: {error}"
            ))
        })?;
        unsafe {
            ffi::av_asset_resource_loading_request_finish_loading_with_error(
                self.ptr,
                message.as_ptr(),
            );
        }
        Ok(())
    }
}

impl AssetResourceRenewalRequest {
    fn info(&self) -> Result<AssetResourceLoadingRequestPayload, AVPlayerError> {
        resource_loading_request_info(self.ptr)
    }

    pub fn request_url(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.request_url)
    }

    pub fn request_method(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.request_method)
    }

    pub fn request_headers(&self) -> Result<Option<BTreeMap<String, String>>, AVPlayerError> {
        Ok(self.info()?.request_headers)
    }

    pub fn is_finished(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.finished)
    }

    pub fn is_cancelled(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.cancelled)
    }

    pub fn content_information_request(
        &self,
    ) -> Option<AssetResourceLoadingContentInformationRequest> {
        copy_content_information_request(self.ptr)
    }

    pub fn data_request(&self) -> Option<AssetResourceLoadingDataRequest> {
        copy_data_request(self.ptr)
    }

    pub fn requestor(&self) -> Option<AssetResourceLoadingRequestor> {
        copy_requestor(self.ptr)
    }

    pub fn finish_loading(&self) {
        unsafe { ffi::av_asset_resource_loading_request_finish_loading(self.ptr) };
    }

    pub fn finish_loading_with_error(&self, message: impl AsRef<str>) -> Result<(), AVPlayerError> {
        let message = CString::new(message.as_ref()).map_err(|error| {
            AVPlayerError::InvalidArgument(format!(
                "resource-renewal error message contains NUL byte: {error}"
            ))
        })?;
        unsafe {
            ffi::av_asset_resource_loading_request_finish_loading_with_error(
                self.ptr,
                message.as_ptr(),
            );
        }
        Ok(())
    }
}

impl AssetResourceLoadingContentInformationRequest {
    fn info(&self) -> Result<AssetResourceLoadingContentInformationRequestPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::av_asset_resource_loading_content_information_request_info_json(self.ptr, &mut err)
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn content_type(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.content_type)
    }

    pub fn allowed_content_types(&self) -> Result<Option<Vec<String>>, AVPlayerError> {
        Ok(self.info()?.allowed_content_types)
    }

    pub fn content_length(&self) -> Result<i64, AVPlayerError> {
        Ok(self.info()?.content_length)
    }

    pub fn is_byte_range_access_supported(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.byte_range_access_supported)
    }

    pub fn renewal_date_iso8601(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.renewal_date_iso8601)
    }

    pub fn is_entire_length_available_on_demand(&self) -> Result<Option<bool>, AVPlayerError> {
        Ok(self.info()?.entire_length_available_on_demand)
    }

    pub fn set_content_type(&self, content_type: Option<&str>) -> Result<(), AVPlayerError> {
        let content_type = content_type
            .map(|value| to_cstring(value, "resource content type"))
            .transpose()?;
        unsafe {
            ffi::av_asset_resource_loading_content_information_request_set_content_type(
                self.ptr,
                content_type
                    .as_ref()
                    .map_or(ptr::null(), |value| value.as_ptr()),
            );
        }
        Ok(())
    }

    pub fn set_content_length(&self, content_length: i64) {
        unsafe {
            ffi::av_asset_resource_loading_content_information_request_set_content_length(
                self.ptr,
                content_length,
            );
        }
    }

    pub fn set_byte_range_access_supported(&self, supported: bool) {
        unsafe {
            ffi::av_asset_resource_loading_content_information_request_set_byte_range_access_supported(
                self.ptr, supported,
            );
        }
    }

    pub fn set_renewal_date_iso8601(&self, value: Option<&str>) -> Result<(), AVPlayerError> {
        let value = value
            .map(|value| to_cstring(value, "resource renewal date"))
            .transpose()?;
        unsafe {
            ffi::av_asset_resource_loading_content_information_request_set_renewal_date_iso8601(
                self.ptr,
                value.as_ref().map_or(ptr::null(), |value| value.as_ptr()),
            );
        }
        Ok(())
    }

    pub fn set_entire_length_available_on_demand(
        &self,
        available: bool,
    ) -> Result<(), AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::av_asset_resource_loading_content_information_request_set_entire_length_available_on_demand(
                self.ptr,
                available,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }
}

impl AssetResourceLoadingDataRequest {
    fn info(&self) -> Result<AssetResourceLoadingDataRequestPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::av_asset_resource_loading_data_request_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn requested_offset(&self) -> Result<i64, AVPlayerError> {
        Ok(self.info()?.requested_offset)
    }

    pub fn requested_length(&self) -> Result<i64, AVPlayerError> {
        Ok(self.info()?.requested_length)
    }

    pub fn requests_all_data_to_end_of_resource(&self) -> Result<Option<bool>, AVPlayerError> {
        Ok(self.info()?.requests_all_data_to_end_of_resource)
    }

    pub fn current_offset(&self) -> Result<i64, AVPlayerError> {
        Ok(self.info()?.current_offset)
    }

    pub fn respond_with_data(&self, data: &[u8]) {
        unsafe {
            ffi::av_asset_resource_loading_data_request_respond_with_data(
                self.ptr,
                data.as_ptr(),
                data.len(),
            );
        }
    }
}

impl AssetResourceLoadingRequestor {
    fn info(&self) -> Result<AssetResourceLoadingRequestorPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::av_asset_resource_loading_requestor_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn provides_expired_session_reports(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.provides_expired_session_reports)
    }
}

fn resource_loading_request_info(
    ptr: *mut c_void,
) -> Result<AssetResourceLoadingRequestPayload, AVPlayerError> {
    let mut err: *mut c_char = ptr::null_mut();
    let json_ptr = unsafe { ffi::av_asset_resource_loading_request_info_json(ptr, &mut err) };
    if json_ptr.is_null() {
        return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
    }
    parse_json_and_free(json_ptr)
}

fn copy_content_information_request(
    ptr: *mut c_void,
) -> Option<AssetResourceLoadingContentInformationRequest> {
    let ptr =
        unsafe { ffi::av_asset_resource_loading_request_copy_content_information_request(ptr) };
    if ptr.is_null() {
        None
    } else {
        Some(AssetResourceLoadingContentInformationRequest { ptr })
    }
}

fn copy_data_request(ptr: *mut c_void) -> Option<AssetResourceLoadingDataRequest> {
    let ptr = unsafe { ffi::av_asset_resource_loading_request_copy_data_request(ptr) };
    if ptr.is_null() {
        None
    } else {
        Some(AssetResourceLoadingDataRequest { ptr })
    }
}

fn copy_requestor(ptr: *mut c_void) -> Option<AssetResourceLoadingRequestor> {
    let ptr = unsafe { ffi::av_asset_resource_loading_request_copy_requestor(ptr) };
    if ptr.is_null() {
        None
    } else {
        Some(AssetResourceLoadingRequestor { ptr })
    }
}

unsafe extern "C" fn asset_resource_loader_event_trampoline(
    userdata: *mut c_void,
    event_name: *const c_char,
    object_ptr: *mut c_void,
) -> bool {
    if userdata.is_null() || event_name.is_null() || object_ptr.is_null() {
        if !object_ptr.is_null() {
            ffi::av_ns_object_release(object_ptr);
        }
        return false;
    }

    let state = &*userdata.cast::<AssetResourceLoaderObserverState>();
    let Ok(event_name) = CStr::from_ptr(event_name).to_str() else {
        ffi::av_ns_object_release(object_ptr);
        return false;
    };

    let event = match event_name {
        "loading_requested" => {
            AssetResourceLoaderEvent::LoadingRequested(AssetResourceLoadingRequest {
                ptr: object_ptr,
            })
        }
        "renewal_requested" => {
            AssetResourceLoaderEvent::RenewalRequested(AssetResourceRenewalRequest {
                ptr: object_ptr,
            })
        }
        "loading_cancelled" => {
            AssetResourceLoaderEvent::LoadingCancelled(AssetResourceLoadingRequest {
                ptr: object_ptr,
            })
        }
        _ => {
            ffi::av_ns_object_release(object_ptr);
            return false;
        }
    };

    let mut result = false;
    catch_cb_panic("asset_resource_loader_event_trampoline", || {
        result = (state.callback)(event);
    });
    result
}

unsafe extern "C" fn asset_resource_loader_observer_drop(userdata: *mut c_void) {
    if !userdata.is_null() {
        drop(Box::from_raw(
            userdata.cast::<AssetResourceLoaderObserverState>(),
        ));
    }
}

// SAFETY: AVFoundation resource-loader wrappers and observer tokens are opaque
// retained objects that may cross thread boundaries for async delegate handling.
unsafe impl Send for AssetResourceLoader {}
unsafe impl Send for AssetResourceLoadingRequest {}
unsafe impl Send for AssetResourceRenewalRequest {}
unsafe impl Send for AssetResourceLoadingContentInformationRequest {}
unsafe impl Send for AssetResourceLoadingDataRequest {}
unsafe impl Send for AssetResourceLoadingRequestor {}
unsafe impl Send for AssetResourceLoaderObserver {}
