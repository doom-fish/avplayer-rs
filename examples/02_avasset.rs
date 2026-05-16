#[path = "../tests/support/mod.rs"]
mod support;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let asset = support::loaded_audio_asset("example-avasset")?;
    let track = support::first_audio_track(&asset)?;

    println!("asset url: {:?}", asset.as_asset().url()?);
    println!("duration: {:?}", asset.as_asset().duration()?);
    println!("metadata items: {}", asset.as_asset().metadata()?.len());
    println!("track id: {}", track.track_id()?);
    println!("media type: {:?}", track.media_type()?);
    println!("natural size: {:?}", track.natural_size()?);
    Ok(())
}
