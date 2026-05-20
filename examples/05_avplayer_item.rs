#[path = "../tests/support/mod.rs"]
mod support;

use std::thread;
use std::time::Duration;

use avplayer::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let item = support::player_item("example-avplayer-item")?;
    let _observer = item.observe(|event| {
        println!("player-item event: {event:?}");
    })?;

    item.set_can_use_network_resources_for_live_streaming_while_paused(true);
    item.set_preferred_forward_buffer_duration(1.0);
    item.set_preferred_peak_bit_rate(12_345.0);
    item.set_audio_time_pitch_algorithm(&AudioTimePitchAlgorithm::Spectral)?;

    println!("status: {:?}", item.status()?);
    println!("duration: {:?}", item.duration()?);
    println!("presentation size: {:?}", item.presentation_size()?);
    println!("track count: {}", item.track_count()?);
    println!("output count: {}", item.output_count()?);
    println!("audio pitch: {:?}", item.audio_time_pitch_algorithm()?);

    match item.variant_preferences() {
        Ok(preferences) => {
            println!("variant preferences: {preferences:?}");
            item.set_variant_preferences(
                preferences | VariantPreferences::SCALABILITY_TO_LOSSLESS_AUDIO,
            )?;
            println!(
                "updated variant preferences: {:?}",
                item.variant_preferences()?
            );
        }
        Err(error) => println!("variant preferences unavailable: {error}"),
    }

    println!(
        "protected content: required={} app={} content={} status={:?}",
        item.authorization_required_for_playback()?,
        item.application_authorized_for_playback()?,
        item.content_authorized_for_playback()?,
        item.content_authorization_request_status()?
    );
    println!(
        "custom video compositor: {:?}",
        item.custom_video_compositor()?
    );

    let player = Player::from_item(&item)?;
    player.play();
    thread::sleep(Duration::from_millis(150));
    player.seek_to(Time::new(0, 1))?;
    thread::sleep(Duration::from_millis(100));
    player.pause();
    Ok(())
}
