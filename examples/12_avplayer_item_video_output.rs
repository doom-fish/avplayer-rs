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
        let _observer = output.observe(Some("examples.avplayer-item-video-output"), |event| {
            println!("video output event: {event:?}");
        })?;
        println!("has delegate after observe: {}", output.has_delegate()?);
        output.request_notification_of_media_data_change(0.1);
    }
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
