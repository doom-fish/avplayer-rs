mod support;

use avplayer::prelude::*;

#[test]
fn avplayer_item_video_output_can_be_attached() -> support::TestResult {
    let settings = PlayerItemVideoOutputSettings::bgra();
    let item = support::player_item("test-avplayer-item-video-output")?;
    let output = PlayerItemVideoOutput::new(Some(&settings))?;

    item.add_video_output(&output)?;
    assert_eq!(item.output_count()?, 1);
    assert!(!output.has_delegate()?);
    output.set_suppresses_player_rendering(true);
    assert!(output.suppresses_player_rendering()?);
    assert!(output.as_output().suppresses_player_rendering());
    let _ = output.as_output().item_time_for_host_time(0.0)?;
    let _ = output.as_output().item_time_for_mach_absolute_time(0)?;
    {
        let _observer = output.observe(Some("tests.avplayer-item-video-output"), |_| {})?;
        assert!(output.has_delegate()?);
        output.request_notification_of_media_data_change(0.1);
    }
    let _ = output.has_new_pixel_buffer_for_item_time(Time::new(0, 1));
    assert!(output
        .copy_pixel_buffer_for_item_time(Time::new(0, 1))
        .is_none());
    item.remove_video_output(&output);
    assert_eq!(item.output_count()?, 0);
    Ok(())
}
