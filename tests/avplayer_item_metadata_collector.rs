mod support;

use avplayer::prelude::*;

#[test]
fn avplayer_item_metadata_collector_can_be_attached() -> support::TestResult {
    let item = support::player_item("test-avplayer-item-metadata-collector")?;
    let collector = PlayerItemMetadataCollector::new(None::<&[&str]>, None::<&[&str]>)?;
    let observer = collector.observe(Some("test-metadata-collector"), |_| {})?;

    item.add_metadata_collector(&collector)?;
    assert_eq!(item.media_data_collectors()?.len(), 1);
    assert!(matches!(
        item.media_data_collectors()?[0].kind,
        PlayerItemMediaDataCollectorKind::MetadataCollector
    ));
    assert!(collector.identifiers()?.is_empty());
    assert!(collector.classifying_labels()?.is_empty());
    assert!(collector.has_delegate()?);
    assert!(matches!(
        collector.as_media_data_collector().kind()?,
        PlayerItemMediaDataCollectorKind::MetadataCollector
    ));

    drop(observer);
    item.remove_metadata_collector(&collector);
    assert!(item.media_data_collectors()?.is_empty());
    Ok(())
}
