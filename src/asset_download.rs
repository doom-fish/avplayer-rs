#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use std::ffi::{CStr, CString};
use std::path::Path;

use doom_fish_utils::stream::{BoundedAsyncStream, NextItem};
use serde::Deserialize;

use crate::asset::UrlAsset;
use crate::asset_variant::AssetVariantQualifier;
use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::media_selection::MediaSelection;
use crate::time::TimeRange;
use crate::util::{catch_cb_panic, parse_json_and_free, to_cstring};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AssetDownloadedAssetEvictionPriority {
    Important,
    Default,
    Unknown(String),
}

impl AssetDownloadedAssetEvictionPriority {
    fn from_raw(raw: &str) -> Self {
        match raw {
            "important" => Self::Important,
            "default" => Self::Default,
            other => Self::Unknown(other.to_owned()),
        }
    }

    fn as_raw(&self) -> &str {
        match self {
            Self::Important => "important",
            Self::Default => "default",
            Self::Unknown(raw) => raw,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssetDownloadStorageManagementPolicyPayload {
    priority: String,
    expiration_date_iso8601: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssetDownloadConfigurationPayload {
    optimizes_auxiliary_content_configurations: bool,
    downloads_interstitial_assets: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssetDownloadContentConfigurationPayload {
    variant_qualifier_count: usize,
    media_selection_count: usize,
}

#[derive(Debug)]
pub struct AssetDownloadStorageManager {
    ptr: *mut c_void,
}

impl Drop for AssetDownloadStorageManager {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

#[derive(Debug)]
pub struct AssetDownloadStorageManagementPolicy {
    ptr: *mut c_void,
}

impl Drop for AssetDownloadStorageManagementPolicy {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

#[derive(Debug)]
pub struct AssetDownloadConfiguration {
    ptr: *mut c_void,
}

impl Drop for AssetDownloadConfiguration {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

#[derive(Debug)]
pub struct AssetDownloadContentConfiguration {
    ptr: *mut c_void,
}

impl Drop for AssetDownloadContentConfiguration {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl AssetDownloadStorageManager {
    pub fn shared() -> Self {
        let ptr = unsafe { ffi::av_asset_download_storage_manager_shared() };
        Self { ptr }
    }

    pub fn set_storage_management_policy_for_file_path(
        &self,
        policy: &AssetDownloadStorageManagementPolicy,
        path: impl AsRef<Path>,
    ) -> Result<(), AVPlayerError> {
        let path = path
            .as_ref()
            .to_str()
            .ok_or_else(|| AVPlayerError::InvalidArgument("path is not valid UTF-8".into()))?;
        let path = to_cstring(path, "download storage path")?;
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::av_asset_download_storage_manager_set_policy_for_file_path(
                self.ptr,
                policy.ptr,
                path.as_ptr(),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn storage_management_policy_for_file_path(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<Option<AssetDownloadStorageManagementPolicy>, AVPlayerError> {
        let path = path
            .as_ref()
            .to_str()
            .ok_or_else(|| AVPlayerError::InvalidArgument("path is not valid UTF-8".into()))?;
        let path = to_cstring(path, "download storage path")?;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_asset_download_storage_manager_copy_policy_for_file_path(
                self.ptr,
                path.as_ptr(),
                &mut err,
            )
        };
        if ptr.is_null() {
            if err.is_null() {
                return Ok(None);
            }
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Some(AssetDownloadStorageManagementPolicy { ptr }))
    }
}

impl AssetDownloadStorageManagementPolicy {
    pub fn new_mutable() -> Self {
        let ptr = unsafe { ffi::av_asset_download_storage_management_policy_create_mutable() };
        Self { ptr }
    }

    fn info(&self) -> Result<AssetDownloadStorageManagementPolicyPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::av_asset_download_storage_management_policy_info_json(self.ptr, &mut err)
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn priority(&self) -> Result<AssetDownloadedAssetEvictionPriority, AVPlayerError> {
        Ok(AssetDownloadedAssetEvictionPriority::from_raw(
            &self.info()?.priority,
        ))
    }

    pub fn expiration_date_iso8601(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.expiration_date_iso8601)
    }

    pub fn set_priority(
        &self,
        priority: &AssetDownloadedAssetEvictionPriority,
    ) -> Result<(), AVPlayerError> {
        let priority = CString::new(priority.as_raw()).map_err(|error| {
            AVPlayerError::InvalidArgument(format!(
                "download storage priority contains NUL byte: {error}"
            ))
        })?;
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::av_asset_download_storage_management_policy_set_priority(
                self.ptr,
                priority.as_ptr(),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn set_expiration_date_iso8601(&self, value: &str) -> Result<(), AVPlayerError> {
        let value = to_cstring(value, "download storage expiration date")?;
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::av_asset_download_storage_management_policy_set_expiration_date_iso8601(
                self.ptr,
                value.as_ptr(),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }
}

impl UrlAsset {
    pub fn download_configuration(
        &self,
        title: &str,
    ) -> Result<AssetDownloadConfiguration, AVPlayerError> {
        let title = to_cstring(title, "download title")?;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_asset_download_configuration_create(self.asset.ptr, title.as_ptr(), &mut err)
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(AssetDownloadConfiguration { ptr })
    }
}

impl AssetDownloadConfiguration {
    fn info(&self) -> Result<AssetDownloadConfigurationPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::av_asset_download_configuration_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn optimizes_auxiliary_content_configurations(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.optimizes_auxiliary_content_configurations)
    }

    pub fn set_optimizes_auxiliary_content_configurations(&self, enabled: bool) {
        unsafe {
            ffi::av_asset_download_configuration_set_optimizes_auxiliary_content_configurations(
                self.ptr, enabled,
            );
        }
    }

    pub fn downloads_interstitial_assets(&self) -> Result<Option<bool>, AVPlayerError> {
        Ok(self.info()?.downloads_interstitial_assets)
    }

    pub fn set_downloads_interstitial_assets(&self, enabled: bool) -> Result<(), AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::av_asset_download_configuration_set_downloads_interstitial_assets(
                self.ptr, enabled, &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn set_artwork_data(&self, artwork_data: Option<&[u8]>) {
        let (ptr, len) = artwork_data.map_or((ptr::null(), 0), |artwork_data| {
            (artwork_data.as_ptr(), artwork_data.len())
        });
        unsafe { ffi::av_asset_download_configuration_set_artwork_data(self.ptr, ptr, len) };
    }

    pub fn primary_content_configuration(&self) -> AssetDownloadContentConfiguration {
        let ptr = unsafe {
            ffi::av_asset_download_configuration_copy_primary_content_configuration(self.ptr)
        };
        AssetDownloadContentConfiguration { ptr }
    }

    pub fn auxiliary_content_configurations(
        &self,
    ) -> Result<Vec<AssetDownloadContentConfiguration>, AVPlayerError> {
        let count = unsafe {
            ffi::av_asset_download_configuration_auxiliary_content_configuration_count(self.ptr)
        };
        if count < 0 {
            return Err(AVPlayerError::OperationFailed(format!(
                "invalid auxiliary content configuration count: {count}"
            )));
        }
        let count = usize::try_from(count).map_err(|error| {
            AVPlayerError::OperationFailed(format!(
                "invalid auxiliary content configuration count: {error}"
            ))
        })?;
        let mut values = Vec::with_capacity(count);
        for index in 0..count {
            let ptr = unsafe {
                ffi::av_asset_download_configuration_copy_auxiliary_content_configuration_at_index(
                    self.ptr,
                    i32::try_from(index).unwrap_or(i32::MAX),
                )
            };
            if ptr.is_null() {
                return Err(AVPlayerError::OperationFailed(format!(
                    "bridge returned null auxiliary content configuration at index {index}"
                )));
            }
            values.push(AssetDownloadContentConfiguration { ptr });
        }
        Ok(values)
    }

    pub fn set_auxiliary_content_configurations(
        &self,
        configurations: &[AssetDownloadContentConfiguration],
    ) {
        let ptrs = configurations
            .iter()
            .map(|value| value.ptr)
            .collect::<Vec<_>>();
        unsafe {
            ffi::av_asset_download_configuration_set_auxiliary_content_configurations(
                self.ptr,
                ptrs.as_ptr(),
                ptrs.len(),
            );
        }
    }
}

impl Default for AssetDownloadContentConfiguration {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetDownloadContentConfiguration {
    pub fn new() -> Self {
        let ptr = unsafe { ffi::av_asset_download_content_configuration_create() };
        Self { ptr }
    }

    fn info(&self) -> Result<AssetDownloadContentConfigurationPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::av_asset_download_content_configuration_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn variant_qualifier_count(&self) -> Result<usize, AVPlayerError> {
        Ok(self.info()?.variant_qualifier_count)
    }

    pub fn media_selection_count(&self) -> Result<usize, AVPlayerError> {
        Ok(self.info()?.media_selection_count)
    }

    pub fn media_selections(&self) -> Result<Vec<MediaSelection>, AVPlayerError> {
        let count =
            unsafe { ffi::av_asset_download_content_configuration_media_selection_count(self.ptr) };
        if count < 0 {
            return Err(AVPlayerError::OperationFailed(format!(
                "invalid download media selection count: {count}"
            )));
        }
        let count = usize::try_from(count).map_err(|error| {
            AVPlayerError::OperationFailed(format!(
                "invalid download media selection count: {error}"
            ))
        })?;
        let mut values = Vec::with_capacity(count);
        for index in 0..count {
            let ptr = unsafe {
                ffi::av_asset_download_content_configuration_copy_media_selection_at_index(
                    self.ptr,
                    i32::try_from(index).unwrap_or(i32::MAX),
                )
            };
            if ptr.is_null() {
                return Err(AVPlayerError::OperationFailed(format!(
                    "bridge returned null download media selection at index {index}"
                )));
            }
            values.push(MediaSelection { ptr });
        }
        Ok(values)
    }

    pub fn set_media_selections(&self, media_selections: &[MediaSelection]) {
        let ptrs = media_selections
            .iter()
            .map(|value| value.ptr)
            .collect::<Vec<_>>();
        unsafe {
            ffi::av_asset_download_content_configuration_set_media_selections(
                self.ptr,
                ptrs.as_ptr(),
                ptrs.len(),
            );
        }
    }

    pub fn set_variant_qualifiers(&self, variant_qualifiers: &[AssetVariantQualifier]) {
        let ptrs = variant_qualifiers
            .iter()
            .map(|value| value.ptr)
            .collect::<Vec<_>>();
        unsafe {
            ffi::av_asset_download_content_configuration_set_variant_qualifiers(
                self.ptr,
                ptrs.as_ptr(),
                ptrs.len(),
            );
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssetDownloadTaskInfoPayload {
    task_identifier: usize,
    url_asset_url: Option<String>,
    state: i32,
    #[serde(default)]
    loaded_time_ranges: Vec<TimeRange>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssetDownloadDelegateEventPayload {
    event: String,
    task_identifier: usize,
    location: Option<String>,
    loaded_time_range: Option<TimeRange>,
    total_time_ranges_loaded: Option<Vec<TimeRange>>,
    time_range_expected_to_load: Option<TimeRange>,
    variant_count: Option<usize>,
    metric_event_class_name: Option<String>,
}

impl AssetDownloadDelegateEventPayload {
    fn require_location(self) -> Option<(usize, String)> {
        self.location
            .map(|location| (self.task_identifier, location))
    }

    fn require_time_ranges(self) -> Option<(usize, TimeRange, Vec<TimeRange>, TimeRange)> {
        Some((
            self.task_identifier,
            self.loaded_time_range?,
            self.total_time_ranges_loaded?,
            self.time_range_expected_to_load?,
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AssetDownloadTaskState {
    Running,
    Suspended,
    Canceling,
    Completed,
    Unknown(i32),
}

impl AssetDownloadTaskState {
    const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::Running,
            1 => Self::Suspended,
            2 => Self::Canceling,
            3 => Self::Completed,
            other => Self::Unknown(other),
        }
    }
}

#[derive(Debug)]
pub struct AssetDownloadTask {
    ptr: *mut c_void,
}

impl Drop for AssetDownloadTask {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

#[derive(Debug)]
pub struct AggregateAssetDownloadTask {
    ptr: *mut c_void,
}

impl Drop for AggregateAssetDownloadTask {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

#[derive(Debug)]
pub struct AssetDownloadURLSession {
    ptr: *mut c_void,
}

impl Drop for AssetDownloadURLSession {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_asset_download_url_session_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum AssetDownloadDelegateEvent {
    WillDownloadToUrl {
        task_identifier: usize,
        location: String,
    },
    DidLoadTimeRange {
        task_identifier: usize,
        loaded_time_range: TimeRange,
        total_time_ranges_loaded: Vec<TimeRange>,
        time_range_expected_to_load: TimeRange,
    },
    DidResolveMediaSelection {
        task_identifier: usize,
    },
    AggregateWillDownloadToUrl {
        task_identifier: usize,
        location: String,
    },
    AggregateDidCompleteForMediaSelection {
        task_identifier: usize,
    },
    AggregateDidLoadTimeRange {
        task_identifier: usize,
        loaded_time_range: TimeRange,
        total_time_ranges_loaded: Vec<TimeRange>,
        time_range_expected_to_load: TimeRange,
    },
    WillDownloadVariants {
        task_identifier: usize,
        variant_count: usize,
    },
    DidReceiveMetricEvent {
        task_identifier: usize,
        metric_event_class_name: String,
    },
}

struct AssetDownloadDelegateState {
    callback: Box<dyn Fn(AssetDownloadDelegateEvent) + Send + 'static>,
}

#[derive(Debug)]
/// Async stream of delegate events sourced from `AVAssetDownloadURLSession`.
pub struct AssetDownloadDelegateEventStream {
    inner: BoundedAsyncStream<AssetDownloadDelegateEvent>,
}

impl AssetDownloadDelegateEventStream {
    #[must_use]
    /// Returns the next buffered delegate event.
    pub const fn next(&self) -> NextItem<'_, AssetDownloadDelegateEvent> {
        self.inner.next()
    }

    #[must_use]
    /// Returns the next buffered delegate event if one is available.
    pub fn try_next(&self) -> Option<AssetDownloadDelegateEvent> {
        self.inner.try_next()
    }

    #[must_use]
    /// Returns the number of currently buffered delegate events.
    pub fn buffered_count(&self) -> usize {
        self.inner.buffered_count()
    }

    /// Drops all currently buffered delegate events without closing the stream.
    pub fn clear_buffer(&self) {
        self.inner.clear_buffer();
    }

    #[must_use]
    /// Returns whether the stream has been closed.
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }
}

impl AssetDownloadURLSession {
    pub fn background(identifier: &str) -> Result<Self, AVPlayerError> {
        Self::background_with_handler(identifier, None, |_| {})
    }

    /// Creates a background download session together with an async delegate-event stream.
    pub fn background_with_events(
        identifier: &str,
        queue_label: Option<&str>,
        capacity: usize,
    ) -> Result<(Self, AssetDownloadDelegateEventStream), AVPlayerError> {
        let (inner, sender) = BoundedAsyncStream::new(capacity);
        let session = Self::background_with_handler(identifier, queue_label, move |event| {
            sender.push(event);
        })?;
        Ok((session, AssetDownloadDelegateEventStream { inner }))
    }

    /// Creates a background download session with the default queue choice and an async delegate-event stream.
    pub fn background_events(
        identifier: &str,
        capacity: usize,
    ) -> Result<(Self, AssetDownloadDelegateEventStream), AVPlayerError> {
        Self::background_with_events(identifier, None, capacity)
    }

    pub fn background_with_handler<F>(
        identifier: &str,
        queue_label: Option<&str>,
        callback: F,
    ) -> Result<Self, AVPlayerError>
    where
        F: Fn(AssetDownloadDelegateEvent) + Send + 'static,
    {
        let identifier = to_cstring(identifier, "asset-download background identifier")?;
        let queue_label = queue_label
            .map(|label| to_cstring(label, "asset-download delegate queue label"))
            .transpose()?;
        let state = Box::new(AssetDownloadDelegateState {
            callback: Box::new(callback),
        });
        let userdata = Box::into_raw(state).cast::<c_void>();
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_asset_download_url_session_create_background(
                identifier.as_ptr(),
                queue_label
                    .as_ref()
                    .map_or(ptr::null(), |label| label.as_ptr()),
                Some(asset_download_delegate_event_trampoline),
                userdata,
                Some(asset_download_delegate_drop),
                &mut err,
            )
        };
        if ptr.is_null() {
            unsafe { asset_download_delegate_drop(userdata) };
            return Err(unsafe { from_swift(ffi::status::OBSERVER_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    pub fn asset_download_task(
        &self,
        configuration: &AssetDownloadConfiguration,
    ) -> Result<AssetDownloadTask, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_asset_download_url_session_create_task_with_configuration(
                self.ptr,
                configuration.ptr,
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(AssetDownloadTask { ptr })
    }

    pub fn aggregate_asset_download_task(
        &self,
        asset: &UrlAsset,
        media_selections: &[MediaSelection],
        title: &str,
        artwork_data: Option<&[u8]>,
    ) -> Result<AggregateAssetDownloadTask, AVPlayerError> {
        let media_selection_ptrs = media_selections
            .iter()
            .map(|selection| selection.ptr)
            .collect::<Vec<_>>();
        let title = to_cstring(title, "aggregate-download title")?;
        let (artwork_ptr, artwork_len) =
            artwork_data.map_or((ptr::null(), 0), |data| (data.as_ptr(), data.len()));
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_asset_download_url_session_create_aggregate_task(
                self.ptr,
                asset.asset.ptr,
                media_selection_ptrs.as_ptr(),
                media_selection_ptrs.len(),
                title.as_ptr(),
                artwork_ptr,
                artwork_len,
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(AggregateAssetDownloadTask { ptr })
    }

    pub fn finish_tasks_and_invalidate(&self) {
        unsafe { ffi::av_asset_download_url_session_finish_tasks_and_invalidate(self.ptr) };
    }

    pub fn invalidate_and_cancel(&self) {
        unsafe { ffi::av_asset_download_url_session_invalidate_and_cancel(self.ptr) };
    }
}

impl AssetDownloadTask {
    fn info(&self) -> Result<AssetDownloadTaskInfoPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_asset_download_task_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn task_identifier(&self) -> Result<usize, AVPlayerError> {
        Ok(self.info()?.task_identifier)
    }

    pub fn url_asset_url(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.url_asset_url)
    }

    pub fn state(&self) -> Result<AssetDownloadTaskState, AVPlayerError> {
        Ok(AssetDownloadTaskState::from_raw(self.info()?.state))
    }

    pub fn loaded_time_ranges(&self) -> Result<Vec<TimeRange>, AVPlayerError> {
        Ok(self.info()?.loaded_time_ranges)
    }

    pub fn resume(&self) {
        unsafe { ffi::av_asset_download_task_resume(self.ptr) };
    }

    pub fn suspend(&self) {
        unsafe { ffi::av_asset_download_task_suspend(self.ptr) };
    }

    pub fn cancel(&self) {
        unsafe { ffi::av_asset_download_task_cancel(self.ptr) };
    }
}

impl AggregateAssetDownloadTask {
    fn info(&self) -> Result<AssetDownloadTaskInfoPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::av_aggregate_asset_download_task_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn task_identifier(&self) -> Result<usize, AVPlayerError> {
        Ok(self.info()?.task_identifier)
    }

    pub fn url_asset_url(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.url_asset_url)
    }

    pub fn state(&self) -> Result<AssetDownloadTaskState, AVPlayerError> {
        Ok(AssetDownloadTaskState::from_raw(self.info()?.state))
    }

    pub fn resume(&self) {
        unsafe { ffi::av_asset_download_task_resume(self.ptr) };
    }

    pub fn suspend(&self) {
        unsafe { ffi::av_asset_download_task_suspend(self.ptr) };
    }

    pub fn cancel(&self) {
        unsafe { ffi::av_asset_download_task_cancel(self.ptr) };
    }
}

unsafe extern "C" fn asset_download_delegate_event_trampoline(
    userdata: *mut c_void,
    payload_json: *const c_char,
) {
    if userdata.is_null() || payload_json.is_null() {
        return;
    }

    let state = &*userdata.cast::<AssetDownloadDelegateState>();
    let Ok(payload) = CStr::from_ptr(payload_json).to_str() else {
        return;
    };
    let Ok(payload) = serde_json::from_str::<AssetDownloadDelegateEventPayload>(payload) else {
        return;
    };

    let event = match payload.event.as_str() {
        "will_download_to_url" => {
            let Some((task_identifier, location)) = payload.require_location() else {
                return;
            };
            AssetDownloadDelegateEvent::WillDownloadToUrl {
                task_identifier,
                location,
            }
        }
        "did_load_time_range" => {
            let Some((
                task_identifier,
                loaded_time_range,
                total_time_ranges_loaded,
                time_range_expected_to_load,
            )) = payload.require_time_ranges()
            else {
                return;
            };
            AssetDownloadDelegateEvent::DidLoadTimeRange {
                task_identifier,
                loaded_time_range,
                total_time_ranges_loaded,
                time_range_expected_to_load,
            }
        }
        "did_resolve_media_selection" => AssetDownloadDelegateEvent::DidResolveMediaSelection {
            task_identifier: payload.task_identifier,
        },
        "aggregate_will_download_to_url" => {
            let Some((task_identifier, location)) = payload.require_location() else {
                return;
            };
            AssetDownloadDelegateEvent::AggregateWillDownloadToUrl {
                task_identifier,
                location,
            }
        }
        "aggregate_did_complete_for_media_selection" => {
            AssetDownloadDelegateEvent::AggregateDidCompleteForMediaSelection {
                task_identifier: payload.task_identifier,
            }
        }
        "aggregate_did_load_time_range" => {
            let Some((
                task_identifier,
                loaded_time_range,
                total_time_ranges_loaded,
                time_range_expected_to_load,
            )) = payload.require_time_ranges()
            else {
                return;
            };
            AssetDownloadDelegateEvent::AggregateDidLoadTimeRange {
                task_identifier,
                loaded_time_range,
                total_time_ranges_loaded,
                time_range_expected_to_load,
            }
        }
        "will_download_variants" => AssetDownloadDelegateEvent::WillDownloadVariants {
            task_identifier: payload.task_identifier,
            variant_count: payload.variant_count.unwrap_or_default(),
        },
        "did_receive_metric_event" => AssetDownloadDelegateEvent::DidReceiveMetricEvent {
            task_identifier: payload.task_identifier,
            metric_event_class_name: payload.metric_event_class_name.unwrap_or_default(),
        },
        _ => return,
    };

    catch_cb_panic("asset_download_delegate_event_trampoline", || {
        (state.callback)(event);
    });
}

unsafe extern "C" fn asset_download_delegate_drop(userdata: *mut c_void) {
    if !userdata.is_null() {
        drop(Box::from_raw(userdata.cast::<AssetDownloadDelegateState>()));
    }
}
