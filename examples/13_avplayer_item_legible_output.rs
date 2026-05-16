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

    println!("output count after add: {}", item.output_count()?);
    println!(
        "suppresses rendering: {}",
        output.suppresses_player_rendering()?
    );
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
