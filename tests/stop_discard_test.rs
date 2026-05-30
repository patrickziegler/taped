mod mock;

use mock::{MockCommand, run_mock};
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use taped::recorder::exporter_task;
use taped::watchdog::monitor_spotify;
use tokio::sync::mpsc;
use zbus::{connection, zvariant::Value};

#[tokio::test]
async fn test_stop_discard_behavior() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt::try_init();

    let temp_dir = tempfile::tempdir()?;

    // Mock binaries
    let mock_pw_record_path = temp_dir.path().join("pw-record");
    fs::write(&mock_pw_record_path, "#!/bin/sh\nexit 0")?;
    fs::set_permissions(&mock_pw_record_path, fs::Permissions::from_mode(0o755))?;

    let mock_ffmpeg_path = temp_dir.path().join("ffmpeg");
    fs::write(
        &mock_ffmpeg_path,
        "#!/bin/sh\nFILE=\"\"\nwhile [ $# -gt 0 ]; do\n  case $1 in\n    -y|-f|-ar|-ac|-i|-codec:a|-qscale:a|-b:a|-q:a) shift ; shift ;;\n    pipe:0) shift ;;\n    *) FILE=$1 ; shift ;;\n  esac\ndone\ntouch \"$FILE\"\nwhile true; do sleep 1; done",
    )?;
    fs::set_permissions(&mock_ffmpeg_path, fs::Permissions::from_mode(0o755))?;

    let mock_pactl_path = temp_dir.path().join("pactl");
    fs::write(
        &mock_pactl_path,
        "#!/bin/sh\ncase $1 in\n  get-default-sink) echo \"default-sink\" ;;\n  list) echo \"1 default-sink.monitor module-null-sink.c s16le 2ch 44100Hz RUNNING\" ;;\nesac",
    )?;
    fs::set_permissions(&mock_pactl_path, fs::Permissions::from_mode(0o755))?;

    let old_path = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", temp_dir.path().to_str().unwrap(), old_path);
    unsafe {
        std::env::set_var("PATH", new_path);
    }

    let spotify_bus_name = "org.mpris.MediaPlayer2.spotify.test_discard";
    let (mock_tx, mock_rx) = mpsc::channel(10);

    tokio::spawn(async move {
        run_mock(mock_rx, spotify_bus_name).await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    let connection = connection::Builder::session()?.build().await?;
    let (exporter_tx, exporter_rx) = mpsc::channel(10);
    let music_dir = temp_dir.path().join("Music");
    fs::create_dir_all(&music_dir)?;

    tokio::spawn(exporter_task(
        exporter_rx,
        music_dir.clone(),
        "{title}".to_string(),
        taped::config::AudioConfig::default(),
    ));

    let (_shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel(1);
    tokio::spawn(monitor_spotify(
        connection.clone(),
        spotify_bus_name.to_string(),
        exporter_tx,
        taped::config::AudioConfig::default(),
        shutdown_rx,
    ));

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // 1. Initial track
    let mut metadata1 = HashMap::new();
    metadata1.insert(
        "mpris:trackid".to_string(),
        Value::from("track1").try_to_owned()?,
    );
    metadata1.insert(
        "xesam:title".to_string(),
        Value::from("Initial").try_to_owned()?,
    );
    mock_tx.send(MockCommand::Metadata(metadata1)).await?;
    mock_tx
        .send(MockCommand::PlaybackStatus("Playing".to_string()))
        .await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // 2. Transition to Song 2 (starts recording)
    let mut metadata2 = HashMap::new();
    metadata2.insert(
        "mpris:trackid".to_string(),
        Value::from("track2").try_to_owned()?,
    );
    metadata2.insert(
        "xesam:title".to_string(),
        Value::from("Song 2").try_to_owned()?,
    );
    mock_tx.send(MockCommand::Metadata(metadata2)).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // 3. Pause playback (should DISCARD Song 2)
    mock_tx
        .send(MockCommand::PlaybackStatus("Paused".to_string()))
        .await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // 4. Check if Song 2 was exported
    let song2_path = music_dir.join("Song 2.mp3");
    assert!(
        !song2_path.exists(),
        "Song 2 should NOT have been exported after pause"
    );

    Ok(())
}
