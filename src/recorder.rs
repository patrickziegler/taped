use crate::track::{TrackInfo, apply_tags, format_path};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

pub struct FinishedRecording {
    pub temp_path: PathBuf,
    pub track: TrackInfo,
}

#[async_trait]
pub trait Recording: Send + Sync {
    fn track_info(&self) -> &TrackInfo;
    async fn stop(self: Box<Self>) -> anyhow::Result<FinishedRecording>;
    async fn discard(self: Box<Self>) -> anyhow::Result<()>;
}

pub struct RealRecording {
    pub child: Child,
    pub temp_path: PathBuf,
    pub track: TrackInfo,
}

#[async_trait]
impl Recording for RealRecording {
    fn track_info(&self) -> &TrackInfo {
        &self.track
    }

    async fn stop(mut self: Box<Self>) -> anyhow::Result<FinishedRecording> {
        info!(
            "Stopping recording for {}",
            self.track.title.as_deref().unwrap_or("Unknown")
        );

        // Use SIGINT on the process group to allow ffmpeg to flush the file
        unsafe {
            if let Some(pid) = self.child.id() {
                libc::kill(-(pid as i32), libc::SIGINT);
            } else {
                return Err(anyhow::anyhow!("No PID for child"));
            }
        }

        // Wait for the process to exit
        match self.child.wait().await {
            Ok(status) => {
                if !status.success() {
                    #[cfg(unix)]
                    let is_expected = status.code() == Some(255)
                        || status.code() == Some(130)
                        || status.signal() == Some(libc::SIGINT);
                    #[cfg(not(unix))]
                    let is_expected = status.code() == Some(255) || status.code() == Some(130);

                    if !is_expected {
                        warn!("Recording process exited with status: {}", status);
                    }
                }
            }
            Err(e) => error!("Failed to wait for recording process: {}", e),
        }

        Ok(FinishedRecording {
            temp_path: self.temp_path,
            track: self.track,
        })
    }

    async fn discard(mut self: Box<Self>) -> anyhow::Result<()> {
        info!(
            "Discarding recording for {}",
            self.track.title.as_deref().unwrap_or("Unknown")
        );

        unsafe {
            if let Some(pid) = self.child.id() {
                // Kill the whole process group immediately
                libc::kill(-(pid as i32), libc::SIGKILL);
            }
        }

        // Wait for the process to exit to avoid zombies
        let _ = self.child.wait().await;

        if self.temp_path.exists() {
            if let Err(e) = tokio::fs::remove_file(&self.temp_path).await {
                error!(
                    "Failed to remove temporary file {:?}: {}",
                    self.temp_path, e
                );
            }
        }
        Ok(())
    }
}

pub async fn get_default_sink_monitor() -> Option<String> {
    let output = Command::new("pactl")
        .arg("get-default-sink")
        .output()
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let default_sink = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let monitor_name = format!("{}.monitor", default_sink);

    let output = Command::new("pactl")
        .arg("list")
        .arg("short")
        .arg("sources")
        .output()
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let sources = String::from_utf8_lossy(&output.stdout);
    for line in sources.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[1] == monitor_name {
            return Some(parts[0].to_string());
        }
    }
    None
}

pub async fn move_file(source: &Path, dest: &Path) -> std::io::Result<()> {
    if let Err(e) = tokio::fs::rename(source, dest).await {
        if e.raw_os_error() == Some(18) {
            tokio::fs::copy(source, dest).await?;
            tokio::fs::remove_file(source).await?;
            Ok(())
        } else {
            Err(e)
        }
    } else {
        Ok(())
    }
}

pub async fn exporter_task(
    mut rx: mpsc::Receiver<FinishedRecording>,
    music_dir: PathBuf,
    pattern: String,
    audio_config: crate::config::AudioConfig,
) {
    while let Some(finished) = rx.recv().await {
        let track = finished.track;
        let temp_path = finished.temp_path;

        if !temp_path.exists() {
            warn!(
                "Recording file {:?} does not exist, skipping export for {}",
                temp_path,
                track.title.as_deref().unwrap_or("Unknown")
            );
            continue;
        }

        info!(
            "Exporting: {} - {}",
            track.artist.as_deref().unwrap_or("Unknown Artist"),
            track.title.as_deref().unwrap_or("Unknown Title")
        );

        let relative_path = format_path(&pattern, &track);
        let dest_path = music_dir
            .join(relative_path)
            .with_extension(audio_config.format().to_string());

        if let Some(parent) = dest_path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }

        let track_clone = track.clone();
        let temp_path_clone = temp_path.clone();
        let tagging_result =
            tokio::task::spawn_blocking(move || apply_tags(&temp_path_clone, &track_clone)).await;

        match tagging_result {
            Ok(Err(e)) => error!("Failed to apply tags to {:?}: {}", temp_path, e),
            Err(e) => error!("Tagging task panicked: {}", e),
            _ => {}
        }

        if let Err(e) = move_file(&temp_path, &dest_path).await {
            error!("Failed to move file to {:?}: {}", dest_path, e);
        } else {
            info!("Track exported to {:?}", dest_path);
        }
    }
}
