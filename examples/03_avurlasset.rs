#[path = "../tests/support/mod.rs"]
mod support;

use avplayer::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = support::audio_path("example-avurlasset")?;
    let options = UrlAssetOptions::new().prefer_precise_duration_and_timing(false);
    let asset = UrlAsset::from_file_path_with_options(&path, options)?;
    let statuses = asset.load_values_asynchronously(["duration", "tracks"])?;

    println!("url asset url: {}", asset.url()?);
    println!("duration: {:?}", asset.duration()?);
    println!("status(duration): {:?}", asset.status_of_value("duration")?);
    println!("loaded statuses: {statuses:?}");
    Ok(())
}
