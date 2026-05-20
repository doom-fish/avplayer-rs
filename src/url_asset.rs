#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use std::path::Path;

use crate::asset::UrlAsset;
use crate::error::AVPlayerError;

/// Mirrors the `AVPlayer` framework counterpart for `UrlAssetOptions`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UrlAssetOptions {
    prefer_precise_duration_and_timing: bool,
}

impl Default for UrlAssetOptions {
    fn default() -> Self {
        Self {
            prefer_precise_duration_and_timing: true,
        }
    }
}

impl UrlAssetOptions {
    /// Mirrors the `AVPlayer` framework constant `fn`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            prefer_precise_duration_and_timing: true,
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
    pub const fn prefers_precise_duration_and_timing(self) -> bool {
        self.prefer_precise_duration_and_timing
    }
}

impl UrlAsset {
    /// Calls the `AVPlayer` framework counterpart for `from_file_path_with_options`.
    pub fn from_file_path_with_options(
        path: impl AsRef<Path>,
        options: UrlAssetOptions,
    ) -> Result<Self, AVPlayerError> {
        let path = path
            .as_ref()
            .to_str()
            .ok_or_else(|| AVPlayerError::InvalidArgument("path is not valid UTF-8".into()))?;
        Self::from_raw_url_with_options(path, true, options.prefers_precise_duration_and_timing())
    }

    /// Calls the `AVPlayer` framework counterpart for `from_remote_url_with_options`.
    pub fn from_remote_url_with_options(
        url: impl AsRef<str>,
        options: UrlAssetOptions,
    ) -> Result<Self, AVPlayerError> {
        Self::from_raw_url_with_options(
            url.as_ref(),
            false,
            options.prefers_precise_duration_and_timing(),
        )
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
