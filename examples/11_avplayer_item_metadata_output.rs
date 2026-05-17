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
    println!("has delegate before observe: {}", output.has_delegate()?);
    println!(
        "suppresses rendering: {}",
        output.suppresses_player_rendering()?
    );
    println!(
        "base output suppresses rendering: {}",
        output.as_output().suppresses_player_rendering()
    );
    println!(
        "item time for host time 0: {:?}",
        output.as_output().item_time_for_host_time(0.0)?
    );
    println!(
        "item time for mach absolute time 0: {:?}",
        output.as_output().item_time_for_mach_absolute_time(0)?
    );
    {
        let _observer = output.observe(Some("examples.avplayer-item-metadata-output"), |event| {
            println!("metadata output event: {event:?}");
        })?;
        println!("has delegate after observe: {}", output.has_delegate()?);
    }
    println!(
        "advance interval: {}",
        output.advance_interval_for_delegate_invocation()?
    );
    println!("identifiers: {:?}", output.identifiers()?);
    item.remove_metadata_output(&output);
    Ok(())
}
