//! Async API for `avplayer` — executor-agnostic `Future` wrappers for
//! `AVFoundation`'s `async throws` and completion-handler surfaces.
//!
//! Gated behind the **`async`** cargo feature.
//!
//! ## Available types
//!
//! | Type | Description |
//! |------|-------------|
//! | [`AsyncAsset`] | Async property / track loading (`AVAsset.load(...)`) |
//! | [`AsyncPlayerItem`] | Async seek (`AVPlayerItem.seek(to:completionHandler:)`) |
//! | [`AsyncPlayer`] | Async seek + preroll (`AVPlayer.seek / preroll`) |
//!
//! ## Note on KVO / periodic observers
//!
//! KVO-based observation (`PlayerItemObserver`, `PeriodicTimeObserver`, etc.)
//! fires multiple times and therefore belongs to a Stream-based "Tier 2" async
//! pattern — **not** this module.
//!
//! ## Example
//!
//! ```rust,no_run
//! use avplayer::{UrlAsset, async_api::AsyncAsset};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     pollster::block_on(async {
//!         let asset = UrlAsset::from_remote_url("https://example.com/sample.mp4")?;
//!         let props = AsyncAsset::new(asset.as_asset()).load_properties().await?;
//!         println!("duration: {:?}  playable: {}", props.duration, props.is_playable);
//!         Ok(())
//!     })
//! }
//! ```

#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use std::ffi::{c_void, CStr};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use doom_fish_utils::completion::{error_from_cstr, AsyncCompletion, AsyncCompletionFuture};
use serde::Deserialize;

use crate::asset::{Asset, MediaType, Size};
use crate::error::AVPlayerError;
use crate::ffi;
use crate::metadata::MetadataItem;
use crate::player::{Player, PlayerItem};
use crate::time::Time;

// ── JSON payloads (private) ───────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AsyncAssetPropertiesPayload {
    duration: Time,
    metadata: Vec<MetadataItem>,
    is_playable: bool,
    is_exportable: bool,
    has_protected_content: bool,
    preferred_rate: f32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TrackInfoPayload {
    track_id: i32,
    media_type: String,
    natural_size: Size,
    nominal_frame_rate: String,
    estimated_data_rate: String,
}

// ── Public result structs ─────────────────────────────────────────────────────

/// Properties loaded asynchronously from `AVAsset.load(...)`.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct AssetProperties {
    /// Asset duration.
    pub duration: Time,
    /// Static metadata items.
    pub metadata: Vec<MetadataItem>,
    /// Whether the asset can be used for playback.
    pub is_playable: bool,
    /// Whether the asset can be exported.
    pub is_exportable: bool,
    /// Whether the asset contains protected (`FairPlay`) content.
    pub has_protected_content: bool,
    /// Natural playback rate encoded in the asset.
    pub preferred_rate: f32,
}

/// Track properties loaded asynchronously.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct TrackProperties {
    /// Persistent track identifier.
    pub track_id: i32,
    /// Track media type.
    pub media_type: MediaType,
    /// Natural presentation size.
    pub natural_size: Size,
    /// Nominal frame rate (for video tracks; `None` if unavailable or non-numeric).
    pub nominal_frame_rate: Option<f32>,
    /// Estimated data rate in bps (`None` if unavailable or non-numeric).
    pub estimated_data_rate: Option<f32>,
}

impl TrackProperties {
    fn from_payload(p: &TrackInfoPayload) -> Self {
        Self {
            track_id: p.track_id,
            media_type: MediaType::from_raw(&p.media_type),
            natural_size: p.natural_size,
            nominal_frame_rate: p.nominal_frame_rate.parse().ok(),
            estimated_data_rate: p.estimated_data_rate.parse().ok(),
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Map a `String` (error message from completion) to `AVPlayerError::LoadFailed`.
const fn bridge_err(msg: String) -> AVPlayerError {
    AVPlayerError::LoadFailed(msg)
}

// ── AssetProperties future ────────────────────────────────────────────────────

extern "C" fn asset_properties_cb(
    result: *const c_void,
    error: *const i8,
    ctx: *mut c_void,
) {
    if !error.is_null() {
        let msg = unsafe { error_from_cstr(error) };
        unsafe { AsyncCompletion::<String>::complete_err(ctx, msg) };
    } else if !result.is_null() {
        let s = unsafe {
            CStr::from_ptr(result.cast())
                .to_string_lossy()
                .into_owned()
        };
        unsafe { ffi::avp_string_free(result as *mut _) };
        unsafe { AsyncCompletion::complete_ok(ctx, s) };
    } else {
        unsafe {
            AsyncCompletion::<String>::complete_err(ctx, "bridge returned null result".into());
        };
    }
}

/// Future returned by [`AsyncAsset::load_properties`].
pub struct AssetPropertiesFuture {
    inner: AsyncCompletionFuture<String>,
}

impl std::fmt::Debug for AssetPropertiesFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssetPropertiesFuture")
            .finish_non_exhaustive()
    }
}

impl Future for AssetPropertiesFuture {
    type Output = Result<AssetProperties, AVPlayerError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(|r| {
            let json = r.map_err(bridge_err)?;
            let p = serde_json::from_str::<AsyncAssetPropertiesPayload>(&json).map_err(|e| {
                AVPlayerError::OperationFailed(format!("async bridge JSON decode failed: {e}"))
            })?;
            Ok(AssetProperties {
                duration: p.duration,
                metadata: p.metadata,
                is_playable: p.is_playable,
                is_exportable: p.is_exportable,
                has_protected_content: p.has_protected_content,
                preferred_rate: p.preferred_rate,
            })
        })
    }
}

// ── Tracks future ─────────────────────────────────────────────────────────────

extern "C" fn asset_tracks_cb(
    result: *const c_void,
    error: *const i8,
    ctx: *mut c_void,
) {
    asset_properties_cb(result, error, ctx); // same JSON-string pattern
}

/// Future returned by [`AsyncAsset::load_tracks`] and [`AsyncAsset::load_tracks_with_media_type`].
pub struct AssetTracksFuture {
    inner: AsyncCompletionFuture<String>,
}

impl std::fmt::Debug for AssetTracksFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssetTracksFuture")
            .finish_non_exhaustive()
    }
}

impl Future for AssetTracksFuture {
    type Output = Result<Vec<TrackProperties>, AVPlayerError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(|r| {
            let json = r.map_err(bridge_err)?;
            let payloads =
                serde_json::from_str::<Vec<TrackInfoPayload>>(&json).map_err(|e| {
                    AVPlayerError::OperationFailed(format!(
                        "async bridge JSON decode failed: {e}"
                    ))
                })?;
            Ok(payloads.into_iter().map(|p| TrackProperties::from_payload(&p)).collect())
        })
    }
}

// ── TrackById future ──────────────────────────────────────────────────────────

extern "C" fn asset_track_by_id_cb(
    result: *const c_void,
    error: *const i8,
    ctx: *mut c_void,
) {
    asset_properties_cb(result, error, ctx); // same JSON-string pattern
}

/// Future returned by [`AsyncAsset::load_track_with_id`].
pub struct AssetTrackByIdFuture {
    inner: AsyncCompletionFuture<String>,
}

impl std::fmt::Debug for AssetTrackByIdFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssetTrackByIdFuture")
            .finish_non_exhaustive()
    }
}

impl Future for AssetTrackByIdFuture {
    type Output = Result<Option<TrackProperties>, AVPlayerError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(|r| {
            let json = r.map_err(bridge_err)?;
            if json.trim() == "null" {
                return Ok(None);
            }
            let payload =
                serde_json::from_str::<TrackInfoPayload>(&json).map_err(|e| {
                    AVPlayerError::OperationFailed(format!(
                        "async bridge JSON decode failed: {e}"
                    ))
                })?;
            Ok(Some(TrackProperties::from_payload(&payload)))
        })
    }
}

// ── Bool (seek / preroll) future ──────────────────────────────────────────────

extern "C" fn bool_completion_cb(
    result: *const c_void,
    error: *const i8,
    ctx: *mut c_void,
) {
    if error.is_null() {
        // result is UnsafeRawPointer(bitPattern: 1) for true, nil for false
        let finished = !result.is_null();
        unsafe { AsyncCompletion::complete_ok(ctx, finished) };
    } else {
        let msg = unsafe { error_from_cstr(error) };
        unsafe { AsyncCompletion::<bool>::complete_err(ctx, msg) };
    }
}

/// Future returned by [`AsyncPlayerItem::seek`].
pub struct PlayerItemSeekFuture {
    inner: AsyncCompletionFuture<bool>,
}

impl std::fmt::Debug for PlayerItemSeekFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlayerItemSeekFuture")
            .finish_non_exhaustive()
    }
}

impl Future for PlayerItemSeekFuture {
    type Output = Result<bool, AVPlayerError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner)
            .poll(cx)
            .map(|r| r.map_err(bridge_err))
    }
}

/// Future returned by [`AsyncPlayer::seek`].
pub struct PlayerSeekFuture {
    inner: AsyncCompletionFuture<bool>,
}

impl std::fmt::Debug for PlayerSeekFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlayerSeekFuture")
            .finish_non_exhaustive()
    }
}

impl Future for PlayerSeekFuture {
    type Output = Result<bool, AVPlayerError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner)
            .poll(cx)
            .map(|r| r.map_err(bridge_err))
    }
}

/// Future returned by [`AsyncPlayer::preroll`].
pub struct PlayerPrerollFuture {
    inner: AsyncCompletionFuture<bool>,
}

impl std::fmt::Debug for PlayerPrerollFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlayerPrerollFuture")
            .finish_non_exhaustive()
    }
}

impl Future for PlayerPrerollFuture {
    type Output = Result<bool, AVPlayerError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner)
            .poll(cx)
            .map(|r| r.map_err(bridge_err))
    }
}

// ── Async wrappers ────────────────────────────────────────────────────────────

/// Async accessor for [`Asset`].
///
/// Wraps the `AVAsset.load(...)` `async throws` family and the modern
/// `AVAsset.loadTracks` / `loadTrack(withTrackID:)` helpers (macOS 12+).
///
/// KVO-based observation belongs to the Tier 2 stream pattern and is **not**
/// included here.
pub struct AsyncAsset<'a> {
    asset: &'a Asset,
}

impl std::fmt::Debug for AsyncAsset<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncAsset").finish_non_exhaustive()
    }
}

impl<'a> AsyncAsset<'a> {
    /// Wrap a borrowed [`Asset`].
    pub const fn new(asset: &'a Asset) -> Self {
        Self { asset }
    }

    /// Load `duration`, `metadata`, `isPlayable`, `isExportable`,
    /// `hasProtectedContent`, and `preferredRate` concurrently via
    /// `AVAsset.load(...)`.
    pub fn load_properties(&self) -> AssetPropertiesFuture {
        let (future, ctx) = AsyncCompletion::create();
        unsafe { ffi::avp_asset_load_properties_async(self.asset.ptr, asset_properties_cb, ctx) };
        AssetPropertiesFuture { inner: future }
    }

    /// Load all tracks via `AVAsset.load(.tracks)`.
    pub fn load_tracks(&self) -> AssetTracksFuture {
        let (future, ctx) = AsyncCompletion::create();
        unsafe { ffi::avp_asset_load_tracks_async(self.asset.ptr, asset_tracks_cb, ctx) };
        AssetTracksFuture { inner: future }
    }

    /// Load tracks filtered by media type via `AVAsset.loadTracks(withMediaType:)`.
    ///
    /// `media_type` should be one of `"audio"`, `"video"`, `"text"`, etc.
    pub fn load_tracks_with_media_type(&self, media_type: &str) -> AssetTracksFuture {
        use std::ffi::CString;
        let mt = CString::new(media_type).unwrap_or_default();
        let (future, ctx) = AsyncCompletion::create();
        unsafe {
            ffi::avp_asset_load_tracks_with_media_type_async(
                self.asset.ptr,
                mt.as_ptr(),
                asset_tracks_cb,
                ctx,
            );
        };
        AssetTracksFuture { inner: future }
    }

    /// Load a single track by persistent track ID via
    /// `AVAsset.loadTrack(withTrackID:)`.
    ///
    /// Returns `Ok(None)` when no track with the given ID exists.
    pub fn load_track_with_id(&self, track_id: i32) -> AssetTrackByIdFuture {
        let (future, ctx) = AsyncCompletion::create();
        unsafe {
            ffi::avp_asset_load_track_with_id_async(
                self.asset.ptr,
                track_id,
                asset_track_by_id_cb,
                ctx,
            );
        };
        AssetTrackByIdFuture { inner: future }
    }
}

/// Async accessor for [`PlayerItem`].
///
/// Wraps `AVPlayerItem.seek(to:completionHandler:)`.
pub struct AsyncPlayerItem<'a> {
    item: &'a PlayerItem,
}

impl std::fmt::Debug for AsyncPlayerItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncPlayerItem").finish_non_exhaustive()
    }
}

impl<'a> AsyncPlayerItem<'a> {
    /// Wrap a borrowed [`PlayerItem`].
    pub const fn new(item: &'a PlayerItem) -> Self {
        Self { item }
    }

    /// Seek the player item to `time`.
    ///
    /// Returns `Ok(true)` when the seek completed to the requested time,
    /// `Ok(false)` when it was interrupted by another seek.
    pub fn seek(&self, time: Time) -> PlayerItemSeekFuture {
        let (future, ctx) = AsyncCompletion::create();
        let (value, timescale, kind) = time.to_raw();
        unsafe {
            ffi::avp_player_item_seek_async(self.item.ptr, value, timescale, kind, bool_completion_cb, ctx);
        };
        PlayerItemSeekFuture { inner: future }
    }
}

/// Async accessor for [`Player`].
///
/// Wraps:
/// * `AVPlayer.seek(to:completionHandler:)`
/// * `AVPlayer.preroll(atRate:completionHandler:)` (deprecated API, still functional)
pub struct AsyncPlayer<'a> {
    player: &'a Player,
}

impl std::fmt::Debug for AsyncPlayer<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncPlayer").finish_non_exhaustive()
    }
}

impl<'a> AsyncPlayer<'a> {
    /// Wrap a borrowed [`Player`].
    pub const fn new(player: &'a Player) -> Self {
        Self { player }
    }

    /// Seek the player to `time`.
    ///
    /// Returns `Ok(true)` when the seek completed to the requested time,
    /// `Ok(false)` when it was interrupted by another seek.
    pub fn seek(&self, time: Time) -> PlayerSeekFuture {
        let (future, ctx) = AsyncCompletion::create();
        let (value, timescale, kind) = time.to_raw();
        unsafe {
            ffi::avp_player_seek_async(self.player.ptr, value, timescale, kind, bool_completion_cb, ctx);
        };
        PlayerSeekFuture { inner: future }
    }

    /// Preroll the player at `rate`.
    ///
    /// Returns `Ok(true)` when the preroll succeeded, `Ok(false)` when it was
    /// cancelled (e.g. by a rate change or item replacement).
    ///
    /// Note: `AVPlayer.preroll(atRate:completionHandler:)` is deprecated in
    /// macOS 26+ but remains functional on all supported platforms.
    pub fn preroll(&self, rate: f32) -> PlayerPrerollFuture {
        let (future, ctx) = AsyncCompletion::create();
        unsafe {
            ffi::avp_player_preroll_async(self.player.ptr, rate, bool_completion_cb, ctx);
        };
        PlayerPrerollFuture { inner: future }
    }
}
