#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::c_char;
use core::ptr;
use std::path::Path;

use serde::Serialize;

use crate::asset::{Asset, UrlAsset};
use crate::error::{from_swift, AVPlayerError};
use crate::ffi;
use crate::util::{json_cstring, to_cstring};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum UrlRequestAttribution {
    Developer,
    User,
}

impl UrlRequestAttribution {
    const fn as_raw(self) -> u64 {
        match self {
            Self::Developer => 0,
            Self::User => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UrlAssetHttpCookie {
    name: String,
    value: String,
    domain: String,
    path: String,
    secure: Option<bool>,
}

impl UrlAssetHttpCookie {
    pub fn new(
        name: impl Into<String>,
        value: impl Into<String>,
        domain: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            domain: domain.into(),
            path: path.into(),
            secure: None,
        }
    }

    #[must_use]
    pub const fn secure(mut self, secure: bool) -> Self {
        self.secure = Some(secure);
        self
    }
}

/// Mirrors the `AVPlayer` framework counterpart for `UrlAssetOptions`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UrlAssetOptions {
    prefer_precise_duration_and_timing: bool,
    override_mime_type: Option<String>,
    reference_restrictions: Option<u64>,
    http_cookies: Option<Vec<UrlAssetHttpCookie>>,
    allows_cellular_access: Option<bool>,
    allows_expensive_network_access: Option<bool>,
    allows_constrained_network_access: Option<bool>,
    should_support_alias_data_references: Option<bool>,
    url_request_attribution: Option<u64>,
    http_user_agent: Option<String>,
    primary_session_identifier: Option<String>,
    should_parse_external_spherical_tags: Option<bool>,
}

impl Default for UrlAssetOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl UrlAssetOptions {
    /// Mirrors the `AVPlayer` framework constant `fn`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            prefer_precise_duration_and_timing: true,
            override_mime_type: None,
            reference_restrictions: None,
            http_cookies: None,
            allows_cellular_access: None,
            allows_expensive_network_access: None,
            allows_constrained_network_access: None,
            should_support_alias_data_references: None,
            url_request_attribution: None,
            http_user_agent: None,
            primary_session_identifier: None,
            should_parse_external_spherical_tags: None,
        }
    }

    /// Mirrors the `AVPlayer` framework constant `fn`.
    #[must_use]
    pub const fn prefer_precise_duration_and_timing(mut self, prefer: bool) -> Self {
        self.prefer_precise_duration_and_timing = prefer;
        self
    }

    /// Mirrors the `AVPlayer` framework constant `fn`.
    #[must_use]
    pub const fn prefers_precise_duration_and_timing(&self) -> bool {
        self.prefer_precise_duration_and_timing
    }

    #[must_use]
    pub fn override_mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.override_mime_type = Some(mime_type.into());
        self
    }

    #[must_use]
    pub const fn reference_restrictions(mut self, restrictions: u64) -> Self {
        self.reference_restrictions = Some(restrictions);
        self
    }

    #[must_use]
    pub fn http_cookies(mut self, cookies: impl IntoIterator<Item = UrlAssetHttpCookie>) -> Self {
        self.http_cookies = Some(cookies.into_iter().collect());
        self
    }

    #[must_use]
    pub const fn allows_cellular_access(mut self, allows: bool) -> Self {
        self.allows_cellular_access = Some(allows);
        self
    }

    #[must_use]
    pub const fn allows_expensive_network_access(mut self, allows: bool) -> Self {
        self.allows_expensive_network_access = Some(allows);
        self
    }

    #[must_use]
    pub const fn allows_constrained_network_access(mut self, allows: bool) -> Self {
        self.allows_constrained_network_access = Some(allows);
        self
    }

    #[must_use]
    pub const fn should_support_alias_data_references(mut self, supports: bool) -> Self {
        self.should_support_alias_data_references = Some(supports);
        self
    }

    #[must_use]
    pub const fn url_request_attribution(mut self, attribution: UrlRequestAttribution) -> Self {
        self.url_request_attribution = Some(attribution.as_raw());
        self
    }

    #[must_use]
    pub fn http_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.http_user_agent = Some(user_agent.into());
        self
    }

    #[must_use]
    pub fn primary_session_identifier(mut self, identifier: impl Into<String>) -> Self {
        self.primary_session_identifier = Some(identifier.into());
        self
    }

    #[must_use]
    pub const fn should_parse_external_spherical_tags(mut self, parse: bool) -> Self {
        self.should_parse_external_spherical_tags = Some(parse);
        self
    }
}

impl UrlAsset {
    /// Calls the `AVPlayer` framework counterpart for `from_file_path_with_options`.
    pub fn from_file_path_with_options(
        path: impl AsRef<Path>,
        options: &UrlAssetOptions,
    ) -> Result<Self, AVPlayerError> {
        let path = path
            .as_ref()
            .to_str()
            .ok_or_else(|| AVPlayerError::InvalidArgument("path is not valid UTF-8".into()))?;
        Self::from_url_with_options(path, true, options)
    }

    /// Calls the `AVPlayer` framework counterpart for `from_remote_url_with_options`.
    pub fn from_remote_url_with_options(
        url: impl AsRef<str>,
        options: &UrlAssetOptions,
    ) -> Result<Self, AVPlayerError> {
        Self::from_url_with_options(url.as_ref(), false, options)
    }

    fn from_url_with_options(
        url: &str,
        is_file_url: bool,
        options: &UrlAssetOptions,
    ) -> Result<Self, AVPlayerError> {
        let url = to_cstring(url, "URL")?;
        let options = json_cstring(options, "URL asset options")?;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::av_url_asset_create_with_options_json(
                url.as_ptr(),
                is_file_url,
                options.as_ptr(),
                &raw mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::ASSET_CREATE_FAILED, err) });
        }
        Ok(Self {
            asset: Asset { ptr },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_options_default_to_precise_duration_and_timing() {
        assert!(UrlAssetOptions::new().prefers_precise_duration_and_timing());
    }

    #[test]
    fn default_matches_new_options() {
        assert_eq!(UrlAssetOptions::default(), UrlAssetOptions::new());
    }

    #[test]
    fn builder_can_disable_precise_duration_and_timing() {
        let options = UrlAssetOptions::new().prefer_precise_duration_and_timing(false);

        assert!(!options.prefers_precise_duration_and_timing());
    }

    #[test]
    fn builder_can_reenable_precise_duration_and_timing() {
        let options = UrlAssetOptions::new()
            .prefer_precise_duration_and_timing(false)
            .prefer_precise_duration_and_timing(true);

        assert!(options.prefers_precise_duration_and_timing());
    }
}
