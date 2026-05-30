use id3::TagLike;
use std::collections::HashMap;
use std::path::Path;
use tracing::{error, info};
use zbus::zvariant::{Array, OwnedValue};

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

impl TrackInfo {
    pub fn is_ad(&self) -> bool {
        self.track_id.contains("/ad/")
            || self.track_id.starts_with("spotify:ad:")
            || self.track_id.is_empty()
            || self.artist.as_deref() == Some("")
            || self.album.as_deref() == Some("")
    }
}

pub fn parse_track_info(metadata: &HashMap<String, OwnedValue>) -> TrackInfo {
    let mut track = TrackInfo::default();

    if let Some(v) = metadata.get("mpris:trackid") {
        if let Ok(s) = v.downcast_ref::<&str>() {
            track.track_id = s.to_string();
        }
    }

    track.title = metadata
        .get("xesam:title")
        .and_then(|v| v.downcast_ref::<&str>().ok().map(|s| s.to_string()));
    track.album = metadata
        .get("xesam:album")
        .and_then(|v| v.downcast_ref::<&str>().ok().map(|s| s.to_string()));
    track.art_url = metadata
        .get("mpris:artUrl")
        .and_then(|v| v.downcast_ref::<&str>().ok().map(|s| s.to_string()));
    track.track_number = metadata
        .get("xesam:trackNumber")
        .and_then(|v| v.downcast_ref::<i32>().ok());
    track.disc_number = metadata
        .get("xesam:discNumber")
        .and_then(|v| v.downcast_ref::<i32>().ok());

    track.artist = metadata.get("xesam:artist").and_then(|v| {
        v.downcast_ref::<Array>().ok().map(|array| {
            array
                .iter()
                .filter_map(|val| {
                    let s: Result<&str, _> = val.try_into();
                    s.ok().map(|s| s.to_string())
                })
                .collect::<Vec<String>>()
                .join(", ")
        })
    });

    track.album_artist = metadata.get("xesam:albumArtist").and_then(|v| {
        v.downcast_ref::<Array>().ok().map(|array| {
            array
                .iter()
                .filter_map(|val| {
                    let s: Result<&str, _> = val.try_into();
                    s.ok().map(|s| s.to_string())
                })
                .collect::<Vec<String>>()
                .join(", ")
        })
    });

    track
}

pub fn apply_tags(path: &Path, track: &TrackInfo) -> anyhow::Result<()> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext != "mp3" && ext != "wav" {
        info!(
            "Skipping ID3 metadata tagging for .{} files (unsupported format)",
            ext
        );
        return Ok(());
    }
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
        info!("Downloading album art from {}", art_url);
        if let Ok(response) = reqwest::blocking::get(art_url) {
            if let Ok(bytes) = response.bytes() {
                tag.add_frame(id3::frame::Picture {
                    mime_type: "image/jpeg".to_string(),
                    picture_type: id3::frame::PictureType::CoverFront,
                    description: "Album Art".to_string(),
                    data: bytes.to_vec(),
                });
            }
        } else {
            error!("Failed to download album art");
        }
    }

    tag.write_to_path(path, id3::Version::Id3v24)?;
    Ok(())
}

pub fn format_path(pattern: &str, track: &TrackInfo) -> String {
    pattern
        .replace("{title}", track.title.as_deref().unwrap_or("Unknown Title"))
        .replace(
            "{artist}",
            track.artist.as_deref().unwrap_or("Unknown Artist"),
        )
        .replace("{album}", track.album.as_deref().unwrap_or("Unknown Album"))
        .replace(
            "{albumArtist}",
            track
                .album_artist
                .as_deref()
                .unwrap_or(track.artist.as_deref().unwrap_or("Unknown Artist")),
        )
        .replace(
            "{trackNumber}",
            &format!("{:02}", track.track_number.unwrap_or(0)),
        )
        .replace("{discNumber}", &track.disc_number.unwrap_or(1).to_string())
}
