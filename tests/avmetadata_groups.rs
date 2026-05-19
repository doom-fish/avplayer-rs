mod support;

use avplayer::prelude::*;

#[test]
fn avmetadata_groups_and_mutable_items_round_trip() -> support::TestResult {
    let asset = support::loaded_audio_asset("test-avmetadata-groups")?;
    let items = asset.metadata()?;
    assert!(!items.is_empty());
    let time_range = TimeRange::new(Time::new(0, 1), Time::new(1, 1));

    let timed = TimedMetadataGroupHandle::new(&items, time_range)?;
    assert_eq!(timed.items()?.len(), items.len());
    assert_eq!(timed.time_range()?, time_range);
    assert_eq!(timed.as_metadata_group().items()?.len(), items.len());

    let mutable_timed = MutableTimedMetadataGroup::new(&items, time_range)?;
    mutable_timed.set_time_range(TimeRange::new(Time::new(1, 1), Time::new(2, 1)));
    mutable_timed.set_items(&items)?;
    assert_eq!(mutable_timed.items()?.len(), items.len());

    let date = DateRangeMetadataGroupHandle::new(
        &items,
        "2026-05-18T12:00:00Z",
        Some("2026-05-18T12:00:01Z"),
    )?;
    assert_eq!(date.items()?.len(), items.len());
    assert!(date.end_date()?.is_some());

    let mut mutable_date = MutableDateRangeMetadataGroup::new(
        &items,
        "2026-05-18T12:00:00Z",
        Some("2026-05-18T12:00:01Z"),
    )?;
    mutable_date.set_start_date("2026-05-18T12:00:02Z")?;
    mutable_date.set_end_date(Some("2026-05-18T12:00:03Z"))?;
    mutable_date.set_items(&items)?;
    assert_eq!(mutable_date.start_date()?, "2026-05-18T12:00:02Z");
    assert_eq!(
        mutable_date.end_date()?.as_deref(),
        Some("2026-05-18T12:00:03Z")
    );
    assert_eq!(mutable_date.items()?.len(), items.len());

    let mutable_item = MutableMetadataItem::new()?;
    mutable_item.set_string_value(Some("doomfish title"))?;
    assert_eq!(
        mutable_item.string_value()?.as_deref(),
        Some("doomfish title")
    );
    mutable_item.clear_value();
    assert!(mutable_item.string_value()?.is_none());

    let filter = MetadataItemFilter::for_sharing()?;
    let _ = filter.filter(&items)?;
    Ok(())
}
