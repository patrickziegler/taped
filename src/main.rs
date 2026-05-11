use clap::Parser;
use spotify_recorder::{ServiceControl, run_service};
use std::path::PathBuf;
use zbus::connection;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Destination directory for recorded tracks
    destination: Option<PathBuf>,

    /// Start in background mode (recording disabled by default)
    #[arg(long)]
    background: bool,

    /// Pattern for constructing filenames
    #[arg(long, default_value = "{albumArtist} - {album}/{trackNumber} - {title}")]
    pattern: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let music_dir = args.destination.unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
        PathBuf::from(home).join("Music").join("Spotify")
    });

    let recording_enabled = !args.background;

    let control = ServiceControl::new(recording_enabled);
    let connection = connection::Builder::session()?
        .name("org.spotify_recorder")?
        .serve_at("/org/spotify_recorder/Control", control)?
        .build()
        .await?;

    run_service(
        connection,
        "org.mpris.MediaPlayer2.spotify",
        music_dir,
        args.pattern,
    )
    .await
}
