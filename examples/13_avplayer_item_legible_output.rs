#[path = "../tests/support/mod.rs"]
mod support;

use avplayer::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subtypes: [u32; 0] = [];
    let item = support::player_item("example-avplayer-item-legible-output")?;
    let output = PlayerItemLegibleOutput::new(Some(&subtypes))?;
    item.add_legible_output(&output)?;
    output.set_suppresses_player_rendering(false);
    output.set_advance_interval_for_delegate_invocation(0.5);
    output.set_text_styling_resolution(
        &PlayerItemLegibleOutputTextStylingResolution::SourceAndRulesOnly,
    )?;

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
    println!(
        "text styling resolution: {:?}",
        output.text_styling_resolution()?
    );
    {
        let _observer = output.observe(Some("examples.avplayer-item-legible-output"), |event| {
            println!("legible output event: {event:?}");
        })?;
        println!("has delegate after observe: {}", output.has_delegate()?);
    }
    println!(
        "advance interval: {}",
        output.advance_interval_for_delegate_invocation()?
    );
    println!(
        "native subtypes: {:?}",
        output.native_representation_subtypes()?
    );
    item.remove_legible_output(&output);
    Ok(())
}
