mod support;

use avplayer::prelude::*;

#[test]
fn avsample_buffer_display_layer_exposes_basic_surface() -> support::TestResult {
    let layer = SampleBufferDisplayLayer::new()?;

    assert_eq!(layer.status()?, QueuedSampleBufferRenderingStatus::Unknown);
    assert!(layer.error()?.is_none());
    layer.set_video_gravity(VideoGravity::ResizeAspectFill)?;
    assert_eq!(layer.video_gravity()?, VideoGravity::ResizeAspectFill);
    let _ = layer.is_ready_for_more_media_data()?;
    let _ = layer.is_ready_for_display()?;
    let _ = layer.has_sufficient_media_data_for_reliable_playback_start()?;
    let _ = layer.requires_flush_to_resume_decoding()?;

    layer.set_prevents_capture(true);
    assert!(layer.prevents_capture()?);
    layer.set_prevents_display_sleep_during_video_playback(true);
    assert!(layer.prevents_display_sleep_during_video_playback()?);

    layer.stop_requesting_media_data();
    layer.flush();
    layer.flush_and_remove_image();
    Ok(())
}
