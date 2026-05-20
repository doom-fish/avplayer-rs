#[path = "../tests/support/mod.rs"]
mod support;

use avplayer::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let player = support::player("example-avplayer-video-output")?;
    let mono = PlayerVideoOutputTagCollection::from_preset(
        PlayerVideoOutputTagCollectionPreset::Monoscopic,
    )?;
    let spec = VideoOutputSpecification::new(&[&mono])?;
    let settings = PlayerVideoOutputSettings::bgra();
    spec.set_default_output_settings(Some(&settings))?;
    spec.set_output_settings_for_tag_collection(&mono, Some(&settings))?;

    let output = PlayerVideoOutput::new(&spec)?;
    player.set_video_output(Some(&output));

    println!(
        "preferred tag collections: {:?}",
        spec.preferred_tag_collections()?
    );
    println!("tag collection tags: {:?}", mono.tags()?);
    println!(
        "sample at host t=0 available: {}",
        output.sample_for_host_time(Time::new(0, 1))?.is_some()
    );

    player.set_video_output(None);
    Ok(())
}
