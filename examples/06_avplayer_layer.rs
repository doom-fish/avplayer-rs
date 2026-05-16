#[path = "../tests/support/mod.rs"]
mod support;

use avplayer::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let player = support::player("example-avplayer-layer")?;
    let layer = PlayerLayer::new(Some(&player))?;
    layer.set_video_gravity(VideoGravity::Resize)?;

    println!("has player: {}", layer.has_player()?);
    println!("video gravity: {:?}", layer.video_gravity()?);
    println!("ready for display: {}", layer.is_ready_for_display()?);
    println!("video rect: {:?}", layer.video_rect()?);
    println!(
        "pixel buffer present: {}",
        layer.copy_displayed_pixel_buffer().is_some()
    );
    Ok(())
}
