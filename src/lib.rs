use async_trait::async_trait;
use id3::TagLike;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use zbus::zvariant::OwnedValue;

pub mod mpris;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TrackInfo {
    pub track_id: String,
    pub title: Option<String>,
    pub album: Option<String>,
    pub artist: Option<String>,
    pub album_artist: Option<String>,
    pub art_url: Option<String>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
}

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
        info!("Stopping recording for {}", self.track.title.as_deref().unwrap_or("Unknown"));
        
        // Use SIGINT to allow ffmpeg to flush the file
        unsafe {
            let pid = self.child.id().ok_or_else(|| anyhow::anyhow!("No PID for child"))? as i32;
            libc::kill(-pid, libc::SIGINT);
        }

        match self.child.wait().await {
            Ok(status) => {
                if !status.success() {
                    warn!("Recording process exited with status: {}", status);
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
        info!("Discarding recording for {}", self.track.title.as_deref().unwrap_or("Unknown"));
        let _ = self.child.kill().await;
        if self.temp_path.exists() {
            tokio::fs::remove_file(&self.temp_path).await?;
        }
        Ok(())
    }
}

pub struct Watchdog {
    pub current_recording: Option<Box<dyn Recording>>,
    pub previous_recording: Option<Box<dyn Recording>>,
    pub exporter_tx: mpsc::Sender<FinishedRecording>,
    pub recording_enabled: bool,
    pub waiting_for_next_track: bool,
    pub playback_status: String,
}

impl Watchdog {
    pub fn new(exporter_tx: mpsc::Sender<FinishedRecording>, recording_enabled: bool) -> Self {
        Self {
            current_recording: None,
            previous_recording: None,
            exporter_tx,
            recording_enabled,
            waiting_for_next_track: recording_enabled,
            playback_status: "Unknown".to_string(),
        }
    }

    pub async fn handle_metadata_update(&mut self, track: TrackInfo) {
        if let Some(current) = &self.current_recording {
            if current.track_info().track_id == track.track_id {
                return;
            }
        }

        info!("Track transition to: {}", track.title.as_deref().unwrap_or("Unknown"));

        // Move current to previous and start finalizing it
        if let Some(recording) = self.current_recording.take() {
            self.previous_recording = Some(recording);
            self.finalize_previous().await;
        }

        if self.recording_enabled {
            if self.waiting_for_next_track {
                info!("Recording enabled, waiting for next track to start capture.");
                self.waiting_for_next_track = false;
                return;
            }

            if self.playback_status == "Playing" {
                self.start_new_recording(track).await;
            } else {
                info!("Spotify is {}, skipping recording start for {}", self.playback_status, track.title.as_deref().unwrap_or("Unknown"));
            }
        }
    }

    pub async fn handle_playback_status(&mut self, status: &str, current_track: Option<TrackInfo>) {
        if self.playback_status == status {
            return;
        }
        info!("Playback status changed: {}", status);
        self.playback_status = status.to_string();

        if status == "Playing" {
            if self.current_recording.is_none() && self.recording_enabled && !self.waiting_for_next_track {
                if let Some(track) = current_track {
                    self.start_new_recording(track).await;
                }
            }
        } else {
            // Stopped or Paused
            if let Some(recording) = self.current_recording.take() {
                self.previous_recording = Some(recording);
                self.finalize_previous().await;
            }
        }
    }

    async fn finalize_previous(&mut self) {
        if let Some(recording) = self.previous_recording.take() {
            let tx = self.exporter_tx.clone();
            tokio::spawn(async move {
                match recording.stop().await {
                    Ok(finished) => {
                        let _ = tx.send(finished).await;
                    }
                    Err(e) => error!("Failed to stop recording: {}", e),
                }
            });
        }
    }

    async fn start_new_recording(&mut self, track: TrackInfo) {
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join(format!("spotify_rec_{}.mp3", uuid::Uuid::new_v4()));
        
        info!("Starting recording for {} to {:?}", track.title.as_deref().unwrap_or("Unknown"), temp_path);

        let target_node = get_default_sink_monitor().await;
        let mut cmd = Command::new("sh");
        
        let pw_cat_target = if let Some(node) = target_node {
            format!("--target {}", node)
        } else {
            "".to_string()
        };

        let shell_cmd = format!(
            "pw-cat --record {} --format s16 --rate 48000 --channels 2 - | ffmpeg -y -f s16le -ar 48000 -ac 2 -i pipe:0 -codec:a libmp3lame -qscale:a 2 {:?}",
            pw_cat_target, temp_path
        );

        cmd.arg("-c").arg(shell_cmd);

        match cmd.spawn() {
            Ok(child) => {
                self.current_recording = Some(Box::new(RealRecording {
                    child,
                    temp_path,
                    track,
                }));
            }
            Err(e) => error!("Failed to start recording process: {}", e),
        }
    }

    pub async fn set_recording_enabled(&mut self, enabled: bool) {
        if self.recording_enabled != enabled {
            self.recording_enabled = enabled;
            if !enabled {
                if let Some(recording) = self.current_recording.take() {
                    let _ = recording.discard().await;
                }
                self.waiting_for_next_track = false;
            } else {
                self.waiting_for_next_track = true;
            }
        }
    }
}

async fn get_default_sink_monitor() -> Option<String> {
    let output = Command::new("pactl").arg("get-default-sink").output().await.ok()?;
    if !output.status.success() {
        return None;
    }
    let default_sink = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let monitor_name = format!("{}.monitor", default_sink);

    let output = Command::new("pactl").arg("list").arg("short").arg("sources").output().await.ok()?;
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

pub async fn exporter_task(
    mut rx: mpsc::Receiver<FinishedRecording>,
    music_dir: PathBuf,
    pattern: String,
) {
    while let Some(finished) = rx.recv().await {
        let track = finished.track;
        let temp_path = finished.temp_path;

        if !temp_path.exists() {
            warn!("Recording file {:?} does not exist, skipping export for {}", temp_path, track.title.as_deref().unwrap_or("Unknown"));
            continue;
        }

        info!("Exporting: {} - {}", 
            track.artist.as_deref().unwrap_or("Unknown Artist"),
            track.title.as_deref().unwrap_or("Unknown Title")
        );

        let relative_path = format_path(&pattern, &track);
        let dest_path = music_dir.join(relative_path).with_extension("mp3");
        
        if let Some(parent) = dest_path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }

        let track_clone = track.clone();
        let temp_path_clone = temp_path.clone();
        let _ = tokio::task::spawn_blocking(move || {
            apply_tags(&temp_path_clone, &track_clone)
        }).await;

        if let Err(e) = move_file(&temp_path, &dest_path).await {
            error!("Failed to move file to {:?}: {}", dest_path, e);
        } else {
            info!("Track exported to {:?}", dest_path);
        }
    }
}

fn format_path(pattern: &str, track: &TrackInfo) -> String {
    pattern
        .replace("{title}", track.title.as_deref().unwrap_or("Unknown Title"))
        .replace("{artist}", track.artist.as_deref().unwrap_or("Unknown Artist"))
        .replace("{album}", track.album.as_deref().unwrap_or("Unknown Album"))
        .replace("{albumArtist}", track.album_artist.as_deref().unwrap_or(track.artist.as_deref().unwrap_or("Unknown Artist")))
        .replace("{trackNumber}", &format!("{:02}", track.track_number.unwrap_or(0)))
        .replace("{discNumber}", &track.disc_number.unwrap_or(1).to_string())
}

async fn move_file(source: &Path, dest: &Path) -> std::io::Result<()> {
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

fn apply_tags(path: &Path, track: &TrackInfo) -> anyhow::Result<()> {
    let mut tag = id3::Tag::new();
    tag.set_title(track.title.as_deref().unwrap_or("Unknown Title"));
    tag.set_artist(track.artist.as_deref().unwrap_or("Unknown Artist"));
    tag.set_album(track.album.as_deref().unwrap_or("Unknown Album"));
    if let Some(artist) = &track.album_artist {
        tag.set_album_artist(artist);
    }
    if let Some(n) = track.track_number {
        tag.set_track(n as u32);
    }
    if let Some(n) = track.disc_number {
        tag.set_disc(n as u32);
    }

    if let Some(art_url) = &track.art_url {
        if let Ok(response) = reqwest::blocking::get(art_url) {
            if let Ok(bytes) = response.bytes() {
                tag.add_frame(id3::frame::Picture {
                    mime_type: "image/jpeg".to_string(),
                    picture_type: id3::frame::PictureType::CoverFront,
                    description: "Album Art".to_string(),
                    data: bytes.to_vec(),
                });
            }
        }
    }

    tag.write_to_path(path, id3::Version::Id3v24)?;
    Ok(())
}

pub fn parse_track_info(metadata: &HashMap<String, OwnedValue>) -> TrackInfo {
    let mut track = TrackInfo::default();

    if let Some(v) = metadata.get("mpris:trackid") {
        if let Ok(s) = v.downcast_ref::<&str>() {
            track.track_id = s.to_string();
        }
    }

    track.title = metadata.get("xesam:title").and_then(|v| v.downcast_ref::<&str>().ok().map(|s| s.to_string()));
    track.album = metadata.get("xesam:album").and_then(|v| v.downcast_ref::<&str>().ok().map(|s| s.to_string()));
    track.art_url = metadata.get("mpris:artUrl").and_then(|v| v.downcast_ref::<&str>().ok().map(|s| s.to_string()));
    track.track_number = metadata.get("xesam:trackNumber").and_then(|v| v.downcast_ref::<i32>().ok());
    track.disc_number = metadata.get("xesam:discNumber").and_then(|v| v.downcast_ref::<i32>().ok());

    track.artist = metadata.get("xesam:artist").and_then(|v| {
        v.downcast_ref::<zbus::zvariant::Array>().ok().map(|array| {
            array.iter().filter_map(|val| {
                let s: Result<&str, _> = val.try_into();
                s.ok().map(|s| s.to_string())
            }).collect::<Vec<String>>().join(", ")
        })
    });

    track.album_artist = metadata.get("xesam:albumArtist").and_then(|v| {
        v.downcast_ref::<zbus::zvariant::Array>().ok().map(|array| {
            array.iter().filter_map(|val| {
                let s: Result<&str, _> = val.try_into();
                s.ok().map(|s| s.to_string())
            }).collect::<Vec<String>>().join(", ")
        })
    });

    track
}

// Placeholder for ServiceControl and run_service which will be integrated next
pub struct ServiceControl {
    pub recording_enabled: bool,
    pub connection_status: ConnectionStatus,
    pub current_track: Option<TrackInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, zbus::zvariant::Type, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum ConnectionStatus {
    Disconnected = 0,
    Connected = 1,
}

impl ServiceControl {
    pub fn new(recording_enabled: bool) -> Self {
        Self {
            recording_enabled,
            connection_status: ConnectionStatus::Disconnected,
            current_track: None,
        }
    }
}

#[zbus::interface(name = "org.spotify_recorder.Control")]
impl ServiceControl {
    #[zbus(property)]
    fn recording_enabled(&self) -> bool {
        self.recording_enabled
    }

    #[zbus(property)]
    pub async fn set_recording_enabled(&mut self, #[zbus(signal_emitter)] emitter: zbus::object_server::SignalEmitter<'_>, enabled: bool) {
        if self.recording_enabled != enabled {
            self.recording_enabled = enabled;
            self.recording_enabled_changed(&emitter).await.unwrap_or_default();
        }
    }

    #[zbus(property)]
    fn connection_status(&self) -> String {
        match self.connection_status {
            ConnectionStatus::Connected => "Connected".to_string(),
            ConnectionStatus::Disconnected => "Disconnected".to_string(),
        }
    }

    #[zbus(property)]
    fn current_song(&self) -> String {
        self.current_track.as_ref().map(|t| format!("{} - {}", t.artist.as_deref().unwrap_or("Unknown Artist"), t.title.as_deref().unwrap_or("Unknown Title"))).unwrap_or_else(|| "None".to_string())
    }
}

pub async fn run_service(
    connection: zbus::Connection,
    spotify_bus_name: &str,
    music_dir: PathBuf,
    pattern: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel(100);
    tokio::spawn(exporter_task(rx, music_dir, pattern));

    let dbus_proxy = zbus::fdo::DBusProxy::new(&connection).await?;
    let mut name_owner_changed = dbus_proxy.receive_name_owner_changed().await?;

    let mut monitor_handle: Option<tokio::task::JoinHandle<()>> = None;

    // Initial check
    let spotify_bus_name_owned = zbus::names::BusName::try_from(spotify_bus_name)?;
    if let Ok(owner) = dbus_proxy.get_name_owner(spotify_bus_name_owned).await {
        info!("Spotify found: {}", owner);
        update_connection_status(&connection, ConnectionStatus::Connected).await;
        monitor_handle = Some(tokio::spawn(monitor_spotify(
            connection.clone(),
            spotify_bus_name.to_string(),
            tx.clone(),
        )));
    }

    while let Some(signal) = futures_util::StreamExt::next(&mut name_owner_changed).await {
        let args = signal.args()?;
        if args.name() == spotify_bus_name {
            if let Some(_new_owner) = args.new_owner().as_ref() {
                info!("Spotify appeared");
                if let Some(handle) = monitor_handle.take() {
                    handle.abort();
                }
                update_connection_status(&connection, ConnectionStatus::Connected).await;
                monitor_handle = Some(tokio::spawn(monitor_spotify(
                    connection.clone(),
                    spotify_bus_name.to_string(),
                    tx.clone(),
                )));
            } else {
                warn!("Spotify disappeared");
                if let Some(handle) = monitor_handle.take() {
                    handle.abort();
                }
                update_connection_status(&connection, ConnectionStatus::Disconnected).await;
            }
        }
    }

    Ok(())
}

async fn update_connection_status(connection: &zbus::Connection, status: ConnectionStatus) {
    if let Ok(iface_ref) = connection
        .object_server()
        .interface::<_, ServiceControl>("/org/spotify_recorder/Control")
        .await
    {
        let mut iface = iface_ref.get_mut().await;
        if iface.connection_status != status {
            iface.connection_status = status;
            if status == ConnectionStatus::Disconnected {
                iface.current_track = None;
                iface.current_song_changed(iface_ref.signal_emitter()).await.unwrap_or_default();
            }
            iface.connection_status_changed(iface_ref.signal_emitter()).await.unwrap_or_default();
        }
    }
}

pub async fn monitor_spotify(
    connection: zbus::Connection,
    bus_name: String,
    tx: mpsc::Sender<FinishedRecording>,
) {
    update_connection_status(&connection, ConnectionStatus::Connected).await;
    let player_proxy = match crate::mpris::PlayerProxy::builder(&connection)
        .destination(bus_name)
        .unwrap()
        .build()
        .await
    {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to create PlayerProxy: {}", e);
            return;
        }
    };

    let mut metadata_stream = player_proxy.receive_metadata_changed().await;
    let mut playback_stream = player_proxy.receive_playback_status_changed().await;

    // Get initial recording_enabled state
    let initial_recording_enabled = if let Ok(iface_ref) = connection
        .object_server()
        .interface::<_, ServiceControl>("/org/spotify_recorder/Control")
        .await
    {
        let guard = iface_ref.get().await;
        guard.recording_enabled
    } else {
        true
    };

    let mut watchdog = Watchdog::new(tx, initial_recording_enabled);

    // Initial state
    if let Ok(metadata) = player_proxy.metadata().await {
        let track = parse_track_info(&metadata);
        update_service_current_track(&connection, Some(track.clone())).await;
        watchdog.handle_metadata_update(track).await;
    }
    if let Ok(status) = player_proxy.playback_status().await {
        let current_track = if let Ok(metadata) = player_proxy.metadata().await {
            Some(parse_track_info(&metadata))
        } else {
            None
        };
        watchdog.handle_playback_status(&status, current_track).await;
    }

    loop {
        tokio::select! {
            Some(_) = futures_util::StreamExt::next(&mut metadata_stream) => {
                if let Ok(iface_ref) = connection.object_server().interface::<_, ServiceControl>("/org/spotify_recorder/Control").await {
                    let guard = iface_ref.get().await;
                    watchdog.set_recording_enabled(guard.recording_enabled).await;
                }
                if let Ok(metadata) = player_proxy.metadata().await {
                    let track = parse_track_info(&metadata);
                    update_service_current_track(&connection, Some(track.clone())).await;
                    watchdog.handle_metadata_update(track).await;
                }
            }
            Some(_) = futures_util::StreamExt::next(&mut playback_stream) => {
                if let Ok(iface_ref) = connection.object_server().interface::<_, ServiceControl>("/org/spotify_recorder/Control").await {
                    let guard = iface_ref.get().await;
                    watchdog.set_recording_enabled(guard.recording_enabled).await;
                }
                if let Ok(status) = player_proxy.playback_status().await {
                    let current_track = if let Ok(metadata) = player_proxy.metadata().await {
                        Some(parse_track_info(&metadata))
                    } else {
                        None
                    };
                    watchdog.handle_playback_status(&status, current_track).await;
                }
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(500)) => {
                // Sync recording_enabled from ServiceControl to Watchdog
                if let Ok(iface_ref) = connection
                    .object_server()
                    .interface::<_, ServiceControl>("/org/spotify_recorder/Control")
                    .await
                {
                    let guard = iface_ref.get().await;
                    watchdog.set_recording_enabled(guard.recording_enabled).await;
                }
            }
        }
    }
}

async fn update_service_current_track(connection: &zbus::Connection, track: Option<TrackInfo>) {
    if let Ok(iface_ref) = connection
        .object_server()
        .interface::<_, ServiceControl>("/org/spotify_recorder/Control")
        .await
    {
        let mut iface = iface_ref.get_mut().await;
        if iface.current_track != track {
            iface.current_track = track;
            iface.current_song_changed(iface_ref.signal_emitter()).await.unwrap_or_default();
        }
    }
}
