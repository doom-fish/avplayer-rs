#[path = "../tests/support/mod.rs"]
mod support;

use avplayer::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let item = support::player_item("example-avplayer-item-metadata-collector")?;
    let collector = PlayerItemMetadataCollector::new(None::<&[&str]>, None::<&[&str]>)?;
    let _observer = collector.observe(Some("example-metadata-collector"), |event| {
        println!("collector event: {event:?}");
    })?;

    item.add_metadata_collector(&collector)?;
    println!(
        "collector count after add: {}",
        item.media_data_collectors()?.len()
    );
    println!("collector has delegate: {}", collector.has_delegate()?);
    println!("identifiers: {:?}", collector.identifiers()?);
    println!("classifying labels: {:?}", collector.classifying_labels()?);
    item.remove_metadata_collector(&collector);
    Ok(())
}
