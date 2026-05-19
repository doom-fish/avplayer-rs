//! Raw FFI declarations matching `swift-bridge/Sources/AVPlayerBridge`.

#![allow(missing_docs)]

use core::ffi::{c_char, c_void};

/// Re-exports the `AVPlayer` framework surface for this item.
pub use doom_fish_utils::ffi_callbacks::{DropCallback, SimpleCallback};

/// Mirrors the `AVPlayer` framework counterpart for `JsonCallback`.
pub type JsonCallback = unsafe extern "C" fn(userdata: *mut c_void, payload_json: *const c_char);
/// Mirrors the `AVPlayer` framework counterpart for `PeriodicTimeCallback`.
pub type PeriodicTimeCallback =
    unsafe extern "C" fn(userdata: *mut c_void, value: i64, timescale: i32, kind: i32);

extern "C" {
    /// Calls the `AVPlayer` framework counterpart for `avp_string_free`.
    pub fn avp_string_free(s: *mut c_char);

    /// Calls the `AVPlayer` framework counterpart for `av_url_asset_create`.
    pub fn av_url_asset_create(
        url: *const c_char,
        is_file_url: bool,
        prefer_precise_duration: bool,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_asset_release`.
    pub fn av_asset_release(asset: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_asset_info_json`.
    pub fn av_asset_info_json(
        asset: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_asset_extra_info_json`.
    pub fn av_asset_extra_info_json(
        asset: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_asset_cancel_loading`.
    pub fn av_asset_cancel_loading(asset: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_asset_load_values_json`.
    pub fn av_asset_load_values_json(
        asset: *mut c_void,
        keys_json: *const c_char,
        timeout_seconds: i32,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_asset_status_of_value`.
    pub fn av_asset_status_of_value(
        asset: *mut c_void,
        key: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_asset_track_count`.
    pub fn av_asset_track_count(asset: *mut c_void) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_asset_copy_track_at_index`.
    pub fn av_asset_copy_track_at_index(asset: *mut c_void, index: i32) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_asset_track_release`.
    pub fn av_asset_track_release(track: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_asset_track_info_json`.
    pub fn av_asset_track_info_json(
        track: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_asset_track_extra_info_json`.
    pub fn av_asset_track_extra_info_json(
        track: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;

    /// Calls the `AVPlayer` framework counterpart for `av_player_item_create_with_url`.
    pub fn av_player_item_create_with_url(
        url: *const c_char,
        is_file_url: bool,
        asset_keys_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_create_with_asset`.
    pub fn av_player_item_create_with_asset(
        asset: *mut c_void,
        asset_keys_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_release`.
    pub fn av_player_item_release(item: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_info_json`.
    pub fn av_player_item_info_json(
        item: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_add_observer`.
    pub fn av_player_item_add_observer(
        item: *mut c_void,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_observer_release`.
    pub fn av_player_item_observer_release(observer: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_set_can_use_network_resources_for_live_streaming_while_paused`.
    pub fn av_player_item_set_can_use_network_resources_for_live_streaming_while_paused(
        item: *mut c_void,
        enabled: bool,
    );
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_set_preferred_forward_buffer_duration`.
    pub fn av_player_item_set_preferred_forward_buffer_duration(item: *mut c_void, duration: f64);
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_set_preferred_peak_bit_rate`.
    pub fn av_player_item_set_preferred_peak_bit_rate(item: *mut c_void, value: f64);
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_set_preferred_peak_bit_rate_for_expensive_networks`.
    pub fn av_player_item_set_preferred_peak_bit_rate_for_expensive_networks(
        item: *mut c_void,
        value: f64,
    );
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_set_preferred_maximum_resolution`.
    pub fn av_player_item_set_preferred_maximum_resolution(
        item: *mut c_void,
        width: f64,
        height: f64,
    );
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_set_preferred_maximum_resolution_for_expensive_networks`.
    pub fn av_player_item_set_preferred_maximum_resolution_for_expensive_networks(
        item: *mut c_void,
        width: f64,
        height: f64,
    );
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_set_audio_time_pitch_algorithm`.
    pub fn av_player_item_set_audio_time_pitch_algorithm(
        item: *mut c_void,
        algorithm: *const c_char,
    );
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_set_variant_preferences`.
    pub fn av_player_item_set_variant_preferences(
        item: *mut c_void,
        raw_value: u64,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_track_count`.
    pub fn av_player_item_track_count(item: *mut c_void) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_copy_track_at_index`.
    pub fn av_player_item_copy_track_at_index(item: *mut c_void, index: i32) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_copy_access_log`.
    pub fn av_player_item_copy_access_log(item: *mut c_void) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_copy_error_log`.
    pub fn av_player_item_copy_error_log(item: *mut c_void) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_add_output`.
    pub fn av_player_item_add_output(
        item: *mut c_void,
        output: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_remove_output`.
    pub fn av_player_item_remove_output(item: *mut c_void, output: *mut c_void);

    /// Calls the `AVPlayer` framework counterpart for `av_player_create`.
    pub fn av_player_create(out_error_message: *mut *mut c_char) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_create_with_url`.
    pub fn av_player_create_with_url(
        url: *const c_char,
        is_file_url: bool,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_create_with_asset`.
    pub fn av_player_create_with_asset(
        asset: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_create_with_item`.
    pub fn av_player_create_with_item(
        item: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_release`.
    pub fn av_player_release(player: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_info_json`.
    pub fn av_player_info_json(
        player: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_player_play`.
    pub fn av_player_play(player: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_pause`.
    pub fn av_player_pause(player: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_set_rate`.
    pub fn av_player_set_rate(player: *mut c_void, rate: f32);
    /// Calls the `AVPlayer` framework counterpart for `av_player_seek`.
    pub fn av_player_seek(
        player: *mut c_void,
        value: i64,
        timescale: i32,
        kind: i32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_player_copy_current_item`.
    pub fn av_player_copy_current_item(player: *mut c_void) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_replace_current_item`.
    pub fn av_player_replace_current_item(player: *mut c_void, item: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_set_action_at_item_end`.
    pub fn av_player_set_action_at_item_end(
        player: *mut c_void,
        raw_value: i32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_player_set_volume`.
    pub fn av_player_set_volume(player: *mut c_void, volume: f32);
    /// Calls the `AVPlayer` framework counterpart for `av_player_set_muted`.
    pub fn av_player_set_muted(player: *mut c_void, muted: bool);
    /// Calls the `AVPlayer` framework counterpart for `av_player_set_automatically_waits_to_minimize_stalling`.
    pub fn av_player_set_automatically_waits_to_minimize_stalling(
        player: *mut c_void,
        enabled: bool,
    );
    /// Calls the `AVPlayer` framework counterpart for `av_player_set_applies_media_selection_criteria_automatically`.
    pub fn av_player_set_applies_media_selection_criteria_automatically(
        player: *mut c_void,
        enabled: bool,
    );
    /// Calls the `AVPlayer` framework counterpart for `av_player_set_audiovisual_background_playback_policy`.
    pub fn av_player_set_audiovisual_background_playback_policy(
        player: *mut c_void,
        raw_value: i32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_player_set_network_resource_priority`.
    pub fn av_player_set_network_resource_priority(
        player: *mut c_void,
        raw_value: i32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_player_add_rate_observer`.
    pub fn av_player_add_rate_observer(
        player: *mut c_void,
        queue_label: *const c_char,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_rate_observer_release`.
    pub fn av_player_rate_observer_release(observer: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_eligible_for_hdr_playback_did_change_notification_name`.
    pub fn av_player_eligible_for_hdr_playback_did_change_notification_name(
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_player_set_media_selection_criteria`.
    pub fn av_player_set_media_selection_criteria(
        player: *mut c_void,
        media_characteristic: *const c_char,
        criteria: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_player_copy_media_selection_criteria`.
    pub fn av_player_copy_media_selection_criteria(
        player: *mut c_void,
        media_characteristic: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_add_periodic_time_observer`.
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
    /// Calls the `AVPlayer` framework counterpart for `av_player_add_boundary_time_observer`.
    pub fn av_player_add_boundary_time_observer(
        player: *mut c_void,
        times_json: *const c_char,
        queue_label: *const c_char,
        callback: Option<SimpleCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_time_observer_release`.
    pub fn av_player_time_observer_release(observer: *mut c_void);

    /// Calls the `AVPlayer` framework counterpart for `av_queue_player_create`.
    pub fn av_queue_player_create(out_error_message: *mut *mut c_char) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_queue_player_create_with_items`.
    pub fn av_queue_player_create_with_items(
        item_ptrs: *const *mut c_void,
        count: usize,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_queue_player_release`.
    pub fn av_queue_player_release(player: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_queue_player_item_count`.
    pub fn av_queue_player_item_count(player: *mut c_void) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_queue_player_copy_item_at_index`.
    pub fn av_queue_player_copy_item_at_index(player: *mut c_void, index: i32) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_queue_player_advance_to_next_item`.
    pub fn av_queue_player_advance_to_next_item(player: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_queue_player_can_insert_item_after_item`.
    pub fn av_queue_player_can_insert_item_after_item(
        player: *mut c_void,
        item: *mut c_void,
        after_item: *mut c_void,
    ) -> bool;
    /// Calls the `AVPlayer` framework counterpart for `av_queue_player_insert_item_after_item`.
    pub fn av_queue_player_insert_item_after_item(
        player: *mut c_void,
        item: *mut c_void,
        after_item: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_queue_player_remove_item`.
    pub fn av_queue_player_remove_item(player: *mut c_void, item: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_queue_player_remove_all_items`.
    pub fn av_queue_player_remove_all_items(player: *mut c_void);

    /// Calls the `AVPlayer` framework counterpart for `av_player_looper_create`.
    pub fn av_player_looper_create(
        player: *mut c_void,
        template_item: *mut c_void,
        use_loop_range: bool,
        start_value: i64,
        start_timescale: i32,
        start_kind: i32,
        duration_value: i64,
        duration_timescale: i32,
        duration_kind: i32,
        item_ordering_raw: i32,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_looper_release`.
    pub fn av_player_looper_release(looper: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_looper_info_json`.
    pub fn av_player_looper_info_json(
        looper: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_player_looper_disable_looping`.
    pub fn av_player_looper_disable_looping(looper: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_looper_looping_item_count`.
    pub fn av_player_looper_looping_item_count(looper: *mut c_void) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_player_looper_copy_looping_item_at_index`.
    pub fn av_player_looper_copy_looping_item_at_index(
        looper: *mut c_void,
        index: i32,
    ) -> *mut c_void;

    /// Calls the `AVPlayer` framework counterpart for `av_player_layer_create`.
    pub fn av_player_layer_create(player: *mut c_void) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_layer_release`.
    pub fn av_player_layer_release(layer: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_layer_info_json`.
    pub fn av_player_layer_info_json(
        layer: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_player_layer_set_player`.
    pub fn av_player_layer_set_player(layer: *mut c_void, player: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_layer_set_video_gravity`.
    pub fn av_player_layer_set_video_gravity(layer: *mut c_void, gravity: *const c_char);
    /// Calls the `AVPlayer` framework counterpart for `av_player_layer_copy_displayed_pixel_buffer`.
    pub fn av_player_layer_copy_displayed_pixel_buffer(layer: *mut c_void) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_sample_buffer_display_layer_create`.
    pub fn av_sample_buffer_display_layer_create() -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_sample_buffer_display_layer_release`.
    pub fn av_sample_buffer_display_layer_release(layer: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_sample_buffer_display_layer_info_json`.
    pub fn av_sample_buffer_display_layer_info_json(
        layer: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_sample_buffer_display_layer_set_video_gravity`.
    pub fn av_sample_buffer_display_layer_set_video_gravity(
        layer: *mut c_void,
        gravity: *const c_char,
    );
    /// Calls the `AVPlayer` framework counterpart for `av_sample_buffer_display_layer_set_prevents_capture`.
    pub fn av_sample_buffer_display_layer_set_prevents_capture(
        layer: *mut c_void,
        prevents_capture: bool,
    );
    /// Calls the `AVPlayer` framework counterpart for `av_sample_buffer_display_layer_set_prevents_display_sleep`.
    pub fn av_sample_buffer_display_layer_set_prevents_display_sleep(
        layer: *mut c_void,
        prevents_display_sleep: bool,
    );
    /// Calls the `AVPlayer` framework counterpart for `av_sample_buffer_display_layer_enqueue_sample_buffer`.
    pub fn av_sample_buffer_display_layer_enqueue_sample_buffer(
        layer: *mut c_void,
        sample_buffer: *mut c_void,
    );
    /// Calls the `AVPlayer` framework counterpart for `av_sample_buffer_display_layer_flush`.
    pub fn av_sample_buffer_display_layer_flush(layer: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_sample_buffer_display_layer_flush_and_remove_image`.
    pub fn av_sample_buffer_display_layer_flush_and_remove_image(layer: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_sample_buffer_display_layer_stop_requesting_media_data`.
    pub fn av_sample_buffer_display_layer_stop_requesting_media_data(layer: *mut c_void);

    /// Calls the `AVPlayer` framework counterpart for `av_player_item_track_release`.
    pub fn av_player_item_track_release(track: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_track_info_json`.
    pub fn av_player_item_track_info_json(
        track: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_track_set_enabled`.
    pub fn av_player_item_track_set_enabled(track: *mut c_void, enabled: bool);
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_track_set_video_field_mode`.
    pub fn av_player_item_track_set_video_field_mode(track: *mut c_void, mode: *const c_char);
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_track_copy_asset_track`.
    pub fn av_player_item_track_copy_asset_track(track: *mut c_void) -> *mut c_void;

    /// Calls the `AVPlayer` framework counterpart for `av_player_item_access_log_release`.
    pub fn av_player_item_access_log_release(log: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_access_log_info_json`.
    pub fn av_player_item_access_log_info_json(
        log: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_error_log_release`.
    pub fn av_player_item_error_log_release(log: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_error_log_info_json`.
    pub fn av_player_item_error_log_info_json(
        log: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;

    /// Calls the `AVPlayer` framework counterpart for `av_player_item_output_release`.
    pub fn av_player_item_output_release(output: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_output_suppresses_player_rendering`.
    pub fn av_player_item_output_suppresses_player_rendering(output: *mut c_void) -> bool;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_output_set_suppresses_player_rendering`.
    pub fn av_player_item_output_set_suppresses_player_rendering(
        output: *mut c_void,
        suppresses: bool,
    );
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_output_item_time_for_host_time_json`.
    pub fn av_player_item_output_item_time_for_host_time_json(
        output: *mut c_void,
        host_time: f64,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_output_item_time_for_mach_absolute_time_json`.
    pub fn av_player_item_output_item_time_for_mach_absolute_time_json(
        output: *mut c_void,
        mach_absolute_time: i64,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_video_output_create`.
    pub fn av_player_item_video_output_create(
        settings_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_video_output_info_json`.
    pub fn av_player_item_video_output_info_json(
        output: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_video_output_add_observer`.
    pub fn av_player_item_video_output_add_observer(
        output: *mut c_void,
        queue_label: *const c_char,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_video_output_observer_release`.
    pub fn av_player_item_video_output_observer_release(observer: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_video_output_request_notification_of_media_data_change`.
    pub fn av_player_item_video_output_request_notification_of_media_data_change(
        output: *mut c_void,
        interval: f64,
    );
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_video_output_has_new_pixel_buffer_for_item_time`.
    pub fn av_player_item_video_output_has_new_pixel_buffer_for_item_time(
        output: *mut c_void,
        value: i64,
        timescale: i32,
        kind: i32,
    ) -> bool;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_video_output_copy_pixel_buffer_for_item_time`.
    pub fn av_player_item_video_output_copy_pixel_buffer_for_item_time(
        output: *mut c_void,
        value: i64,
        timescale: i32,
        kind: i32,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_metadata_output_create`.
    pub fn av_player_item_metadata_output_create(
        identifiers_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_metadata_output_info_json`.
    pub fn av_player_item_metadata_output_info_json(
        output: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_metadata_output_add_observer`.
    pub fn av_player_item_metadata_output_add_observer(
        output: *mut c_void,
        queue_label: *const c_char,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_metadata_output_observer_release`.
    pub fn av_player_item_metadata_output_observer_release(observer: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_metadata_output_set_advance_interval`.
    pub fn av_player_item_metadata_output_set_advance_interval(output: *mut c_void, interval: f64);
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_metadata_collector_create`.
    pub fn av_player_item_metadata_collector_create(
        identifiers_json: *const c_char,
        classifying_labels_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_metadata_collector_release`.
    pub fn av_player_item_metadata_collector_release(collector: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_metadata_collector_info_json`.
    pub fn av_player_item_metadata_collector_info_json(
        collector: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_media_data_collector_kind`.
    pub fn av_player_item_media_data_collector_kind(
        collector: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_metadata_collector_add_observer`.
    pub fn av_player_item_metadata_collector_add_observer(
        collector: *mut c_void,
        queue_label: *const c_char,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_metadata_collector_observer_release`.
    pub fn av_player_item_metadata_collector_observer_release(observer: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_add_media_data_collector`.
    pub fn av_player_item_add_media_data_collector(
        item: *mut c_void,
        collector: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_remove_media_data_collector`.
    pub fn av_player_item_remove_media_data_collector(item: *mut c_void, collector: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_media_data_collectors_json`.
    pub fn av_player_item_media_data_collectors_json(
        item: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_legible_output_create`.
    pub fn av_player_item_legible_output_create(
        native_representation_subtypes_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_legible_output_info_json`.
    pub fn av_player_item_legible_output_info_json(
        output: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_legible_output_add_observer`.
    pub fn av_player_item_legible_output_add_observer(
        output: *mut c_void,
        queue_label: *const c_char,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_legible_output_observer_release`.
    pub fn av_player_item_legible_output_observer_release(observer: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_legible_output_set_advance_interval`.
    pub fn av_player_item_legible_output_set_advance_interval(output: *mut c_void, interval: f64);
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_legible_output_set_text_styling_resolution`.
    pub fn av_player_item_legible_output_set_text_styling_resolution(
        output: *mut c_void,
        resolution: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_rendered_legible_output_create`.
    pub fn av_player_item_rendered_legible_output_create(
        width: f64,
        height: f64,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_rendered_legible_output_info_json`.
    pub fn av_player_item_rendered_legible_output_info_json(
        output: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_rendered_legible_output_set_advance_interval`.
    pub fn av_player_item_rendered_legible_output_set_advance_interval(
        output: *mut c_void,
        interval: f64,
    );
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_rendered_legible_output_set_video_display_size`.
    pub fn av_player_item_rendered_legible_output_set_video_display_size(
        output: *mut c_void,
        width: f64,
        height: f64,
    );
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_rendered_legible_output_add_observer`.
    pub fn av_player_item_rendered_legible_output_add_observer(
        output: *mut c_void,
        queue_label: *const c_char,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_rendered_legible_output_observer_release`.
    pub fn av_player_item_rendered_legible_output_observer_release(observer: *mut c_void);

    /// Calls the `AVPlayer` framework counterpart for `av_player_video_output_tag_collection_create_with_preset`.
    pub fn av_player_video_output_tag_collection_create_with_preset(
        preset_raw: u32,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_video_output_tag_collection_release`.
    pub fn av_player_video_output_tag_collection_release(tag_collection: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_video_output_tag_collection_info_json`.
    pub fn av_player_video_output_tag_collection_info_json(
        tag_collection: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_video_output_specification_create`.
    pub fn av_video_output_specification_create(
        tag_collection_ptrs: *const *mut c_void,
        count: usize,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_video_output_specification_release`.
    pub fn av_video_output_specification_release(specification: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_video_output_specification_info_json`.
    pub fn av_video_output_specification_info_json(
        specification: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_video_output_specification_set_default_output_settings`.
    pub fn av_video_output_specification_set_default_output_settings(
        specification: *mut c_void,
        settings_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_video_output_specification_set_output_settings_for_tag_collection`.
    pub fn av_video_output_specification_set_output_settings_for_tag_collection(
        specification: *mut c_void,
        settings_json: *const c_char,
        tag_collection: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_player_video_output_create`.
    pub fn av_player_video_output_create(
        specification: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_video_output_release`.
    pub fn av_player_video_output_release(output: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_video_output_sample_json`.
    pub fn av_player_video_output_sample_json(
        output: *mut c_void,
        host_time_value: i64,
        host_time_timescale: i32,
        host_time_kind: i32,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_player_set_video_output`.
    pub fn av_player_set_video_output(player: *mut c_void, output: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_copy_video_output`.
    pub fn av_player_copy_video_output(player: *mut c_void) -> *mut c_void;

    /// Calls the `AVPlayer` framework counterpart for `av_player_interstitial_event_create_with_time`.
    pub fn av_player_interstitial_event_create_with_time(
        item: *mut c_void,
        value: i64,
        timescale: i32,
        kind: i32,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_interstitial_event_release`.
    pub fn av_player_interstitial_event_release(event: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_interstitial_event_info_json`.
    pub fn av_player_interstitial_event_info_json(
        event: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_player_interstitial_event_set_identifier`.
    pub fn av_player_interstitial_event_set_identifier(
        event: *mut c_void,
        identifier: *const c_char,
    );
    /// Calls the `AVPlayer` framework counterpart for `av_player_interstitial_event_set_restrictions`.
    pub fn av_player_interstitial_event_set_restrictions(event: *mut c_void, restrictions: u64);
    /// Calls the `AVPlayer` framework counterpart for `av_player_interstitial_event_set_resumption_offset`.
    pub fn av_player_interstitial_event_set_resumption_offset(
        event: *mut c_void,
        value: i64,
        timescale: i32,
        kind: i32,
    );
    /// Calls the `AVPlayer` framework counterpart for `av_player_interstitial_event_set_playout_limit`.
    pub fn av_player_interstitial_event_set_playout_limit(
        event: *mut c_void,
        value: i64,
        timescale: i32,
        kind: i32,
    );
    /// Calls the `AVPlayer` framework counterpart for `av_player_interstitial_event_set_aligns_start_with_primary_segment_boundary`.
    pub fn av_player_interstitial_event_set_aligns_start_with_primary_segment_boundary(
        event: *mut c_void,
        enabled: bool,
    );
    /// Calls the `AVPlayer` framework counterpart for `av_player_interstitial_event_set_aligns_resumption_with_primary_segment_boundary`.
    pub fn av_player_interstitial_event_set_aligns_resumption_with_primary_segment_boundary(
        event: *mut c_void,
        enabled: bool,
    );
    /// Calls the `AVPlayer` framework counterpart for `av_player_interstitial_event_set_cue`.
    pub fn av_player_interstitial_event_set_cue(event: *mut c_void, cue: *const c_char);
    /// Calls the `AVPlayer` framework counterpart for `av_player_interstitial_event_set_will_play_once`.
    pub fn av_player_interstitial_event_set_will_play_once(event: *mut c_void, enabled: bool);
    /// Calls the `AVPlayer` framework counterpart for `av_player_interstitial_event_set_timeline_occupancy`.
    pub fn av_player_interstitial_event_set_timeline_occupancy(event: *mut c_void, raw_value: i32);
    /// Calls the `AVPlayer` framework counterpart for `av_player_interstitial_event_set_supplements_primary_content`.
    pub fn av_player_interstitial_event_set_supplements_primary_content(
        event: *mut c_void,
        enabled: bool,
    );
    /// Calls the `AVPlayer` framework counterpart for `av_player_interstitial_event_set_content_may_vary`.
    pub fn av_player_interstitial_event_set_content_may_vary(event: *mut c_void, enabled: bool);
    /// Calls the `AVPlayer` framework counterpart for `av_player_interstitial_event_monitor_create`.
    pub fn av_player_interstitial_event_monitor_create(
        player: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_interstitial_event_monitor_release`.
    pub fn av_player_interstitial_event_monitor_release(monitor: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_interstitial_event_monitor_info_json`.
    pub fn av_player_interstitial_event_monitor_info_json(
        monitor: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_player_interstitial_event_monitor_add_observer`.
    pub fn av_player_interstitial_event_monitor_add_observer(
        monitor: *mut c_void,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_interstitial_event_monitor_observer_release`.
    pub fn av_player_interstitial_event_monitor_observer_release(observer: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_interstitial_event_controller_create`.
    pub fn av_player_interstitial_event_controller_create(
        player: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_interstitial_event_controller_release`.
    pub fn av_player_interstitial_event_controller_release(controller: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_interstitial_event_controller_info_json`.
    pub fn av_player_interstitial_event_controller_info_json(
        controller: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_player_interstitial_event_controller_set_events`.
    pub fn av_player_interstitial_event_controller_set_events(
        controller: *mut c_void,
        event_ptrs: *const *mut c_void,
        count: usize,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_player_interstitial_event_controller_cancel_current_event_with_resumption_offset`.
    pub fn av_player_interstitial_event_controller_cancel_current_event_with_resumption_offset(
        controller: *mut c_void,
        value: i64,
        timescale: i32,
        kind: i32,
    );
    /// Calls the `AVPlayer` framework counterpart for `av_player_interstitial_event_controller_skip_current_event`.
    pub fn av_player_interstitial_event_controller_skip_current_event(controller: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_waiting_during_interstitial_event_reason`.
    pub fn av_player_waiting_during_interstitial_event_reason(
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;

    /// Calls the `AVPlayer` framework counterpart for `av_player_item_copy_integrated_timeline`.
    pub fn av_player_item_copy_integrated_timeline(
        item: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_integrated_timeline_release`.
    pub fn av_player_item_integrated_timeline_release(timeline: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_integrated_timeline_info_json`.
    pub fn av_player_item_integrated_timeline_info_json(
        timeline: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_integrated_timeline_copy_current_snapshot`.
    pub fn av_player_item_integrated_timeline_copy_current_snapshot(
        timeline: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_integrated_timeline_seek_to_time`.
    pub fn av_player_item_integrated_timeline_seek_to_time(
        timeline: *mut c_void,
        time_value: i64,
        time_timescale: i32,
        time_kind: i32,
        before_value: i64,
        before_timescale: i32,
        before_kind: i32,
        after_value: i64,
        after_timescale: i32,
        after_kind: i32,
        out_success: *mut bool,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_integrated_timeline_seek_to_date`.
    pub fn av_player_item_integrated_timeline_seek_to_date(
        timeline: *mut c_void,
        date: *const c_char,
        out_success: *mut bool,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_integrated_timeline_add_periodic_time_observer`.
    pub fn av_player_item_integrated_timeline_add_periodic_time_observer(
        timeline: *mut c_void,
        value: i64,
        timescale: i32,
        kind: i32,
        callback: Option<PeriodicTimeCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_integrated_timeline_add_boundary_time_observer`.
    pub fn av_player_item_integrated_timeline_add_boundary_time_observer(
        timeline: *mut c_void,
        segment: *mut c_void,
        offsets_json: *const c_char,
        callback: Option<PeriodicTimeCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_integrated_timeline_add_out_of_sync_observer`.
    pub fn av_player_item_integrated_timeline_add_out_of_sync_observer(
        timeline: *mut c_void,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_integrated_timeline_observer_release`.
    pub fn av_player_item_integrated_timeline_observer_release(observer: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_integrated_timeline_snapshot_release`.
    pub fn av_player_item_integrated_timeline_snapshot_release(snapshot: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_integrated_timeline_snapshot_info_json`.
    pub fn av_player_item_integrated_timeline_snapshot_info_json(
        snapshot: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_integrated_timeline_snapshot_copy_current_segment`.
    pub fn av_player_item_integrated_timeline_snapshot_copy_current_segment(
        snapshot: *mut c_void,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_integrated_timeline_snapshot_segment_count`.
    pub fn av_player_item_integrated_timeline_snapshot_segment_count(
        snapshot: *mut c_void,
    ) -> usize;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_integrated_timeline_snapshot_copy_segment_at_index`.
    pub fn av_player_item_integrated_timeline_snapshot_copy_segment_at_index(
        snapshot: *mut c_void,
        index: usize,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_integrated_timeline_snapshot_segment_and_offset_json`.
    pub fn av_player_item_integrated_timeline_snapshot_segment_and_offset_json(
        snapshot: *mut c_void,
        value: i64,
        timescale: i32,
        kind: i32,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_integrated_timeline_segment_release`.
    pub fn av_player_item_integrated_timeline_segment_release(segment: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_item_integrated_timeline_segment_info_json`.
    pub fn av_player_item_integrated_timeline_segment_info_json(
        segment: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_player_integrated_timeline_snapshots_out_of_sync_notification`.
    pub fn av_player_integrated_timeline_snapshots_out_of_sync_notification(
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_player_integrated_timeline_snapshots_out_of_sync_reason_key`.
    pub fn av_player_integrated_timeline_snapshots_out_of_sync_reason_key(
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_player_integrated_timeline_snapshots_out_of_sync_reason_segments_changed`.
    pub fn av_player_integrated_timeline_snapshots_out_of_sync_reason_segments_changed(
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_player_integrated_timeline_snapshots_out_of_sync_reason_current_segment_changed`.
    pub fn av_player_integrated_timeline_snapshots_out_of_sync_reason_current_segment_changed(
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_player_integrated_timeline_snapshots_out_of_sync_reason_loaded_time_ranges_changed`.
    pub fn av_player_integrated_timeline_snapshots_out_of_sync_reason_loaded_time_ranges_changed(
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;

    /// Calls the `AVPlayer` framework counterpart for `av_player_media_selection_criteria_create`.
    pub fn av_player_media_selection_criteria_create(
        preferred_languages_json: *const c_char,
        preferred_media_characteristics_json: *const c_char,
        principal_media_characteristics_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_player_media_selection_criteria_release`.
    pub fn av_player_media_selection_criteria_release(criteria: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_player_media_selection_criteria_info_json`.
    pub fn av_player_media_selection_criteria_info_json(
        criteria: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;

    /// Calls the `AVPlayer` framework counterpart for `av_reader_create`.
    pub fn av_reader_create(asset: *mut c_void, out_error_message: *mut *mut c_char)
        -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_reader_release`.
    pub fn av_reader_release(reader: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_reader_info_json`.
    pub fn av_reader_info_json(
        reader: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_reader_set_time_range`.
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
    /// Calls the `AVPlayer` framework counterpart for `av_reader_start`.
    pub fn av_reader_start(reader: *mut c_void, out_error_message: *mut *mut c_char) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_reader_cancel`.
    pub fn av_reader_cancel(reader: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_reader_can_add_output`.
    pub fn av_reader_can_add_output(reader: *mut c_void, output: *mut c_void) -> bool;
    /// Calls the `AVPlayer` framework counterpart for `av_reader_add_output`.
    pub fn av_reader_add_output(
        reader: *mut c_void,
        output: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;

    /// Calls the `AVPlayer` framework counterpart for `av_reader_track_output_create_video`.
    pub fn av_reader_track_output_create_video(
        track: *mut c_void,
        settings_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_reader_track_output_create_audio`.
    pub fn av_reader_track_output_create_audio(
        track: *mut c_void,
        settings_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_reader_track_output_create_passthrough`.
    pub fn av_reader_track_output_create_passthrough(
        track: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_reader_audio_mix_output_create`.
    pub fn av_reader_audio_mix_output_create(
        tracks: *const *mut c_void,
        count: usize,
        settings_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_reader_video_composition_output_create`.
    pub fn av_reader_video_composition_output_create(
        tracks: *const *mut c_void,
        count: usize,
        settings_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_reader_output_release`.
    pub fn av_reader_output_release(output: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_reader_output_set_always_copies_sample_data`.
    pub fn av_reader_output_set_always_copies_sample_data(output: *mut c_void, always_copies: bool);
    /// Calls the `AVPlayer` framework counterpart for `av_reader_output_media_type`.
    pub fn av_reader_output_media_type(output: *mut c_void) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_reader_output_copy_next_sample_buffer`.
    pub fn av_reader_output_copy_next_sample_buffer(output: *mut c_void) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_reader_output_copy_next_video_pixel_buffer`.
    pub fn av_reader_output_copy_next_video_pixel_buffer(output: *mut c_void) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_reader_output_supports_random_access`.
    pub fn av_reader_output_supports_random_access(output: *mut c_void) -> bool;
    /// Calls the `AVPlayer` framework counterpart for `av_reader_output_set_supports_random_access`.
    pub fn av_reader_output_set_supports_random_access(
        output: *mut c_void,
        supports_random_access: bool,
    );
    /// Calls the `AVPlayer` framework counterpart for `av_reader_output_reset_for_time_ranges_json`.
    pub fn av_reader_output_reset_for_time_ranges_json(
        output: *mut c_void,
        time_ranges_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_reader_output_mark_configuration_as_final`.
    pub fn av_reader_output_mark_configuration_as_final(output: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_reader_sample_reference_output_create`.
    pub fn av_reader_sample_reference_output_create(
        track: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_reader_sample_reference_output_copy_track`.
    pub fn av_reader_sample_reference_output_copy_track(output: *mut c_void) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_reader_output_metadata_adaptor_create`.
    pub fn av_reader_output_metadata_adaptor_create(
        track_output: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_reader_output_metadata_adaptor_copy_track_output`.
    pub fn av_reader_output_metadata_adaptor_copy_track_output(adaptor: *mut c_void)
        -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_reader_output_metadata_adaptor_copy_next_timed_metadata_group`.
    pub fn av_reader_output_metadata_adaptor_copy_next_timed_metadata_group(
        adaptor: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_reader_output_caption_adaptor_create`.
    pub fn av_reader_output_caption_adaptor_create(
        track_output: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_reader_output_caption_adaptor_copy_track_output`.
    pub fn av_reader_output_caption_adaptor_copy_track_output(adaptor: *mut c_void) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_reader_output_caption_adaptor_next_caption_group_json`.
    pub fn av_reader_output_caption_adaptor_next_caption_group_json(
        adaptor: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_reader_output_caption_adaptor_add_validation_observer`.
    pub fn av_reader_output_caption_adaptor_add_validation_observer(
        adaptor: *mut c_void,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_reader_output_caption_validation_observer_release`.
    pub fn av_reader_output_caption_validation_observer_release(observer: *mut c_void);

    /// Calls the `AVPlayer` framework counterpart for `av_ns_object_release`.
    pub fn av_ns_object_release(object: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_fragmented_asset_create`.
    pub fn av_fragmented_asset_create(
        url: *const c_char,
        is_file_url: bool,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_fragmented_asset_track_count`.
    pub fn av_fragmented_asset_track_count(asset: *mut c_void) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_fragmented_asset_copy_track_at_index`.
    pub fn av_fragmented_asset_copy_track_at_index(asset: *mut c_void, index: i32) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_fragmented_asset_copy_track_with_id`.
    pub fn av_fragmented_asset_copy_track_with_id(asset: *mut c_void, track_id: i32)
        -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_fragmented_asset_is_associated_with_minder`.
    pub fn av_fragmented_asset_is_associated_with_minder(asset: *mut c_void) -> bool;
    /// Calls the `AVPlayer` framework counterpart for `av_fragmented_asset_expects_property_revised_notifications`.
    pub fn av_fragmented_asset_expects_property_revised_notifications() -> bool;
    /// Calls the `AVPlayer` framework counterpart for `av_fragmented_asset_is_playable_extended_mime_type`.
    pub fn av_fragmented_asset_is_playable_extended_mime_type(mime_type: *const c_char) -> bool;
    /// Calls the `AVPlayer` framework counterpart for `av_fragmented_asset_track_segment_count`.
    pub fn av_fragmented_asset_track_segment_count(track: *mut c_void) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_url_asset_copy_media_extension_properties`.
    pub fn av_url_asset_copy_media_extension_properties(asset: *mut c_void) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_media_extension_properties_info_json`.
    pub fn av_media_extension_properties_info_json(
        properties: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_fragmented_asset_minder_create`.
    pub fn av_fragmented_asset_minder_create(
        asset: *mut c_void,
        minding_interval: f64,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_fragmented_asset_minder_info_json`.
    pub fn av_fragmented_asset_minder_info_json(
        minder: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_fragmented_asset_minder_copy_asset_at_index`.
    pub fn av_fragmented_asset_minder_copy_asset_at_index(
        minder: *mut c_void,
        index: i32,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_fragmented_asset_minder_set_interval`.
    pub fn av_fragmented_asset_minder_set_interval(minder: *mut c_void, minding_interval: f64);
    /// Calls the `AVPlayer` framework counterpart for `av_fragmented_asset_minder_add_asset`.
    pub fn av_fragmented_asset_minder_add_asset(minder: *mut c_void, asset: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_fragmented_asset_minder_remove_asset`.
    pub fn av_fragmented_asset_minder_remove_asset(minder: *mut c_void, asset: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_asset_available_media_selection_characteristics_json`.
    pub fn av_asset_available_media_selection_characteristics_json(
        asset: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_asset_copy_media_selection_group_for_characteristic`.
    pub fn av_asset_copy_media_selection_group_for_characteristic(
        asset: *mut c_void,
        characteristic: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_asset_copy_preferred_media_selection`.
    pub fn av_asset_copy_preferred_media_selection(
        asset: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_asset_media_selection_count`.
    pub fn av_asset_media_selection_count(asset: *mut c_void) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_asset_copy_media_selection_at_index`.
    pub fn av_asset_copy_media_selection_at_index(asset: *mut c_void, index: i32) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_media_selection_copy_selected_option`.
    pub fn av_media_selection_copy_selected_option(
        selection: *mut c_void,
        group: *mut c_void,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_media_selection_can_apply_automatically`.
    pub fn av_media_selection_can_apply_automatically(
        selection: *mut c_void,
        group: *mut c_void,
    ) -> bool;
    /// Calls the `AVPlayer` framework counterpart for `av_media_selection_mutable_copy`.
    pub fn av_media_selection_mutable_copy(
        selection: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_mutable_media_selection_select_option`.
    pub fn av_mutable_media_selection_select_option(
        selection: *mut c_void,
        option: *mut c_void,
        group: *mut c_void,
    );
    /// Calls the `AVPlayer` framework counterpart for `av_media_selection_group_option_count`.
    pub fn av_media_selection_group_option_count(group: *mut c_void) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_media_selection_group_copy_option_at_index`.
    pub fn av_media_selection_group_copy_option_at_index(
        group: *mut c_void,
        index: i32,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_media_selection_group_copy_default_option`.
    pub fn av_media_selection_group_copy_default_option(group: *mut c_void) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_media_selection_group_allows_empty_selection`.
    pub fn av_media_selection_group_allows_empty_selection(group: *mut c_void) -> bool;
    /// Calls the `AVPlayer` framework counterpart for `av_media_selection_group_copy_custom_media_selection_scheme`.
    pub fn av_media_selection_group_copy_custom_media_selection_scheme(
        group: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_media_selection_option_info_json`.
    pub fn av_media_selection_option_info_json(
        option: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_media_selection_option_has_media_characteristic`.
    pub fn av_media_selection_option_has_media_characteristic(
        option: *mut c_void,
        characteristic: *const c_char,
    ) -> bool;
    /// Calls the `AVPlayer` framework counterpart for `av_media_selection_option_copy_associated_option`.
    pub fn av_media_selection_option_copy_associated_option(
        option: *mut c_void,
        group: *mut c_void,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_media_selection_option_display_name_for_locale_identifier`.
    pub fn av_media_selection_option_display_name_for_locale_identifier(
        option: *mut c_void,
        locale_identifier: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_custom_media_selection_scheme_should_offer_language_selection`.
    pub fn av_custom_media_selection_scheme_should_offer_language_selection(
        scheme: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> bool;
    /// Calls the `AVPlayer` framework counterpart for `av_custom_media_selection_scheme_available_languages_json`.
    pub fn av_custom_media_selection_scheme_available_languages_json(
        scheme: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_custom_media_selection_scheme_selector_count`.
    pub fn av_custom_media_selection_scheme_selector_count(
        scheme: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_custom_media_selection_scheme_copy_selector_at_index`.
    pub fn av_custom_media_selection_scheme_copy_selector_at_index(
        scheme: *mut c_void,
        index: i32,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_media_presentation_selector_identifier`.
    pub fn av_media_presentation_selector_identifier(
        selector: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_media_presentation_selector_display_name_for_locale_identifier`.
    pub fn av_media_presentation_selector_display_name_for_locale_identifier(
        selector: *mut c_void,
        locale_identifier: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_media_presentation_selector_setting_count`.
    pub fn av_media_presentation_selector_setting_count(
        selector: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_media_presentation_selector_copy_setting_at_index`.
    pub fn av_media_presentation_selector_copy_setting_at_index(
        selector: *mut c_void,
        index: i32,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_media_presentation_setting_media_characteristic`.
    pub fn av_media_presentation_setting_media_characteristic(
        setting: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_media_presentation_setting_display_name_for_locale_identifier`.
    pub fn av_media_presentation_setting_display_name_for_locale_identifier(
        setting: *mut c_void,
        locale_identifier: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_asset_variant_count`.
    pub fn av_asset_variant_count(asset: *mut c_void) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_asset_copy_variant_at_index`.
    pub fn av_asset_copy_variant_at_index(asset: *mut c_void, index: i32) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_asset_variant_info_json`.
    pub fn av_asset_variant_info_json(
        variant: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_asset_variant_copy_video_attributes`.
    pub fn av_asset_variant_copy_video_attributes(variant: *mut c_void) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_asset_variant_copy_audio_attributes`.
    pub fn av_asset_variant_copy_audio_attributes(variant: *mut c_void) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_asset_variant_video_attributes_info_json`.
    pub fn av_asset_variant_video_attributes_info_json(
        attributes: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_asset_variant_video_layout_attribute_count`.
    pub fn av_asset_variant_video_layout_attribute_count(attributes: *mut c_void) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_asset_variant_video_layout_attribute_copy_at_index`.
    pub fn av_asset_variant_video_layout_attribute_copy_at_index(
        attributes: *mut c_void,
        index: i32,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_asset_variant_video_layout_attributes_info_json`.
    pub fn av_asset_variant_video_layout_attributes_info_json(
        attributes: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_asset_variant_audio_attributes_info_json`.
    pub fn av_asset_variant_audio_attributes_info_json(
        attributes: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_asset_variant_audio_attributes_copy_rendition_specific_attributes`.
    pub fn av_asset_variant_audio_attributes_copy_rendition_specific_attributes(
        attributes: *mut c_void,
        option: *mut c_void,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_asset_variant_audio_rendition_info_json`.
    pub fn av_asset_variant_audio_rendition_info_json(
        attributes: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_asset_variant_qualifier_create_with_variant`.
    pub fn av_asset_variant_qualifier_create_with_variant(
        variant: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_metadata_group_info_json`.
    pub fn av_metadata_group_info_json(
        group: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_timed_metadata_group_create`.
    pub fn av_timed_metadata_group_create(
        items_json: *const c_char,
        start_value: i64,
        start_timescale: i32,
        start_kind: i32,
        duration_value: i64,
        duration_timescale: i32,
        duration_kind: i32,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_timed_metadata_group_info_json`.
    pub fn av_timed_metadata_group_info_json(
        group: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_mutable_timed_metadata_group_create`.
    pub fn av_mutable_timed_metadata_group_create(
        items_json: *const c_char,
        start_value: i64,
        start_timescale: i32,
        start_kind: i32,
        duration_value: i64,
        duration_timescale: i32,
        duration_kind: i32,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_mutable_timed_metadata_group_set_time_range`.
    pub fn av_mutable_timed_metadata_group_set_time_range(
        group: *mut c_void,
        start_value: i64,
        start_timescale: i32,
        start_kind: i32,
        duration_value: i64,
        duration_timescale: i32,
        duration_kind: i32,
    );
    /// Calls the `AVPlayer` framework counterpart for `av_mutable_timed_metadata_group_set_items_json`.
    pub fn av_mutable_timed_metadata_group_set_items_json(
        group: *mut c_void,
        items_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_date_range_metadata_group_create`.
    pub fn av_date_range_metadata_group_create(
        items_json: *const c_char,
        start_date: *const c_char,
        end_date: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_date_range_metadata_group_info_json`.
    pub fn av_date_range_metadata_group_info_json(
        group: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_mutable_date_range_metadata_group_create`.
    pub fn av_mutable_date_range_metadata_group_create(
        items_json: *const c_char,
        start_date: *const c_char,
        end_date: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_mutable_metadata_item_create`.
    pub fn av_mutable_metadata_item_create() -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_mutable_metadata_item_info_json`.
    pub fn av_mutable_metadata_item_info_json(
        item: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    /// Calls the `AVPlayer` framework counterpart for `av_mutable_metadata_item_set_identifier`.
    pub fn av_mutable_metadata_item_set_identifier(item: *mut c_void, identifier: *const c_char);
    /// Calls the `AVPlayer` framework counterpart for `av_mutable_metadata_item_set_extended_language_tag`.
    pub fn av_mutable_metadata_item_set_extended_language_tag(
        item: *mut c_void,
        language_tag: *const c_char,
    );
    /// Calls the `AVPlayer` framework counterpart for `av_mutable_metadata_item_set_locale_identifier`.
    pub fn av_mutable_metadata_item_set_locale_identifier(
        item: *mut c_void,
        locale_identifier: *const c_char,
    );
    /// Calls the `AVPlayer` framework counterpart for `av_mutable_metadata_item_set_time`.
    pub fn av_mutable_metadata_item_set_time(
        item: *mut c_void,
        value: i64,
        timescale: i32,
        kind: i32,
    );
    /// Calls the `AVPlayer` framework counterpart for `av_mutable_metadata_item_set_duration`.
    pub fn av_mutable_metadata_item_set_duration(
        item: *mut c_void,
        value: i64,
        timescale: i32,
        kind: i32,
    );
    /// Calls the `AVPlayer` framework counterpart for `av_mutable_metadata_item_set_data_type`.
    pub fn av_mutable_metadata_item_set_data_type(item: *mut c_void, data_type: *const c_char);
    /// Calls the `AVPlayer` framework counterpart for `av_mutable_metadata_item_set_string_value`.
    pub fn av_mutable_metadata_item_set_string_value(item: *mut c_void, value: *const c_char);
    /// Calls the `AVPlayer` framework counterpart for `av_mutable_metadata_item_set_number_value`.
    pub fn av_mutable_metadata_item_set_number_value(item: *mut c_void, value: f64);
    /// Calls the `AVPlayer` framework counterpart for `av_mutable_metadata_item_clear_value`.
    pub fn av_mutable_metadata_item_clear_value(item: *mut c_void);
    /// Calls the `AVPlayer` framework counterpart for `av_mutable_metadata_item_set_start_date`.
    pub fn av_mutable_metadata_item_set_start_date(
        item: *mut c_void,
        start_date: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    /// Calls the `AVPlayer` framework counterpart for `av_mutable_metadata_item_set_key_space`.
    pub fn av_mutable_metadata_item_set_key_space(item: *mut c_void, key_space: *const c_char);
    /// Calls the `AVPlayer` framework counterpart for `av_mutable_metadata_item_set_key_string`.
    pub fn av_mutable_metadata_item_set_key_string(item: *mut c_void, key: *const c_char);
    /// Calls the `AVPlayer` framework counterpart for `av_metadata_item_filter_create_for_sharing`.
    pub fn av_metadata_item_filter_create_for_sharing() -> *mut c_void;
    /// Calls the `AVPlayer` framework counterpart for `av_metadata_item_filter_filter_json`.
    pub fn av_metadata_item_filter_filter_json(
        filter: *mut c_void,
        items_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
}

/// Groups `AVPlayer` framework constants for `status`.
pub mod status {
    /// Mirrors the `AVPlayer` framework constant `OK`.
    pub const OK: i32 = 0;
    /// Mirrors the `AVPlayer` framework constant `INVALID_ARGUMENT`.
    pub const INVALID_ARGUMENT: i32 = -1;
    /// Mirrors the `AVPlayer` framework constant `ASSET_CREATE_FAILED`.
    pub const ASSET_CREATE_FAILED: i32 = -2;
    /// Mirrors the `AVPlayer` framework constant `PLAYER_CREATE_FAILED`.
    pub const PLAYER_CREATE_FAILED: i32 = -3;
    /// Mirrors the `AVPlayer` framework constant `READER_CREATE_FAILED`.
    pub const READER_CREATE_FAILED: i32 = -4;
    /// Mirrors the `AVPlayer` framework constant `OPERATION_FAILED`.
    pub const OPERATION_FAILED: i32 = -5;
    /// Mirrors the `AVPlayer` framework constant `OBSERVER_FAILED`.
    pub const OBSERVER_FAILED: i32 = -6;
    /// Mirrors the `AVPlayer` framework constant `LOAD_FAILED`.
    pub const LOAD_FAILED: i32 = -7;
}

// ── Async (feature = "async") ────────────────────────────────────────────────

/// Callback signature used by all `avp_*_async` Swift thunks.
///
/// * `result`  – opaque result pointer (JSON `*mut c_char` or packed bool);
///   `null` when an error occurred.
/// * `error`   – null-terminated error message; `null` on success.
/// * `ctx`     – opaque context returned unchanged to the Rust completion handler.
#[cfg(feature = "async")]
pub type AsyncCallback =
    unsafe extern "C" fn(result: *const c_void, error: *const c_char, ctx: *mut c_void);

#[cfg(feature = "async")]
extern "C" {
    /// Load asset properties (duration, metadata, isPlayable, isExportable,
    /// hasProtectedContent, preferredRate) concurrently via `AVAsset.load(...)`.
    /// On success the `result` pointer is a heap-allocated JSON `*mut c_char`
    /// that the Rust callback must free with `avp_string_free`.
    pub fn avp_asset_load_properties_async(asset: *mut c_void, cb: AsyncCallback, ctx: *mut c_void);

    /// Load all tracks via `AVAsset.load(.tracks)`.
    /// Result is a JSON array of `TrackInfoPayload` objects (heap-allocated).
    pub fn avp_asset_load_tracks_async(asset: *mut c_void, cb: AsyncCallback, ctx: *mut c_void);

    /// Load tracks filtered by media-type string via `AVAsset.loadTracks(withMediaType:)`.
    /// Result is a JSON array of `TrackInfoPayload` objects (heap-allocated).
    pub fn avp_asset_load_tracks_with_media_type_async(
        asset: *mut c_void,
        media_type: *const c_char,
        cb: AsyncCallback,
        ctx: *mut c_void,
    );

    /// Load a single track by persistent track ID via `AVAsset.loadTrack(withTrackID:)`.
    /// Result is a JSON object (`TrackInfoPayload`) or the literal `"null"` when
    /// no track with that ID exists.
    pub fn avp_asset_load_track_with_id_async(
        asset: *mut c_void,
        track_id: i32,
        cb: AsyncCallback,
        ctx: *mut c_void,
    );

    /// Seek a player item via `AVPlayerItem.seek(to:completionHandler:)`.
    /// Result pointer encodes the `finished` bool: `1` = finished, `null` = not finished.
    pub fn avp_player_item_seek_async(
        item: *mut c_void,
        value: i64,
        timescale: i32,
        kind: i32,
        cb: AsyncCallback,
        ctx: *mut c_void,
    );

    /// Seek the player via `AVPlayer.seek(to:completionHandler:)`.
    /// Result pointer encodes the `finished` bool: `1` = finished, `null` = not finished.
    pub fn avp_player_seek_async(
        player: *mut c_void,
        value: i64,
        timescale: i32,
        kind: i32,
        cb: AsyncCallback,
        ctx: *mut c_void,
    );

    /// Preroll via `AVPlayer.preroll(atRate:completionHandler:)`.
    /// Result pointer encodes `finished`: `1` = finished, `null` = not finished.
    pub fn avp_player_preroll_async(
        player: *mut c_void,
        rate: f32,
        cb: AsyncCallback,
        ctx: *mut c_void,
    );
}
