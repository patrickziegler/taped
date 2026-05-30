use clap::Parser;
use std::path::PathBuf;
use zbus::connection;

pub use taped::watchdog::run_service;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Destination directory for recorded tracks
    destination: Option<PathBuf>,

    /// Pattern for constructing filenames
    #[arg(
        long,
        default_value = "{albumArtist} - {album}/{trackNumber} - {title}"
    )]
    pattern: String,

    #[command(flatten)]
    audio: taped::config::AudioConfig,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let music_dir = args.destination.unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
        PathBuf::from(home).join("Music").join("Spotify")
    });

    let connection = connection::Builder::session()?.build().await?;

    let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel(1);

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        tracing::info!("Shutdown signal received (CTRL-C)");
        let _ = shutdown_tx.send(());
    });

    run_service(
        connection,
        "org.mpris.MediaPlayer2.spotify",
        music_dir,
        args.pattern,
        args.audio,
        shutdown_rx,
    )
    .await
}
