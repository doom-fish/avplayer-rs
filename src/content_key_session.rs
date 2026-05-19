#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use std::ffi::CString;
use std::path::Path;

use serde::Deserialize;

use crate::asset::UrlAsset;
use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::util::{parse_json_and_free, to_cstring};

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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContentKeySessionInfoPayload {
    key_system: String,
    storage_url: Option<String>,
    content_protection_session_identifier_base64: Option<String>,
    recipient_count: usize,
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

    pub fn expire(&self) {
        unsafe { ffi::av_content_key_session_expire(self.ptr) };
    }
}
