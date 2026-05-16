#![allow(dead_code)]

use std::error::Error;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

use avplayer::prelude::*;

pub type TestResult = Result<(), Box<dyn Error>>;

pub fn artifacts_dir() -> Result<PathBuf, Box<dyn Error>> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/example-artifacts");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn audio_path(stem: &str) -> Result<PathBuf, Box<dyn Error>> {
    let path = artifacts_dir()?.join(format!("{stem}.aiff"));
    if path.exists() {
        fs::remove_file(&path)?;
    }

    let phrase = format!("avplayer {stem}");
    let status = Command::new("/usr/bin/say")
        .args([
            "-o",
            path.to_str().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "artifact path is not valid UTF-8",
                )
            })?,
            &phrase,
        ])
        .status()?;
    if !status.success() {
        return Err(format!("`say` failed with status {status}").into());
    }

    Ok(path)
}

pub fn loaded_audio_asset(stem: &str) -> Result<UrlAsset, Box<dyn Error>> {
    let path = audio_path(stem)?;
    let asset = UrlAsset::from_file_path(&path)?;
    asset.load_values_asynchronously(["duration", "tracks", "metadata"])?;
    Ok(asset)
}

pub fn player_item(stem: &str) -> Result<PlayerItem, Box<dyn Error>> {
    let path = audio_path(stem)?;
    Ok(PlayerItem::from_file_path(path)?)
}

pub fn player(stem: &str) -> Result<Player, Box<dyn Error>> {
    let path = audio_path(stem)?;
    Ok(Player::from_file_path(path)?)
}

pub fn player_items(stem: &str, count: usize) -> Result<Vec<PlayerItem>, Box<dyn Error>> {
    let mut items = Vec::with_capacity(count);
    for index in 0..count {
        let path = audio_path(&format!("{stem}-{index}"))?;
        items.push(PlayerItem::from_file_path(path)?);
    }
    Ok(items)
}

pub fn first_audio_track(asset: &UrlAsset) -> Result<AssetTrack, Box<dyn Error>> {
    asset
        .tracks()?
        .into_iter()
        .find(|track| matches!(track.media_type(), Ok(MediaType::Audio)))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "expected an audio track").into())
}

pub fn first_item_track(item: &PlayerItem) -> Result<PlayerItemTrack, Box<dyn Error>> {
    if let Some(track) = item.tracks()?.into_iter().next() {
        return Ok(track);
    }

    let player = Player::from_item(item)?;
    player.play();
    for _ in 0..40 {
        if let Some(track) = item.tracks()?.into_iter().next() {
            player.pause();
            return Ok(track);
        }
        thread::sleep(Duration::from_millis(50));
    }
    player.pause();

    item.tracks()?.into_iter().next().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "expected a player-item track").into()
    })
}
