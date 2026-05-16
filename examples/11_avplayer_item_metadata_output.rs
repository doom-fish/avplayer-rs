#[path = "../tests/support/mod.rs"]
mod support;

use avplayer::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let item = support::player_item("example-avplayer-item-metadata-output")?;
    let output = PlayerItemMetadataOutput::new(None::<&[&str]>)?;
    item.add_metadata_output(&output)?;
    output.set_suppresses_player_rendering(false);
    output.set_advance_interval_for_delegate_invocation(0.25);

    println!("output count after add: {}", item.output_count()?);
    println!(
        "suppresses rendering: {}",
        output.suppresses_player_rendering()?
    );
    println!(
        "advance interval: {}",
        output.advance_interval_for_delegate_invocation()?
    );
    println!("identifiers: {:?}", output.identifiers()?);
    item.remove_metadata_output(&output);
    Ok(())
}
