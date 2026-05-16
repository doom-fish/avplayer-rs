use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

use avplayer::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let artifacts = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/example-artifacts");
    std::fs::create_dir_all(&artifacts)?;

    let audio_path = artifacts.join("test.aiff");
    if audio_path.exists() {
        std::fs::remove_file(&audio_path)?;
    }

    let say_status = Command::new("/usr/bin/say")
        .args([
            "-o",
            audio_path.to_str().ok_or("non-UTF-8 artifact path")?,
            "test",
        ])
        .status()?;
    if !say_status.success() {
        return Err(format!("`say` failed with status {say_status}").into());
    }

    let asset = UrlAsset::from_file_path(&audio_path)?;
    let statuses = asset.load_values_asynchronously(["duration", "tracks", "metadata"])?;
    println!("load statuses: {statuses:?}");
    println!("asset url: {}", asset.url()?);
    println!("duration: {:?}", asset.as_asset().duration()?);
    println!("metadata items: {}", asset.as_asset().metadata()?.len());

    let tracks = asset.as_asset().tracks()?;
    println!("track count: {}", tracks.len());
    let audio_track = tracks
        .iter()
        .find(|track| matches!(track.media_type(), Ok(MediaType::Audio)))
        .ok_or("expected an audio track")?;
    println!("audio track id: {}", audio_track.track_id()?);

    let reader = AssetReader::new(asset.as_asset())?;
    let output = AssetReaderTrackOutput::audio(audio_track, Some(&AudioOutputSettings::pcm_i16(44_100.0, 1)))?;
    output.set_always_copies_sample_data(false);
    assert!(reader.can_add_track_output(&output));
    reader.add_track_output(&output)?;
    reader.start_reading()?;

    let mut sample_buffers = 0usize;
    let mut samples = 0i64;
    while sample_buffers < 10 {
        let Some(buffer) = output.copy_next_sample_buffer() else {
            break;
        };
        sample_buffers += 1;
        samples += buffer.num_samples();
    }
    println!("reader sample buffers: {sample_buffers}, samples: {samples}");

    let player = Player::from_asset(asset.as_asset())?;
    println!("player status: {:?}", player.status()?);
    let current_item = player.current_item().ok_or("expected AVPlayer current item")?;
    println!("item status: {:?}", current_item.status()?);
    println!("presentation size: {:?}", current_item.presentation_size()?);
    let _observer = current_item.observe(|event| {
        println!("player-item event: {event:?}");
    })?;
    let _periodic = player.add_periodic_time_observer(Time::new(1, 2), Some("avplayer-smoke-periodic"), |time| {
        println!("periodic time: {time:?}");
    })?;
    let _boundary = player.add_boundary_time_observer(&[Time::new(1, 1)], Some("avplayer-smoke-boundary"), || {
        println!("boundary reached");
    })?;
    player.play();
    thread::sleep(Duration::from_millis(150));
    player.pause();

    println!("✅ avplayer + assetreader OK");
    Ok(())
}
