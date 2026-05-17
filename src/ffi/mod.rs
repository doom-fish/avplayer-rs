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
    pub fn av_asset_info_json(
        asset: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
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
    pub fn av_asset_track_info_json(
        track: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;

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
    pub fn av_player_item_set_can_use_network_resources_for_live_streaming_while_paused(
        item: *mut c_void,
        enabled: bool,
    );
    pub fn av_player_item_set_preferred_forward_buffer_duration(item: *mut c_void, duration: f64);
    pub fn av_player_item_set_preferred_peak_bit_rate(item: *mut c_void, value: f64);
    pub fn av_player_item_set_preferred_peak_bit_rate_for_expensive_networks(
        item: *mut c_void,
        value: f64,
    );
    pub fn av_player_item_set_preferred_maximum_resolution(
        item: *mut c_void,
        width: f64,
        height: f64,
    );
    pub fn av_player_item_set_preferred_maximum_resolution_for_expensive_networks(
        item: *mut c_void,
        width: f64,
        height: f64,
    );
    pub fn av_player_item_set_audio_time_pitch_algorithm(
        item: *mut c_void,
        algorithm: *const c_char,
    );
    pub fn av_player_item_set_variant_preferences(
        item: *mut c_void,
        raw_value: u64,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_player_item_track_count(item: *mut c_void) -> i32;
    pub fn av_player_item_copy_track_at_index(item: *mut c_void, index: i32) -> *mut c_void;
    pub fn av_player_item_copy_access_log(item: *mut c_void) -> *mut c_void;
    pub fn av_player_item_copy_error_log(item: *mut c_void) -> *mut c_void;
    pub fn av_player_item_add_output(
        item: *mut c_void,
        output: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_player_item_remove_output(item: *mut c_void, output: *mut c_void);

    pub fn av_player_create(out_error_message: *mut *mut c_char) -> *mut c_void;
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
    pub fn av_player_info_json(
        player: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
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
    pub fn av_player_replace_current_item(player: *mut c_void, item: *mut c_void);
    pub fn av_player_set_action_at_item_end(
        player: *mut c_void,
        raw_value: i32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_player_set_volume(player: *mut c_void, volume: f32);
    pub fn av_player_set_muted(player: *mut c_void, muted: bool);
    pub fn av_player_set_automatically_waits_to_minimize_stalling(
        player: *mut c_void,
        enabled: bool,
    );
    pub fn av_player_set_applies_media_selection_criteria_automatically(
        player: *mut c_void,
        enabled: bool,
    );
    pub fn av_player_set_audiovisual_background_playback_policy(
        player: *mut c_void,
        raw_value: i32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_player_set_network_resource_priority(
        player: *mut c_void,
        raw_value: i32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_player_add_rate_observer(
        player: *mut c_void,
        queue_label: *const c_char,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_player_rate_observer_release(observer: *mut c_void);
    pub fn av_player_eligible_for_hdr_playback_did_change_notification_name(
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_player_set_media_selection_criteria(
        player: *mut c_void,
        media_characteristic: *const c_char,
        criteria: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_player_copy_media_selection_criteria(
        player: *mut c_void,
        media_characteristic: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
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

    pub fn av_queue_player_create(out_error_message: *mut *mut c_char) -> *mut c_void;
    pub fn av_queue_player_create_with_items(
        item_ptrs: *const *mut c_void,
        count: usize,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_queue_player_release(player: *mut c_void);
    pub fn av_queue_player_item_count(player: *mut c_void) -> i32;
    pub fn av_queue_player_copy_item_at_index(player: *mut c_void, index: i32) -> *mut c_void;
    pub fn av_queue_player_advance_to_next_item(player: *mut c_void);
    pub fn av_queue_player_can_insert_item_after_item(
        player: *mut c_void,
        item: *mut c_void,
        after_item: *mut c_void,
    ) -> bool;
    pub fn av_queue_player_insert_item_after_item(
        player: *mut c_void,
        item: *mut c_void,
        after_item: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_queue_player_remove_item(player: *mut c_void, item: *mut c_void);
    pub fn av_queue_player_remove_all_items(player: *mut c_void);

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
    pub fn av_player_looper_release(looper: *mut c_void);
    pub fn av_player_looper_info_json(
        looper: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_player_looper_disable_looping(looper: *mut c_void);
    pub fn av_player_looper_looping_item_count(looper: *mut c_void) -> i32;
    pub fn av_player_looper_copy_looping_item_at_index(
        looper: *mut c_void,
        index: i32,
    ) -> *mut c_void;

    pub fn av_player_layer_create(player: *mut c_void) -> *mut c_void;
    pub fn av_player_layer_release(layer: *mut c_void);
    pub fn av_player_layer_info_json(
        layer: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_player_layer_set_player(layer: *mut c_void, player: *mut c_void);
    pub fn av_player_layer_set_video_gravity(layer: *mut c_void, gravity: *const c_char);
    pub fn av_player_layer_copy_displayed_pixel_buffer(layer: *mut c_void) -> *mut c_void;

    pub fn av_player_item_track_release(track: *mut c_void);
    pub fn av_player_item_track_info_json(
        track: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_player_item_track_set_enabled(track: *mut c_void, enabled: bool);
    pub fn av_player_item_track_set_video_field_mode(track: *mut c_void, mode: *const c_char);
    pub fn av_player_item_track_copy_asset_track(track: *mut c_void) -> *mut c_void;

    pub fn av_player_item_access_log_release(log: *mut c_void);
    pub fn av_player_item_access_log_info_json(
        log: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_player_item_error_log_release(log: *mut c_void);
    pub fn av_player_item_error_log_info_json(
        log: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;

    pub fn av_player_item_output_release(output: *mut c_void);
    pub fn av_player_item_output_suppresses_player_rendering(output: *mut c_void) -> bool;
    pub fn av_player_item_output_set_suppresses_player_rendering(
        output: *mut c_void,
        suppresses: bool,
    );
    pub fn av_player_item_output_item_time_for_host_time_json(
        output: *mut c_void,
        host_time: f64,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_player_item_output_item_time_for_mach_absolute_time_json(
        output: *mut c_void,
        mach_absolute_time: i64,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_player_item_video_output_create(
        settings_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_player_item_video_output_info_json(
        output: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_player_item_video_output_add_observer(
        output: *mut c_void,
        queue_label: *const c_char,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_player_item_video_output_observer_release(observer: *mut c_void);
    pub fn av_player_item_video_output_request_notification_of_media_data_change(
        output: *mut c_void,
        interval: f64,
    );
    pub fn av_player_item_video_output_has_new_pixel_buffer_for_item_time(
        output: *mut c_void,
        value: i64,
        timescale: i32,
        kind: i32,
    ) -> bool;
    pub fn av_player_item_video_output_copy_pixel_buffer_for_item_time(
        output: *mut c_void,
        value: i64,
        timescale: i32,
        kind: i32,
    ) -> *mut c_void;
    pub fn av_player_item_metadata_output_create(
        identifiers_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_player_item_metadata_output_info_json(
        output: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_player_item_metadata_output_add_observer(
        output: *mut c_void,
        queue_label: *const c_char,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_player_item_metadata_output_observer_release(observer: *mut c_void);
    pub fn av_player_item_metadata_output_set_advance_interval(output: *mut c_void, interval: f64);
    pub fn av_player_item_metadata_collector_create(
        identifiers_json: *const c_char,
        classifying_labels_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_player_item_metadata_collector_release(collector: *mut c_void);
    pub fn av_player_item_metadata_collector_info_json(
        collector: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_player_item_metadata_collector_add_observer(
        collector: *mut c_void,
        queue_label: *const c_char,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_player_item_metadata_collector_observer_release(observer: *mut c_void);
    pub fn av_player_item_add_media_data_collector(
        item: *mut c_void,
        collector: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_player_item_remove_media_data_collector(item: *mut c_void, collector: *mut c_void);
    pub fn av_player_item_media_data_collectors_json(
        item: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_player_item_legible_output_create(
        native_representation_subtypes_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_player_item_legible_output_info_json(
        output: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_player_item_legible_output_add_observer(
        output: *mut c_void,
        queue_label: *const c_char,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_player_item_legible_output_observer_release(observer: *mut c_void);
    pub fn av_player_item_legible_output_set_advance_interval(output: *mut c_void, interval: f64);
    pub fn av_player_item_legible_output_set_text_styling_resolution(
        output: *mut c_void,
        resolution: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_player_item_rendered_legible_output_create(
        width: f64,
        height: f64,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_player_item_rendered_legible_output_info_json(
        output: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_player_item_rendered_legible_output_set_advance_interval(
        output: *mut c_void,
        interval: f64,
    );
    pub fn av_player_item_rendered_legible_output_set_video_display_size(
        output: *mut c_void,
        width: f64,
        height: f64,
    );
    pub fn av_player_item_rendered_legible_output_add_observer(
        output: *mut c_void,
        queue_label: *const c_char,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_player_item_rendered_legible_output_observer_release(observer: *mut c_void);

    pub fn av_player_video_output_tag_collection_create_with_preset(
        preset_raw: u32,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_player_video_output_tag_collection_release(tag_collection: *mut c_void);
    pub fn av_player_video_output_tag_collection_info_json(
        tag_collection: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_video_output_specification_create(
        tag_collection_ptrs: *const *mut c_void,
        count: usize,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_video_output_specification_release(specification: *mut c_void);
    pub fn av_video_output_specification_info_json(
        specification: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_video_output_specification_set_default_output_settings(
        specification: *mut c_void,
        settings_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_video_output_specification_set_output_settings_for_tag_collection(
        specification: *mut c_void,
        settings_json: *const c_char,
        tag_collection: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_player_video_output_create(
        specification: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_player_video_output_release(output: *mut c_void);
    pub fn av_player_video_output_sample_json(
        output: *mut c_void,
        host_time_value: i64,
        host_time_timescale: i32,
        host_time_kind: i32,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_player_set_video_output(player: *mut c_void, output: *mut c_void);
    pub fn av_player_copy_video_output(player: *mut c_void) -> *mut c_void;

    pub fn av_player_interstitial_event_create_with_time(
        item: *mut c_void,
        value: i64,
        timescale: i32,
        kind: i32,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_player_interstitial_event_release(event: *mut c_void);
    pub fn av_player_interstitial_event_info_json(
        event: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_player_interstitial_event_set_identifier(
        event: *mut c_void,
        identifier: *const c_char,
    );
    pub fn av_player_interstitial_event_set_restrictions(event: *mut c_void, restrictions: u64);
    pub fn av_player_interstitial_event_set_resumption_offset(
        event: *mut c_void,
        value: i64,
        timescale: i32,
        kind: i32,
    );
    pub fn av_player_interstitial_event_set_playout_limit(
        event: *mut c_void,
        value: i64,
        timescale: i32,
        kind: i32,
    );
    pub fn av_player_interstitial_event_set_aligns_start_with_primary_segment_boundary(
        event: *mut c_void,
        enabled: bool,
    );
    pub fn av_player_interstitial_event_set_aligns_resumption_with_primary_segment_boundary(
        event: *mut c_void,
        enabled: bool,
    );
    pub fn av_player_interstitial_event_set_cue(event: *mut c_void, cue: *const c_char);
    pub fn av_player_interstitial_event_set_will_play_once(event: *mut c_void, enabled: bool);
    pub fn av_player_interstitial_event_set_timeline_occupancy(event: *mut c_void, raw_value: i32);
    pub fn av_player_interstitial_event_set_supplements_primary_content(
        event: *mut c_void,
        enabled: bool,
    );
    pub fn av_player_interstitial_event_set_content_may_vary(event: *mut c_void, enabled: bool);
    pub fn av_player_interstitial_event_monitor_create(
        player: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_player_interstitial_event_monitor_release(monitor: *mut c_void);
    pub fn av_player_interstitial_event_monitor_info_json(
        monitor: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_player_interstitial_event_monitor_add_observer(
        monitor: *mut c_void,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_player_interstitial_event_monitor_observer_release(observer: *mut c_void);
    pub fn av_player_interstitial_event_controller_create(
        player: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_player_interstitial_event_controller_release(controller: *mut c_void);
    pub fn av_player_interstitial_event_controller_info_json(
        controller: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_player_interstitial_event_controller_set_events(
        controller: *mut c_void,
        event_ptrs: *const *mut c_void,
        count: usize,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_player_interstitial_event_controller_cancel_current_event_with_resumption_offset(
        controller: *mut c_void,
        value: i64,
        timescale: i32,
        kind: i32,
    );
    pub fn av_player_interstitial_event_controller_skip_current_event(controller: *mut c_void);
    pub fn av_player_waiting_during_interstitial_event_reason(
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;

    pub fn av_player_item_copy_integrated_timeline(
        item: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_player_item_integrated_timeline_release(timeline: *mut c_void);
    pub fn av_player_item_integrated_timeline_info_json(
        timeline: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_player_item_integrated_timeline_copy_current_snapshot(
        timeline: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
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
    pub fn av_player_item_integrated_timeline_seek_to_date(
        timeline: *mut c_void,
        date: *const c_char,
        out_success: *mut bool,
        out_error_message: *mut *mut c_char,
    ) -> i32;
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
    pub fn av_player_item_integrated_timeline_add_boundary_time_observer(
        timeline: *mut c_void,
        segment: *mut c_void,
        offsets_json: *const c_char,
        callback: Option<PeriodicTimeCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_player_item_integrated_timeline_add_out_of_sync_observer(
        timeline: *mut c_void,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_player_item_integrated_timeline_observer_release(observer: *mut c_void);
    pub fn av_player_item_integrated_timeline_snapshot_release(snapshot: *mut c_void);
    pub fn av_player_item_integrated_timeline_snapshot_info_json(
        snapshot: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_player_item_integrated_timeline_snapshot_copy_current_segment(
        snapshot: *mut c_void,
    ) -> *mut c_void;
    pub fn av_player_item_integrated_timeline_snapshot_segment_count(snapshot: *mut c_void)
        -> usize;
    pub fn av_player_item_integrated_timeline_snapshot_copy_segment_at_index(
        snapshot: *mut c_void,
        index: usize,
    ) -> *mut c_void;
    pub fn av_player_item_integrated_timeline_snapshot_segment_and_offset_json(
        snapshot: *mut c_void,
        value: i64,
        timescale: i32,
        kind: i32,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_player_item_integrated_timeline_segment_release(segment: *mut c_void);
    pub fn av_player_item_integrated_timeline_segment_info_json(
        segment: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_player_integrated_timeline_snapshots_out_of_sync_notification(
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_player_integrated_timeline_snapshots_out_of_sync_reason_key(
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_player_integrated_timeline_snapshots_out_of_sync_reason_segments_changed(
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_player_integrated_timeline_snapshots_out_of_sync_reason_current_segment_changed(
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_player_integrated_timeline_snapshots_out_of_sync_reason_loaded_time_ranges_changed(
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;

    pub fn av_player_media_selection_criteria_create(
        preferred_languages_json: *const c_char,
        preferred_media_characteristics_json: *const c_char,
        principal_media_characteristics_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_player_media_selection_criteria_release(criteria: *mut c_void);
    pub fn av_player_media_selection_criteria_info_json(
        criteria: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;

    pub fn av_reader_create(asset: *mut c_void, out_error_message: *mut *mut c_char)
        -> *mut c_void;
    pub fn av_reader_release(reader: *mut c_void);
    pub fn av_reader_info_json(
        reader: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
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
