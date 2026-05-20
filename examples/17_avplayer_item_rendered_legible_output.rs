#[path = "../tests/support/mod.rs"]
mod support;

use avplayer::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let item = support::player_item("example-avplayer-item-rendered-legible-output")?;
    let output = PlayerItemRenderedLegibleOutput::new(Size {
        width: 640.0,
        height: 360.0,
    })?;
    let _observer = output.observe(Some("example-rendered-legible-output"), |event| {
        println!("delegate event: {event:?}");
    })?;
    item.add_rendered_legible_output(&output)?;
    output.set_advance_interval_for_delegate_invocation(0.25);
    output.set_video_display_size(Size {
        width: 960.0,
        height: 540.0,
    });

    println!("output count after add: {}", item.output_count()?);
    println!(
        "suppresses rendering: {}",
        output.suppresses_player_rendering()?
    );
    println!(
        "advance interval: {}",
        output.advance_interval_for_delegate_invocation()?
    );
    println!("video display size: {:?}", output.video_display_size()?);
    item.remove_rendered_legible_output(&output);
    Ok(())
}
