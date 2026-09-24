#![allow(dead_code)]

use std::error::Error;
use std::fs;
use std::io;
use std::panic;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use avplayer::prelude::*;

pub type TestResult = Result<(), Box<dyn Error>>;

pub fn run_with_deadline<F>(name: &str, limit: Duration, body: F) -> TestResult
where
    F: FnOnce() -> TestResult + Send + 'static,
{
    let (sender, receiver) = mpsc::channel();
    let worker = thread::Builder::new()
        .name(name.to_owned())
        .spawn(move || {
            let _ = sender.send(body().map_err(|error| error.to_string()));
        })?;
    match receiver.recv_timeout(limit) {
        Ok(result) => result.map_err(Into::into),
        Err(mpsc::RecvTimeoutError::Disconnected) => match worker.join() {
            Err(payload) => panic::resume_unwind(payload),
            Ok(()) => Err(format!("{name} ended without reporting a result").into()),
        },
        Err(mpsc::RecvTimeoutError::Timeout) => {
            eprintln!("{}", stack_report(name));
            Err(format!("{name} did not finish within {limit:?}").into())
        }
    }
}

fn stack_report(name: &str) -> String {
    let path = match artifacts_dir() {
        Ok(dir) => dir.join(format!("{name}-stacks.txt")),
        Err(error) => return format!("no artifacts directory for a stack report: {error}"),
    };
    let pid = std::process::id().to_string();
    let file = path.to_string_lossy().into_owned();
    match Command::new("/usr/bin/sample")
        .args([pid.as_str(), "2", "-mayDie", "-file", file.as_str()])
        .output()
    {
        Ok(output) if output.status.success() => fs::read_to_string(&path)
            .unwrap_or_else(|error| format!("could not read the stack report {file}: {error}")),
        Ok(output) => format!(
            "`sample` failed with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ),
        Err(error) => format!("could not run `sample`: {error}"),
    }
}

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
