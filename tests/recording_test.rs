mod mock;

use mock::{run_mock, MockCommand};
use spotify_recorder::watchdog::monitor_spotify;
use spotify_recorder::recorder::exporter_task;
use std::collections::HashMap;
use tokio::sync::mpsc;
use zbus::{connection, zvariant::Value};
use std::os::unix::fs::PermissionsExt;
use std::fs;

#[tokio::test]
async fn test_recording_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt::try_init();
    
    // 1. Create a mock pw-record, ffmpeg and pactl script
    let temp_dir = tempfile::tempdir()?;
    let mock_pw_record_path = temp_dir.path().join("pw-record");
    fs::write(&mock_pw_record_path, "#!/bin/sh\n# just a stub for pw-cat as it is called via sh -c\nexit 0")?;
    let mut perms = fs::metadata(&mock_pw_record_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&mock_pw_record_path, perms)?;

    let mock_ffmpeg_path = temp_dir.path().join("ffmpeg");
    fs::write(&mock_ffmpeg_path, "#!/bin/sh\n# parse args to find output file\nFILE=\"\"\nwhile [ $# -gt 0 ]; do\n  case $1 in\n    -y|-f|-ar|-ac|-i|-codec:a|-qscale:a) shift ; shift ;;\n    pipe:0) shift ;;\n    *) FILE=$1 ; shift ;;\n  esac\ndone\ntouch \"$FILE\"\n# stay alive until signaled\nwhile true; do sleep 1; done")?;
    // Simplified ffmpeg mock for this test as we just need to see the file created
    let mut perms = fs::metadata(&mock_ffmpeg_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&mock_ffmpeg_path, perms)?;

    let mock_pactl_path = temp_dir.path().join("pactl");
    fs::write(&mock_pactl_path, "#!/bin/sh\ncase $1 in\n  get-default-sink) echo \"default-sink\" ;;\n  list) echo \"1 default-sink.monitor module-null-sink.c s16le 2ch 44100Hz RUNNING\" ;;\nesac")?;
    let mut perms = fs::metadata(&mock_pactl_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&mock_pactl_path, perms)?;

    // 2. Add temp_dir to PATH
    let old_path = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", temp_dir.path().to_str().unwrap(), old_path);
    unsafe {
        std::env::set_var("PATH", new_path);
    }

    let spotify_bus_name = "org.mpris.MediaPlayer2.spotify.test_recording";
    let service_bus_name = "org.spotify_recorder.test_rec";
    let (mock_tx, mock_rx) = mpsc::channel(10);

    // Start mock Spotify
    tokio::spawn(async move {
        run_mock(mock_rx, spotify_bus_name).await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    let connection = connection::Builder::session()?
        .name(service_bus_name)?
        .build()
        .await?;
    
    let (exporter_tx, exporter_rx) = mpsc::channel(10);

    let music_dir = temp_dir.path().join("Music");
    fs::create_dir_all(&music_dir)?;
    let pattern = "{albumArtist} - {album}/{trackNumber} - {title}".to_string();

    // Start exporter
    tokio::spawn(exporter_task(exporter_rx, music_dir.clone(), pattern));

    // 1. Prepare Initial Playing Song 1 (should be skipped)
    let mut metadata1 = HashMap::new();
    metadata1.insert("mpris:trackid".to_string(), Value::from("track1").try_to_owned()?);
    metadata1.insert("xesam:title".to_string(), Value::from("Song 1").try_to_owned()?);
    mock_tx.send(MockCommand::Metadata(metadata1)).await?;
    mock_tx.send(MockCommand::PlaybackStatus("Playing".to_string())).await?;

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Start monitor
    tokio::spawn(monitor_spotify(
        connection.clone(),
        spotify_bus_name.to_string(),
        exporter_tx,
    ));

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // 2. Change track to Song 2 (This triggers the first transition, starting recording for Song 2)
    let mut metadata2 = HashMap::new();
    metadata2.insert("mpris:trackid".to_string(), Value::from("track2").try_to_owned()?);
    metadata2.insert("xesam:title".to_string(), Value::from("Song 2").try_to_owned()?);
    mock_tx.send(MockCommand::Metadata(metadata2)).await?;

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    
    // Song 1 should NOT be exported
    let song1_path = music_dir.join("Unknown Artist - Unknown Album").join("00 - Song 1.mp3");
    assert!(!song1_path.exists(), "Song 1 should NOT be exported as it was the first track");

    // 3. Change track to Song 3 (This STARTS recording Song 3)
    let mut metadata3 = HashMap::new();
    metadata3.insert("mpris:trackid".to_string(), Value::from("track3").try_to_owned()?);
    metadata3.insert("xesam:title".to_string(), Value::from("Song 3").try_to_owned()?);
    mock_tx.send(MockCommand::Metadata(metadata3)).await?;

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // 4. Change track to Song 4 (This trigger Song 3 export)
    let mut metadata4 = HashMap::new();
    metadata4.insert("mpris:trackid".to_string(), Value::from("track4").try_to_owned()?);
    metadata4.insert("xesam:title".to_string(), Value::from("Song 4").try_to_owned()?);
    mock_tx.send(MockCommand::Metadata(metadata4)).await?;

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // 5. Song 3 should be exported
    let song3_path = music_dir.join("Unknown Artist - Unknown Album").join("00 - Song 3.mp3");
    
    // Wait a bit more for exporter
    for _ in 0..10 {
        if song3_path.exists() { break; }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    assert!(song3_path.exists(), "Exported Song 3 should exist at {:?}", song3_path);

    Ok(())
}
