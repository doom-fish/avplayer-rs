#[path = "../tests/support/mod.rs"]
mod support;

use avplayer::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = PlayerItemVideoOutputSettings::bgra();
    let item = support::player_item("example-avplayer-item-video-output")?;
    let output = PlayerItemVideoOutput::new(Some(&settings))?;
    item.add_video_output(&output)?;
    output.set_suppresses_player_rendering(true);

    println!("output count after add: {}", item.output_count()?);
    println!(
        "suppresses rendering: {}",
        output.suppresses_player_rendering()?
    );
    println!(
        "has new pixel buffer at t=0: {}",
        output.has_new_pixel_buffer_for_item_time(Time::new(0, 1))
    );
    println!(
        "pixel buffer available at t=0: {}",
        output
            .copy_pixel_buffer_for_item_time(Time::new(0, 1))
            .is_some()
    );
    item.remove_video_output(&output);
    Ok(())
}
