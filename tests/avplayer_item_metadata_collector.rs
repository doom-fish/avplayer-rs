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
    assert_eq!(collector.identifiers()?, [] as [std::string::String; 0]);
    assert_eq!(collector.classifying_labels()?, [] as [std::string::String; 0]);
    assert!(collector.has_delegate()?);
    assert!(matches!(
        collector.as_media_data_collector().kind()?,
        PlayerItemMediaDataCollectorKind::MetadataCollector
    ));

    drop(observer);
    item.remove_metadata_collector(&collector);
    assert_eq!(item.media_data_collectors()?, [] as [avplayer::PlayerItemMediaDataCollectorInfo; 0]);
    Ok(())
}
