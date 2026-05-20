mod support;

use avplayer::prelude::*;

#[test]
fn avplayer_item_integrated_timeline_surfaces_smoke() -> support::TestResult {
    let item = support::player_item("test-avplayer-item-integrated-timeline")?;
    let _player = Player::from_item(&item)?;
    let timeline = item.integrated_timeline()?;

    let info = timeline.info()?;
    let snapshot = timeline.current_snapshot()?;
    let snapshot_info = snapshot.info()?;
    assert_eq!(snapshot.segment_count(), snapshot_info.segments.len());
    assert!(!player_integrated_timeline_snapshots_out_of_sync_notification()?.is_empty());
    assert!(!player_integrated_timeline_snapshots_out_of_sync_reason_key()?.is_empty());
    assert!(
        !player_integrated_timeline_snapshots_out_of_sync_reason_segments_changed()?.is_empty()
    );
    assert!(
        !player_integrated_timeline_snapshots_out_of_sync_reason_current_segment_changed()?
            .is_empty()
    );
    assert!(
        !player_integrated_timeline_snapshots_out_of_sync_reason_loaded_time_ranges_changed()?
            .is_empty()
    );

    let _periodic = timeline.observe_periodic_times(Time::new(1, 10), |_| {})?;
    let _out_of_sync = timeline.observe_snapshots_out_of_sync(|_| {})?;
    let _ = timeline.seek_to_time(Time::new(0, 1), Time::new(0, 1), Time::new(0, 1));
    if let Some(current_date) = info.current_date.as_deref() {
        let _ = timeline.seek_to_date(current_date);
    }

    if let Some(segment) = snapshot.segment_at_index(0) {
        let segment_info = segment.info()?;
        let _boundary = timeline.observe_boundary_times(&segment, &[Time::new(0, 1)], |_| {})?;
        let _ = snapshot.segment_and_offset_into_segment(segment_info.time_mapping_target.start)?;
    }

    if let Some(current_segment) = snapshot.current_segment() {
        let _ = current_segment.info()?;
    }

    Ok(())
}
