#![allow(
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::struct_excessive_bools
)]

use core::ffi::{c_char, c_void, CStr};
use core::ptr;
use std::ffi::CString;
use std::panic::AssertUnwindSafe;
use std::path::Path;

use doom_fish_utils::stream::{BoundedAsyncStream, NextItem};
use serde::{Deserialize, Serialize};

use crate::asset::UrlAsset;
use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::util::{json_cstring, maybe_json_cstring, parse_json_and_free, to_cstring};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContentKeySystem {
    FairPlayStreaming,
    ClearKey,
    AuthorizationToken,
    Unknown(String),
}

impl ContentKeySystem {
    fn from_raw(raw: &str) -> Self {
        match raw {
            "fair_play_streaming" => Self::FairPlayStreaming,
            "clear_key" => Self::ClearKey,
            "authorization_token" => Self::AuthorizationToken,
            other => Self::Unknown(other.to_owned()),
        }
    }

    fn as_raw(&self) -> &str {
        match self {
            Self::FairPlayStreaming => "fair_play_streaming",
            Self::ClearKey => "clear_key",
            Self::AuthorizationToken => "authorization_token",
            Self::Unknown(raw) => raw,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContentKeyIdentifier {
    String(String),
    Url(String),
    Data(Vec<u8>),
    Number(i64),
}

impl ContentKeyIdentifier {
    #[must_use]
    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    #[must_use]
    pub fn url(value: impl Into<String>) -> Self {
        Self::Url(value.into())
    }

    #[must_use]
    pub fn data(bytes: impl Into<Vec<u8>>) -> Self {
        Self::Data(bytes.into())
    }

    #[must_use]
    pub const fn number(value: i64) -> Self {
        Self::Number(value)
    }

    fn to_payload(&self) -> ContentKeyIdentifierPayload {
        match self {
            Self::String(value) => ContentKeyIdentifierPayload {
                kind: "string".into(),
                value: Some(value.clone()),
                bytes: None,
                number_value: None,
            },
            Self::Url(value) => ContentKeyIdentifierPayload {
                kind: "url".into(),
                value: Some(value.clone()),
                bytes: None,
                number_value: None,
            },
            Self::Data(bytes) => ContentKeyIdentifierPayload {
                kind: "data".into(),
                value: None,
                bytes: Some(bytes.clone()),
                number_value: None,
            },
            Self::Number(value) => ContentKeyIdentifierPayload {
                kind: "number".into(),
                value: None,
                bytes: None,
                number_value: Some(*value),
            },
        }
    }

    fn from_payload(payload: ContentKeyIdentifierPayload) -> Self {
        match payload.kind.as_str() {
            "url" => Self::Url(payload.value.unwrap_or_default()),
            "data" => Self::Data(payload.bytes.unwrap_or_default()),
            "number" => Self::Number(payload.number_value.unwrap_or_default()),
            _ => Self::String(payload.value.unwrap_or_default()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContentKeyIdentifierPayload {
    kind: String,
    value: Option<String>,
    bytes: Option<Vec<u8>>,
    number_value: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentKeyRequestOptions {
    #[serde(default)]
    pub protocol_versions: Vec<i32>,
    pub should_randomize_device_identifier: Option<bool>,
    pub random_device_identifier_seed: Option<Vec<u8>>,
}

impl ContentKeyRequestOptions {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.protocol_versions.is_empty()
            && self.should_randomize_device_identifier.is_none()
            && self.random_device_identifier_seed.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContentKeyRequestRetryReason {
    TimedOut,
    ReceivedResponseWithExpiredLease,
    ReceivedObsoleteContentKey,
    Unknown(String),
}

impl ContentKeyRequestRetryReason {
    fn from_raw(raw: &str) -> Self {
        match raw {
            "timed_out" => Self::TimedOut,
            "received_response_with_expired_lease" => Self::ReceivedResponseWithExpiredLease,
            "received_obsolete_content_key" => Self::ReceivedObsoleteContentKey,
            other => Self::Unknown(other.to_owned()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContentKeyRequestStatus {
    RequestingResponse,
    ReceivedResponse,
    Renewed,
    Retried,
    Cancelled,
    Failed,
    Unknown(i32),
}

impl ContentKeyRequestStatus {
    const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::RequestingResponse,
            1 => Self::ReceivedResponse,
            2 => Self::Renewed,
            3 => Self::Retried,
            4 => Self::Cancelled,
            5 => Self::Failed,
            other => Self::Unknown(other),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExternalContentProtectionStatus {
    Pending,
    Sufficient,
    Insufficient,
    Unknown(i32),
}

impl ExternalContentProtectionStatus {
    const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::Pending,
            1 => Self::Sufficient,
            2 => Self::Insufficient,
            other => Self::Unknown(other),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContentKeySessionInfoPayload {
    key_system: String,
    storage_url: Option<String>,
    content_protection_session_identifier_base64: Option<String>,
    recipient_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContentKeyRequestInfoPayload {
    status: i32,
    error_message: Option<String>,
    identifier: Option<ContentKeyIdentifierPayload>,
    initialization_data: Option<Vec<u8>>,
    options: Option<ContentKeyRequestOptions>,
    can_provide_persistable_content_key: bool,
    renews_expiring_response_data: bool,
    has_content_key_specifier: bool,
    has_content_key: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContentKeySpecifierInfoPayload {
    key_system: String,
    identifier: Option<ContentKeyIdentifierPayload>,
    options: Option<ContentKeyRequestOptions>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContentKeyInfoPayload {
    external_content_protection_status: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContentKeyBytesPayload {
    bytes: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContentKeySessionEventPayload {
    event: String,
    request_ptr: Option<u64>,
    key_request_ptrs: Option<Vec<u64>>,
    content_key_ptr: Option<u64>,
    initialization_data: Option<Vec<u8>>,
    persistable_content_key: Option<Vec<u8>>,
    key_identifier: Option<ContentKeyIdentifierPayload>,
    error_message: Option<String>,
    retry_reason: Option<String>,
}

#[derive(Debug)]
pub struct ContentKeySession {
    ptr: *mut c_void,
}

impl Drop for ContentKeySession {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

#[derive(Debug)]
pub struct ContentKeyRequest {
    ptr: *mut c_void,
}

impl Drop for ContentKeyRequest {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

#[derive(Debug)]
pub struct PersistableContentKeyRequest {
    ptr: *mut c_void,
}

impl Drop for PersistableContentKeyRequest {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

#[derive(Debug)]
pub struct ContentKeyResponse {
    ptr: *mut c_void,
}

impl Drop for ContentKeyResponse {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

#[derive(Debug)]
pub struct ContentKeySpecifier {
    ptr: *mut c_void,
}

impl Drop for ContentKeySpecifier {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

#[derive(Debug)]
pub struct ContentKey {
    ptr: *mut c_void,
}

impl Drop for ContentKey {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_ns_object_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

#[derive(Debug)]
pub struct ContentKeySessionObserver {
    token: *mut c_void,
}

impl Drop for ContentKeySessionObserver {
    fn drop(&mut self) {
        if !self.token.is_null() {
            unsafe { ffi::av_content_key_session_observer_release(self.token) };
            self.token = ptr::null_mut();
        }
    }
}

#[derive(Debug)]
pub struct ContentKeySessionEventStream {
    inner: BoundedAsyncStream<ContentKeySessionEvent>,
    _observer: ContentKeySessionObserver,
}

impl ContentKeySessionEventStream {
    #[must_use]
    pub const fn next(&self) -> NextItem<'_, ContentKeySessionEvent> {
        self.inner.next()
    }

    #[must_use]
    pub fn try_next(&self) -> Option<ContentKeySessionEvent> {
        self.inner.try_next()
    }

    #[must_use]
    pub fn buffered_count(&self) -> usize {
        self.inner.buffered_count()
    }

    pub fn clear_buffer(&self) {
        self.inner.clear_buffer();
    }

    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ContentKeySessionEvent {
    Requested(ContentKeyRequest),
    Renewing(ContentKeyRequest),
    Persistable(PersistableContentKeyRequest),
    UpdatedPersistableContentKey {
        persistable_content_key: Vec<u8>,
        key_identifier: Option<ContentKeyIdentifier>,
    },
    Failed {
        request: ContentKeyRequest,
        error_message: String,
    },
    RetryRequested {
        request: ContentKeyRequest,
        reason: ContentKeyRequestRetryReason,
    },
    Succeeded(ContentKeyRequest),
    ContentProtectionSessionIdentifierDidChange,
    ExpiredSessionReportGenerated,
    ExternalProtectionStatusDidChange(ContentKey),
    RequestedCollection {
        requests: Vec<ContentKeyRequest>,
        initialization_data: Option<Vec<u8>>,
    },
}

pub type ContentKeyEvent = ContentKeySessionEvent;

struct ContentKeySessionObserverState {
    callback: Box<dyn Fn(ContentKeySessionEvent) -> bool + Send + 'static>,
}

impl UrlAsset {
    pub fn may_require_content_keys_for_media_data_processing(&self) -> bool {
        unsafe {
            ffi::av_url_asset_may_require_content_keys_for_media_data_processing(self.asset.ptr)
        }
    }
}

impl ContentKeySession {
    pub fn new(key_system: &ContentKeySystem) -> Result<Self, AVPlayerError> {
        let key_system = CString::new(key_system.as_raw()).map_err(|error| {
            AVPlayerError::InvalidArgument(format!("key system contains NUL byte: {error}"))
        })?;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::av_content_key_session_create(key_system.as_ptr(), &mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    pub fn with_storage_directory(
        key_system: &ContentKeySystem,
        path: impl AsRef<Path>,
    ) -> Result<Self, AVPlayerError> {
        let key_system = CString::new(key_system.as_raw()).map_err(|error| {
            AVPlayerError::InvalidArgument(format!("key system contains NUL byte: {error}"))
        })?;
        let path = path
            .as_ref()
            .to_str()
            .ok_or_else(|| AVPlayerError::InvalidArgument("path is not valid UTF-8".into()))?;
        let path = to_cstring(path, "content-key storage directory")?;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_content_key_session_create_with_storage_directory(
                key_system.as_ptr(),
                path.as_ptr(),
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn info(&self) -> Result<ContentKeySessionInfoPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_content_key_session_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn key_system(&self) -> Result<ContentKeySystem, AVPlayerError> {
        Ok(ContentKeySystem::from_raw(&self.info()?.key_system))
    }

    pub fn storage_url(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.storage_url)
    }

    pub fn content_protection_session_identifier_base64(
        &self,
    ) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.content_protection_session_identifier_base64)
    }

    pub fn recipient_count(&self) -> Result<usize, AVPlayerError> {
        Ok(self.info()?.recipient_count)
    }

    pub fn add_content_key_recipient(&self, recipient: &UrlAsset) {
        unsafe {
            ffi::av_content_key_session_add_content_key_recipient(self.ptr, recipient.asset.ptr);
        }
    }

    pub fn remove_content_key_recipient(&self, recipient: &UrlAsset) {
        unsafe {
            ffi::av_content_key_session_remove_content_key_recipient(self.ptr, recipient.asset.ptr);
        }
    }

    pub fn process_content_key_request(
        &self,
        identifier: Option<&ContentKeyIdentifier>,
        initialization_data: Option<&[u8]>,
        options: Option<&ContentKeyRequestOptions>,
    ) -> Result<(), AVPlayerError> {
        if identifier.is_none() && initialization_data.is_none() {
            return Err(AVPlayerError::InvalidArgument(
                "content-key requests require an identifier, initialization data, or both".into(),
            ));
        }

        if let Some(identifier) = identifier {
            match (self.key_system()?, identifier) {
                (ContentKeySystem::ClearKey | ContentKeySystem::AuthorizationToken, ContentKeyIdentifier::String(_)) => {}
                (ContentKeySystem::ClearKey | ContentKeySystem::AuthorizationToken, _) => {
                    return Err(AVPlayerError::InvalidArgument(
                        "clear-key and authorization-token content-key requests require string identifiers"
                            .into(),
                    ));
                }
                _ => {}
            }
        }

        let identifier_json = identifier
            .map(|identifier| json_cstring(&identifier.to_payload(), "content-key identifier"))
            .transpose()?;
        let options_json = maybe_json_cstring(options, "content-key request options")?;
        let initialization_data = initialization_data.unwrap_or_default();
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::av_content_key_session_process_content_key_request(
                self.ptr,
                identifier_json
                    .as_ref()
                    .map_or(ptr::null(), |json| json.as_ptr()),
                if initialization_data.is_empty() {
                    ptr::null()
                } else {
                    initialization_data.as_ptr()
                },
                initialization_data.len(),
                options_json.as_ref().map_or(ptr::null(), |json| json.as_ptr()),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn renew_expiring_response_data_for_content_key_request(
        &self,
        request: &ContentKeyRequest,
    ) -> Result<(), AVPlayerError> {
        if self.key_system()? != ContentKeySystem::FairPlayStreaming {
            return Err(AVPlayerError::InvalidArgument(
                "content-key renewal requires the FairPlay Streaming key system".into(),
            ));
        }
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::av_content_key_session_renew_expiring_response_data_for_request(
                self.ptr,
                request.ptr,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn observe<F>(
        &self,
        queue_label: Option<&str>,
        callback: F,
    ) -> Result<ContentKeySessionObserver, AVPlayerError>
    where
        F: Fn(ContentKeySessionEvent) -> bool + Send + 'static,
    {
        let queue_label = to_cstring(
            queue_label.unwrap_or("avplayer.content-key-session"),
            "content-key session queue label",
        )?;
        let state = Box::new(ContentKeySessionObserverState {
            callback: Box::new(callback),
        });
        let userdata = Box::into_raw(state).cast::<c_void>();
        let mut err: *mut c_char = ptr::null_mut();
        let token = unsafe {
            ffi::av_content_key_session_add_observer(
                self.ptr,
                queue_label.as_ptr(),
                Some(content_key_session_event_trampoline),
                userdata,
                Some(content_key_session_observer_drop),
                &mut err,
            )
        };
        if token.is_null() {
            unsafe { content_key_session_observer_drop(userdata) };
            return Err(unsafe { from_swift(ffi::status::OBSERVER_FAILED, err) });
        }
        Ok(ContentKeySessionObserver { token })
    }

    pub fn observe_events(
        &self,
        queue_label: Option<&str>,
        capacity: usize,
    ) -> Result<ContentKeySessionEventStream, AVPlayerError> {
        let (inner, sender) = BoundedAsyncStream::new(capacity);
        let observer = self.observe(queue_label, move |event| {
            sender.push(event);
            false
        })?;
        Ok(ContentKeySessionEventStream {
            inner,
            _observer: observer,
        })
    }

    pub fn event_stream(
        &self,
        capacity: usize,
    ) -> Result<ContentKeySessionEventStream, AVPlayerError> {
        self.observe_events(Some("avplayer.content-key-session"), capacity)
    }

    pub fn expire(&self) {
        unsafe { ffi::av_content_key_session_expire(self.ptr) };
    }
}

impl ContentKeyResponse {
    pub fn fair_play_streaming_key_response_data(
        bytes: impl AsRef<[u8]>,
    ) -> Result<Self, AVPlayerError> {
        let bytes = bytes.as_ref();
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_content_key_response_create_fair_play_streaming(
                if bytes.is_empty() { ptr::null() } else { bytes.as_ptr() },
                bytes.len(),
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    pub fn clear_key_data(
        key_data: impl AsRef<[u8]>,
        initialization_vector: Option<&[u8]>,
    ) -> Result<Self, AVPlayerError> {
        let key_data = key_data.as_ref();
        let initialization_vector = initialization_vector.unwrap_or_default();
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_content_key_response_create_clear_key(
                if key_data.is_empty() {
                    ptr::null()
                } else {
                    key_data.as_ptr()
                },
                key_data.len(),
                if initialization_vector.is_empty() {
                    ptr::null()
                } else {
                    initialization_vector.as_ptr()
                },
                initialization_vector.len(),
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    pub fn authorization_token_data(bytes: impl AsRef<[u8]>) -> Result<Self, AVPlayerError> {
        let bytes = bytes.as_ref();
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_content_key_response_create_authorization_token(
                if bytes.is_empty() { ptr::null() } else { bytes.as_ptr() },
                bytes.len(),
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }
}

impl ContentKeySpecifier {
    pub fn new(
        key_system: &ContentKeySystem,
        identifier: &ContentKeyIdentifier,
        options: Option<&ContentKeyRequestOptions>,
    ) -> Result<Self, AVPlayerError> {
        let key_system = to_cstring(key_system.as_raw(), "content-key system")?;
        let identifier = json_cstring(&identifier.to_payload(), "content-key identifier")?;
        let options = maybe_json_cstring(options, "content-key specifier options")?;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_content_key_specifier_create(
                key_system.as_ptr(),
                identifier.as_ptr(),
                options.as_ref().map_or(ptr::null(), |value| value.as_ptr()),
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    fn info(&self) -> Result<ContentKeySpecifierInfoPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_content_key_specifier_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn key_system(&self) -> Result<ContentKeySystem, AVPlayerError> {
        Ok(ContentKeySystem::from_raw(&self.info()?.key_system))
    }

    pub fn identifier(&self) -> Result<Option<ContentKeyIdentifier>, AVPlayerError> {
        Ok(self.info()?.identifier.map(ContentKeyIdentifier::from_payload))
    }

    pub fn options(&self) -> Result<ContentKeyRequestOptions, AVPlayerError> {
        Ok(self.info()?.options.unwrap_or_default())
    }
}

impl ContentKey {
    fn info(&self) -> Result<ContentKeyInfoPayload, AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_content_key_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn content_key_specifier(&self) -> Option<ContentKeySpecifier> {
        let ptr = unsafe { ffi::av_content_key_copy_content_key_specifier(self.ptr) };
        if ptr.is_null() {
            None
        } else {
            Some(ContentKeySpecifier { ptr })
        }
    }

    pub fn external_content_protection_status(
        &self,
    ) -> Result<Option<ExternalContentProtectionStatus>, AVPlayerError> {
        Ok(self
            .info()?
            .external_content_protection_status
            .map(ExternalContentProtectionStatus::from_raw))
    }

    pub fn revoke(&self) -> Result<(), AVPlayerError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe { ffi::av_content_key_revoke(self.ptr, &mut err) };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }
}

impl ContentKeyRequest {
    fn info(&self) -> Result<ContentKeyRequestInfoPayload, AVPlayerError> {
        request_info(self.ptr)
    }

    pub fn status(&self) -> Result<ContentKeyRequestStatus, AVPlayerError> {
        Ok(ContentKeyRequestStatus::from_raw(self.info()?.status))
    }

    pub fn error_message(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.error_message)
    }

    pub fn identifier(&self) -> Result<Option<ContentKeyIdentifier>, AVPlayerError> {
        Ok(self.info()?.identifier.map(ContentKeyIdentifier::from_payload))
    }

    pub fn initialization_data(&self) -> Result<Option<Vec<u8>>, AVPlayerError> {
        Ok(self.info()?.initialization_data)
    }

    pub fn options(&self) -> Result<ContentKeyRequestOptions, AVPlayerError> {
        Ok(self.info()?.options.unwrap_or_default())
    }

    pub fn can_provide_persistable_content_key(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.can_provide_persistable_content_key)
    }

    pub fn renews_expiring_response_data(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.renews_expiring_response_data)
    }

    pub fn content_key_specifier(&self) -> Result<Option<ContentKeySpecifier>, AVPlayerError> {
        Ok(if self.info()?.has_content_key_specifier {
            copy_content_key_request_specifier(self.ptr)
        } else {
            None
        })
    }

    pub fn content_key(&self) -> Result<Option<ContentKey>, AVPlayerError> {
        Ok(if self.info()?.has_content_key {
            copy_content_key_request_content_key(self.ptr)
        } else {
            None
        })
    }

    pub fn make_streaming_content_key_request_data(
        &self,
        app_identifier: &[u8],
        content_identifier: Option<&[u8]>,
        options: Option<&ContentKeyRequestOptions>,
    ) -> Result<Option<Vec<u8>>, AVPlayerError> {
        request_make_streaming_content_key_request_data(
            self.ptr,
            app_identifier,
            content_identifier,
            options,
        )
    }

    pub fn process_response(&self, response: &ContentKeyResponse) -> Result<(), AVPlayerError> {
        request_process_response(self.ptr, response)
    }

    pub fn process_content_key_response(
        &self,
        fair_play_streaming_key_response_data: impl AsRef<[u8]>,
    ) -> Result<(), AVPlayerError> {
        let response = ContentKeyResponse::fair_play_streaming_key_response_data(
            fair_play_streaming_key_response_data,
        )?;
        self.process_response(&response)
    }

    pub fn process_clear_key_response(
        &self,
        key_data: impl AsRef<[u8]>,
        initialization_vector: Option<&[u8]>,
    ) -> Result<(), AVPlayerError> {
        let response = ContentKeyResponse::clear_key_data(key_data, initialization_vector)?;
        self.process_response(&response)
    }

    pub fn process_authorization_token_response(
        &self,
        authorization_token_data: impl AsRef<[u8]>,
    ) -> Result<(), AVPlayerError> {
        let response = ContentKeyResponse::authorization_token_data(authorization_token_data)?;
        self.process_response(&response)
    }

    pub fn process_content_key_response_error(
        &self,
        message: impl AsRef<str>,
    ) -> Result<(), AVPlayerError> {
        request_process_response_error(self.ptr, message.as_ref())
    }

    pub fn respond_by_requesting_persistable_content_key_request(&self) -> Result<(), AVPlayerError> {
        request_request_persistable_content_key(self.ptr)
    }
}

impl PersistableContentKeyRequest {
    fn info(&self) -> Result<ContentKeyRequestInfoPayload, AVPlayerError> {
        request_info(self.ptr)
    }

    pub fn status(&self) -> Result<ContentKeyRequestStatus, AVPlayerError> {
        Ok(ContentKeyRequestStatus::from_raw(self.info()?.status))
    }

    pub fn error_message(&self) -> Result<Option<String>, AVPlayerError> {
        Ok(self.info()?.error_message)
    }

    pub fn identifier(&self) -> Result<Option<ContentKeyIdentifier>, AVPlayerError> {
        Ok(self.info()?.identifier.map(ContentKeyIdentifier::from_payload))
    }

    pub fn initialization_data(&self) -> Result<Option<Vec<u8>>, AVPlayerError> {
        Ok(self.info()?.initialization_data)
    }

    pub fn options(&self) -> Result<ContentKeyRequestOptions, AVPlayerError> {
        Ok(self.info()?.options.unwrap_or_default())
    }

    pub fn can_provide_persistable_content_key(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.can_provide_persistable_content_key)
    }

    pub fn renews_expiring_response_data(&self) -> Result<bool, AVPlayerError> {
        Ok(self.info()?.renews_expiring_response_data)
    }

    pub fn content_key_specifier(&self) -> Result<Option<ContentKeySpecifier>, AVPlayerError> {
        Ok(if self.info()?.has_content_key_specifier {
            copy_content_key_request_specifier(self.ptr)
        } else {
            None
        })
    }

    pub fn content_key(&self) -> Result<Option<ContentKey>, AVPlayerError> {
        Ok(if self.info()?.has_content_key {
            copy_content_key_request_content_key(self.ptr)
        } else {
            None
        })
    }

    pub fn make_streaming_content_key_request_data(
        &self,
        app_identifier: &[u8],
        content_identifier: Option<&[u8]>,
        options: Option<&ContentKeyRequestOptions>,
    ) -> Result<Option<Vec<u8>>, AVPlayerError> {
        request_make_streaming_content_key_request_data(
            self.ptr,
            app_identifier,
            content_identifier,
            options,
        )
    }

    pub fn process_response(&self, response: &ContentKeyResponse) -> Result<(), AVPlayerError> {
        request_process_response(self.ptr, response)
    }

    pub fn process_content_key_response(
        &self,
        fair_play_streaming_key_response_data: impl AsRef<[u8]>,
    ) -> Result<(), AVPlayerError> {
        let response = ContentKeyResponse::fair_play_streaming_key_response_data(
            fair_play_streaming_key_response_data,
        )?;
        self.process_response(&response)
    }

    pub fn process_clear_key_response(
        &self,
        key_data: impl AsRef<[u8]>,
        initialization_vector: Option<&[u8]>,
    ) -> Result<(), AVPlayerError> {
        let response = ContentKeyResponse::clear_key_data(key_data, initialization_vector)?;
        self.process_response(&response)
    }

    pub fn process_authorization_token_response(
        &self,
        authorization_token_data: impl AsRef<[u8]>,
    ) -> Result<(), AVPlayerError> {
        let response = ContentKeyResponse::authorization_token_data(authorization_token_data)?;
        self.process_response(&response)
    }

    pub fn process_content_key_response_error(
        &self,
        message: impl AsRef<str>,
    ) -> Result<(), AVPlayerError> {
        request_process_response_error(self.ptr, message.as_ref())
    }

    pub fn persistable_content_key_from_key_vendor_response(
        &self,
        key_vendor_response: &[u8],
        options: Option<&ContentKeyRequestOptions>,
    ) -> Result<Vec<u8>, AVPlayerError> {
        let options = maybe_json_cstring(options, "persistable content-key options")?;
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::av_persistable_content_key_request_persistable_content_key_json(
                self.ptr,
                if key_vendor_response.is_empty() {
                    ptr::null()
                } else {
                    key_vendor_response.as_ptr()
                },
                key_vendor_response.len(),
                options.as_ref().map_or(ptr::null(), |value| value.as_ptr()),
                &mut err,
            )
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(parse_json_and_free::<ContentKeyBytesPayload>(json_ptr)?
            .bytes
            .unwrap_or_default())
    }
}

fn request_info(ptr: *mut c_void) -> Result<ContentKeyRequestInfoPayload, AVPlayerError> {
    let mut err: *mut c_char = ptr::null_mut();
    let json_ptr = unsafe { ffi::av_content_key_request_info_json(ptr, &mut err) };
    if json_ptr.is_null() {
        return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
    }
    parse_json_and_free(json_ptr)
}

fn copy_content_key_request_specifier(ptr: *mut c_void) -> Option<ContentKeySpecifier> {
    let ptr = unsafe { ffi::av_content_key_request_copy_content_key_specifier(ptr) };
    if ptr.is_null() {
        None
    } else {
        Some(ContentKeySpecifier { ptr })
    }
}

fn copy_content_key_request_content_key(ptr: *mut c_void) -> Option<ContentKey> {
    let ptr = unsafe { ffi::av_content_key_request_copy_content_key(ptr) };
    if ptr.is_null() {
        None
    } else {
        Some(ContentKey { ptr })
    }
}

fn request_make_streaming_content_key_request_data(
    ptr: *mut c_void,
    app_identifier: &[u8],
    content_identifier: Option<&[u8]>,
    options: Option<&ContentKeyRequestOptions>,
) -> Result<Option<Vec<u8>>, AVPlayerError> {
    let options = maybe_json_cstring(options, "streaming content-key request options")?;
    let content_identifier = content_identifier.unwrap_or_default();
    let mut err: *mut c_char = ptr::null_mut();
    let json_ptr = unsafe {
        ffi::av_content_key_request_make_streaming_content_key_request_data_json(
            ptr,
            if app_identifier.is_empty() {
                ptr::null()
            } else {
                app_identifier.as_ptr()
            },
            app_identifier.len(),
            if content_identifier.is_empty() {
                ptr::null()
            } else {
                content_identifier.as_ptr()
            },
            content_identifier.len(),
            options.as_ref().map_or(ptr::null(), |value| value.as_ptr()),
            &mut err,
        )
    };
    if json_ptr.is_null() {
        return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
    }
    Ok(parse_json_and_free::<ContentKeyBytesPayload>(json_ptr)?.bytes)
}

fn request_process_response(
    request_ptr: *mut c_void,
    response: &ContentKeyResponse,
) -> Result<(), AVPlayerError> {
    let mut err: *mut c_char = ptr::null_mut();
    let status = unsafe {
        ffi::av_content_key_request_process_content_key_response(
            request_ptr,
            response.ptr,
            &mut err,
        )
    };
    if status != ffi::status::OK {
        return Err(unsafe { from_swift(status, err) });
    }
    Ok(())
}

fn request_process_response_error(
    request_ptr: *mut c_void,
    message: &str,
) -> Result<(), AVPlayerError> {
    let message = CString::new(message).map_err(|error| {
        AVPlayerError::InvalidArgument(format!(
            "content-key response error message contains NUL byte: {error}"
        ))
    })?;
    let mut err: *mut c_char = ptr::null_mut();
    let status = unsafe {
        ffi::av_content_key_request_process_content_key_response_error(
            request_ptr,
            message.as_ptr(),
            &mut err,
        )
    };
    if status != ffi::status::OK {
        return Err(unsafe { from_swift(status, err) });
    }
    Ok(())
}

fn request_request_persistable_content_key(request_ptr: *mut c_void) -> Result<(), AVPlayerError> {
    let mut err: *mut c_char = ptr::null_mut();
    let status = unsafe {
        ffi::av_content_key_request_request_persistable_content_key(request_ptr, &mut err)
    };
    if status != ffi::status::OK {
        return Err(unsafe { from_swift(status, err) });
    }
    Ok(())
}

fn catch_cb_panic_bool<F: FnOnce() -> bool>(site: &str, f: F) -> bool {
    match std::panic::catch_unwind(AssertUnwindSafe(f)) {
        Ok(value) => value,
        Err(payload) => {
            let msg = payload.downcast_ref::<&str>().copied().unwrap_or_else(|| {
                payload
                    .downcast_ref::<String>()
                    .map_or("<non-string panic>", String::as_str)
            });
            eprintln!("avplayer: panic in {site} caught at C ABI boundary: {msg}");
            false
        }
    }
}

fn content_key_session_event_from_payload(
    payload: ContentKeySessionEventPayload,
) -> Option<ContentKeySessionEvent> {
    match payload.event.as_str() {
        "requested" => Some(ContentKeySessionEvent::Requested(ContentKeyRequest {
            ptr: ptr_from_u64(payload.request_ptr?),
        })),
        "renewing" => Some(ContentKeySessionEvent::Renewing(ContentKeyRequest {
            ptr: ptr_from_u64(payload.request_ptr?),
        })),
        "persistable" => Some(ContentKeySessionEvent::Persistable(
            PersistableContentKeyRequest {
                ptr: ptr_from_u64(payload.request_ptr?),
            },
        )),
        "updated_persistable_content_key" => {
            Some(ContentKeySessionEvent::UpdatedPersistableContentKey {
                persistable_content_key: payload.persistable_content_key.unwrap_or_default(),
                key_identifier: payload.key_identifier.map(ContentKeyIdentifier::from_payload),
            })
        }
        "failed" => Some(ContentKeySessionEvent::Failed {
            request: ContentKeyRequest {
                ptr: ptr_from_u64(payload.request_ptr?),
            },
            error_message: payload.error_message.unwrap_or_default(),
        }),
        "retry_requested" => Some(ContentKeySessionEvent::RetryRequested {
            request: ContentKeyRequest {
                ptr: ptr_from_u64(payload.request_ptr?),
            },
            reason: ContentKeyRequestRetryReason::from_raw(
                payload.retry_reason.as_deref().unwrap_or_default(),
            ),
        }),
        "succeeded" => Some(ContentKeySessionEvent::Succeeded(ContentKeyRequest {
            ptr: ptr_from_u64(payload.request_ptr?),
        })),
        "content_protection_session_identifier_did_change" => {
            Some(ContentKeySessionEvent::ContentProtectionSessionIdentifierDidChange)
        }
        "expired_session_report_generated" => {
            Some(ContentKeySessionEvent::ExpiredSessionReportGenerated)
        }
        "external_protection_status_did_change" => Some(
            ContentKeySessionEvent::ExternalProtectionStatusDidChange(ContentKey {
                ptr: ptr_from_u64(payload.content_key_ptr?),
            }),
        ),
        "requested_collection" => {
            let mut requests = payload
                .key_request_ptrs?
                .into_iter()
                .map(|raw| ContentKeyRequest {
                    ptr: ptr_from_u64(raw),
                })
                .collect::<Vec<_>>();
            if requests.len() == 1 && payload.initialization_data.is_none() {
                return requests.pop().map(ContentKeySessionEvent::Requested);
            }
            Some(ContentKeySessionEvent::RequestedCollection {
                requests,
                initialization_data: payload.initialization_data,
            })
        }
        _ => None,
    }
}

fn ptr_from_u64(raw: u64) -> *mut c_void {
    usize::try_from(raw).expect("content-key event pointer fits in usize") as *mut c_void
}

unsafe extern "C" fn content_key_session_event_trampoline(
    userdata: *mut c_void,
    payload_json: *const c_char,
) -> bool {
    if userdata.is_null() || payload_json.is_null() {
        return false;
    }

    let callback = unsafe { &*userdata.cast::<ContentKeySessionObserverState>() };
    let Ok(payload) = unsafe { CStr::from_ptr(payload_json) }.to_str() else {
        return false;
    };
    let Ok(payload) = serde_json::from_str::<ContentKeySessionEventPayload>(payload) else {
        return false;
    };
    let Some(event) = content_key_session_event_from_payload(payload) else {
        return false;
    };

    catch_cb_panic_bool("content_key_session_event_trampoline", || {
        (callback.callback)(event)
    })
}

unsafe extern "C" fn content_key_session_observer_drop(userdata: *mut c_void) {
    if !userdata.is_null() {
        drop(unsafe { Box::from_raw(userdata.cast::<ContentKeySessionObserverState>()) });
    }
}

// SAFETY: AVFoundation content-key handles and observer tokens may be sent
// across thread boundaries; the underlying objects manage their own internal
// synchronization and lifetime once retained.
unsafe impl Send for ContentKeySession {}
unsafe impl Send for ContentKeyRequest {}
unsafe impl Send for PersistableContentKeyRequest {}
unsafe impl Send for ContentKeyResponse {}
unsafe impl Send for ContentKeySpecifier {}
unsafe impl Send for ContentKey {}
unsafe impl Send for ContentKeySessionObserver {}
