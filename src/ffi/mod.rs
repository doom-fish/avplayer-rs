//! Raw FFI declarations matching `swift-bridge/Sources/AVPlayerBridge`.

#![allow(missing_docs)]

use core::ffi::{c_char, c_void};

pub type JsonCallback = unsafe extern "C" fn(userdata: *mut c_void, payload_json: *const c_char);
pub type PeriodicTimeCallback =
    unsafe extern "C" fn(userdata: *mut c_void, value: i64, timescale: i32, kind: i32);
pub type SimpleCallback = unsafe extern "C" fn(userdata: *mut c_void);
pub type DropCallback = unsafe extern "C" fn(userdata: *mut c_void);

extern "C" {
    pub fn avp_string_free(s: *mut c_char);

    pub fn av_url_asset_create(
        url: *const c_char,
        is_file_url: bool,
        prefer_precise_duration: bool,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_asset_release(asset: *mut c_void);
    pub fn av_asset_info_json(asset: *mut c_void, out_error_message: *mut *mut c_char) -> *mut c_char;
    pub fn av_asset_load_values_json(
        asset: *mut c_void,
        keys_json: *const c_char,
        timeout_seconds: i32,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_asset_status_of_value(
        asset: *mut c_void,
        key: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_asset_track_count(asset: *mut c_void) -> i32;
    pub fn av_asset_copy_track_at_index(asset: *mut c_void, index: i32) -> *mut c_void;
    pub fn av_asset_track_release(track: *mut c_void);
    pub fn av_asset_track_info_json(track: *mut c_void, out_error_message: *mut *mut c_char) -> *mut c_char;

    pub fn av_player_item_create_with_url(
        url: *const c_char,
        is_file_url: bool,
        asset_keys_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_player_item_create_with_asset(
        asset: *mut c_void,
        asset_keys_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_player_item_release(item: *mut c_void);
    pub fn av_player_item_info_json(
        item: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_player_item_add_observer(
        item: *mut c_void,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_player_item_observer_release(observer: *mut c_void);

    pub fn av_player_create_with_url(
        url: *const c_char,
        is_file_url: bool,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_player_create_with_asset(
        asset: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_player_create_with_item(
        item: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_player_release(player: *mut c_void);
    pub fn av_player_info_json(player: *mut c_void, out_error_message: *mut *mut c_char) -> *mut c_char;
    pub fn av_player_play(player: *mut c_void);
    pub fn av_player_pause(player: *mut c_void);
    pub fn av_player_set_rate(player: *mut c_void, rate: f32);
    pub fn av_player_seek(
        player: *mut c_void,
        value: i64,
        timescale: i32,
        kind: i32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_player_copy_current_item(player: *mut c_void) -> *mut c_void;
    pub fn av_player_add_periodic_time_observer(
        player: *mut c_void,
        interval_value: i64,
        interval_timescale: i32,
        interval_kind: i32,
        queue_label: *const c_char,
        callback: Option<PeriodicTimeCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_player_add_boundary_time_observer(
        player: *mut c_void,
        times_json: *const c_char,
        queue_label: *const c_char,
        callback: Option<SimpleCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_player_time_observer_release(observer: *mut c_void);

    pub fn av_reader_create(
        asset: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_reader_release(reader: *mut c_void);
    pub fn av_reader_info_json(reader: *mut c_void, out_error_message: *mut *mut c_char) -> *mut c_char;
    pub fn av_reader_set_time_range(
        reader: *mut c_void,
        start_value: i64,
        start_timescale: i32,
        start_kind: i32,
        duration_value: i64,
        duration_timescale: i32,
        duration_kind: i32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_reader_start(reader: *mut c_void, out_error_message: *mut *mut c_char) -> i32;
    pub fn av_reader_cancel(reader: *mut c_void);
    pub fn av_reader_can_add_output(reader: *mut c_void, output: *mut c_void) -> bool;
    pub fn av_reader_add_output(
        reader: *mut c_void,
        output: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;

    pub fn av_reader_track_output_create_video(
        track: *mut c_void,
        settings_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_reader_track_output_create_audio(
        track: *mut c_void,
        settings_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_reader_track_output_create_passthrough(
        track: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_reader_audio_mix_output_create(
        tracks: *const *mut c_void,
        count: usize,
        settings_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_reader_video_composition_output_create(
        tracks: *const *mut c_void,
        count: usize,
        settings_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_reader_output_release(output: *mut c_void);
    pub fn av_reader_output_set_always_copies_sample_data(output: *mut c_void, always_copies: bool);
    pub fn av_reader_output_media_type(output: *mut c_void) -> *mut c_char;
    pub fn av_reader_output_copy_next_sample_buffer(output: *mut c_void) -> *mut c_void;
    pub fn av_reader_output_copy_next_video_pixel_buffer(output: *mut c_void) -> *mut c_void;
}

pub mod status {
    pub const OK: i32 = 0;
    pub const INVALID_ARGUMENT: i32 = -1;
    pub const ASSET_CREATE_FAILED: i32 = -2;
    pub const PLAYER_CREATE_FAILED: i32 = -3;
    pub const READER_CREATE_FAILED: i32 = -4;
    pub const OPERATION_FAILED: i32 = -5;
    pub const OBSERVER_FAILED: i32 = -6;
    pub const LOAD_FAILED: i32 = -7;
}
