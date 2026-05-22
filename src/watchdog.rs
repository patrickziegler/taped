use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use zbus::proxy;
use crate::track::{TrackInfo, parse_track_info};
use crate::recorder::{Recording, RealRecording, FinishedRecording, get_default_sink_monitor};

#[proxy(
    interface = "org.mpris.MediaPlayer2.Player",
    default_service = "org.mpris.MediaPlayer2.spotify",
    default_path = "/org/mpris/MediaPlayer2"
)]
pub trait Player {
    #[zbus(property)]
    fn metadata(&self) -> zbus::Result<std::collections::HashMap<String, zbus::zvariant::OwnedValue>>;

    #[zbus(property)]
    fn playback_status(&self) -> zbus::Result<String>;
}

pub struct Watchdog {
    pub current_recording: Option<Box<dyn Recording>>,
    pub previous_recording: Option<Box<dyn Recording>>,
    pub exporter_tx: mpsc::Sender<FinishedRecording>,
    pub waiting_for_next_track: bool,
    pub last_track_id: Option<String>,
    pub playback_status: String,
}

impl Watchdog {
    pub fn new(exporter_tx: mpsc::Sender<FinishedRecording>) -> Self {
        Self {
            current_recording: None,
            previous_recording: None,
            exporter_tx,
            waiting_for_next_track: true,
            last_track_id: None,
            playback_status: "Unknown".to_string(),
        }
    }

    pub async fn handle_metadata_update(&mut self, track: TrackInfo) {
        let is_transition = if let Some(last_id) = &self.last_track_id {
            *last_id != track.track_id
        } else {
            true
        };

        if !is_transition {
            return;
        }

        let old_track_id = self.last_track_id.take();
        self.last_track_id = Some(track.track_id.clone());

        if self.waiting_for_next_track {
            if old_track_id.is_none() {
                info!("Initial track detected: {}. Waiting for next track transition to start capture.", track.title.as_deref().unwrap_or("Unknown"));
                return;
            } else {
                info!("First track transition detected. Starting capture from now on.");
                self.waiting_for_next_track = false;
            }
        }

        info!("Track transition to: {}", track.title.as_deref().unwrap_or("Unknown"));

        // Move current to previous and start finalizing it
        if let Some(recording) = self.current_recording.take() {
            self.previous_recording = Some(recording);
            self.finalize_previous().await;
        }

        if track.is_ad() {
            info!(
                "Ad/Invalid track detected (ID: {}, Title: {}). Skipping recording.",
                track.track_id,
                track.title.as_deref().unwrap_or("Unknown")
            );
            return;
        }

        if self.playback_status == "Playing" {
            self.start_new_recording(track).await;
        } else {
            info!("Spotify is {}, skipping recording start for {}", self.playback_status, track.title.as_deref().unwrap_or("Unknown"));
        }
    }

    pub async fn handle_playback_status(&mut self, status: &str, current_track: Option<TrackInfo>) {
        if self.playback_status == status {
            return;
        }
        info!("Playback status changed: {}", status);
        self.playback_status = status.to_string();

        if status == "Playing" {
            if self.current_recording.is_none() && !self.waiting_for_next_track {
                if let Some(track) = current_track {
                    if track.is_ad() {
                        info!("Spotify is playing an ad, skipping recording");
                    } else {
                        self.start_new_recording(track).await;
                    }
                }
            }
        } else {
            // Stopped or Paused
            if let Some(recording) = self.current_recording.take() {
                info!("Playback {} - discarding current recording", status);
                tokio::spawn(async move {
                    if let Err(e) = recording.discard().await {
                        error!("Failed to discard recording: {}", e);
                    }
                });
            }
            self.waiting_for_next_track = true;
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

        cmd.arg("-c").arg(shell_cmd)
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        // Important: start in a new process group so we can kill the whole group (pw-cat + ffmpeg)
        #[cfg(unix)]
        unsafe {
            cmd.pre_exec(|| {
                libc::setpgid(0, 0);
                Ok(())
            });
        }

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
}

pub async fn monitor_spotify(
    connection: zbus::Connection,
    bus_name: String,
    tx: mpsc::Sender<FinishedRecording>,
    mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
) {
    let player_proxy = match PlayerProxy::builder(&connection)
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

    let mut watchdog = Watchdog::new(tx);

    // Initial state
    if let Ok(metadata) = player_proxy.metadata().await {
        let track = parse_track_info(&metadata);
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
                if let Ok(metadata) = player_proxy.metadata().await {
                    let track = parse_track_info(&metadata);
                    watchdog.handle_metadata_update(track).await;
                }
            }
            Some(_) = futures_util::StreamExt::next(&mut playback_stream) => {
                if let Ok(_status) = player_proxy.playback_status().await {
                    let _current_track = if let Ok(metadata) = player_proxy.metadata().await {
                        Some(parse_track_info(&metadata))
                    } else {
                        None
                    };
                    watchdog.handle_playback_status(&_status, _current_track).await;
                }
            }
            _ = shutdown_rx.recv() => {
                if let Some(recording) = watchdog.current_recording.take() {
                    let _ = recording.discard().await;
                }
                return;
            }
        }
    }
}

pub async fn run_service(
    connection: zbus::Connection,
    spotify_bus_name: &str,
    music_dir: PathBuf,
    pattern: String,
    shutdown_rx: tokio::sync::broadcast::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut main_shutdown_rx = shutdown_rx;
    let (tx, rx) = mpsc::channel(100);
    tokio::spawn(crate::recorder::exporter_task(rx, music_dir, pattern));

    let dbus_proxy = zbus::fdo::DBusProxy::new(&connection).await?;
    let mut name_owner_changed = dbus_proxy.receive_name_owner_changed().await?;

    let mut monitor_handle: Option<tokio::task::JoinHandle<()>> = None;

    // Initial check
    let spotify_bus_name_owned = zbus::names::BusName::try_from(spotify_bus_name)?;
    if let Ok(owner) = dbus_proxy.get_name_owner(spotify_bus_name_owned).await {
        info!("Spotify found: {}", owner);
        monitor_handle = Some(tokio::spawn(monitor_spotify(
            connection.clone(),
            spotify_bus_name.to_string(),
            tx.clone(),
            main_shutdown_rx.resubscribe(),
        )));
    }

    loop {
        tokio::select! {
            Some(signal) = futures_util::StreamExt::next(&mut name_owner_changed) => {
                let args = signal.args()?;
                if args.name() == spotify_bus_name {
                    if let Some(_new_owner) = args.new_owner().as_ref() {
                        info!("Spotify appeared");
                        if let Some(handle) = monitor_handle.take() {
                            handle.abort();
                        }
                        monitor_handle = Some(tokio::spawn(monitor_spotify(
                            connection.clone(),
                            spotify_bus_name.to_string(),
                            tx.clone(),
                            main_shutdown_rx.resubscribe(),
                        )));
                    } else {
                        warn!("Spotify disappeared");
                        if let Some(handle) = monitor_handle.take() {
                            handle.abort();
                        }
                    }
                }
            }
            _ = main_shutdown_rx.recv() => {
                info!("Service received shutdown signal");
                if let Some(handle) = monitor_handle.take() {
                    let _ = handle.await;
                }
                break;
            }
        }
    }

    Ok(())
}
