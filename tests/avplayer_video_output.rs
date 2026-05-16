mod support;

use avplayer::prelude::*;

#[test]
fn avplayer_video_output_can_be_configured() -> support::TestResult {
    let player = support::player("test-avplayer-video-output")?;
    let mono = PlayerVideoOutputTagCollection::from_preset(
        PlayerVideoOutputTagCollectionPreset::Monoscopic,
    )?;
    let stereo = PlayerVideoOutputTagCollection::from_preset(
        PlayerVideoOutputTagCollectionPreset::Stereoscopic,
    )?;
    assert_eq!(mono.tags()?.len(), 1);
    assert_eq!(stereo.tags()?.len(), 1);

    let spec = VideoOutputSpecification::new(&[&mono, &stereo])?;
    assert_eq!(spec.preferred_tag_collections()?.len(), 2);

    let settings = PlayerVideoOutputSettings::bgra();
    spec.set_default_output_settings(Some(&settings))?;
    spec.set_output_settings_for_tag_collection(&mono, Some(&settings))?;

    let output = PlayerVideoOutput::new(&spec)?;
    player.set_video_output(Some(&output));
    let attached = player.video_output();
    assert!(attached.is_some());
    assert!(attached
        .as_ref()
        .expect("video output should be attached")
        .sample_for_host_time(Time::new(0, 1))?
        .is_none());

    player.set_video_output(None);
    assert!(player.video_output().is_none());
    Ok(())
}
