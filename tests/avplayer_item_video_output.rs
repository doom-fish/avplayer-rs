mod support;

use avplayer::prelude::*;

#[test]
fn avplayer_item_video_output_can_be_attached() -> support::TestResult {
    let settings = PlayerItemVideoOutputSettings::bgra();
    let item = support::player_item("test-avplayer-item-video-output")?;
    let output = PlayerItemVideoOutput::new(Some(&settings))?;

    item.add_video_output(&output)?;
    assert_eq!(item.output_count()?, 1);
    output.set_suppresses_player_rendering(true);
    assert!(output.suppresses_player_rendering()?);
    let _ = output.has_new_pixel_buffer_for_item_time(Time::new(0, 1));
    assert!(output
        .copy_pixel_buffer_for_item_time(Time::new(0, 1))
        .is_none());
    item.remove_video_output(&output);
    assert_eq!(item.output_count()?, 0);
    Ok(())
}
