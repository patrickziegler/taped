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
    use lofty::config::WriteOptions;
    use lofty::file::{AudioFile, TaggedFileExt};
    use lofty::picture::{Picture, PictureType};
    use lofty::probe::Probe;
    use lofty::tag::{Accessor, ItemKey, Tag};
    use std::io::Cursor;

    let mut tagged_file = Probe::open(path)
        .map_err(|e| anyhow::anyhow!("Failed to probe file: {e}"))?
        .read()
        .map_err(|e| anyhow::anyhow!("Failed to read tags: {e}"))?;

    let tag = match tagged_file.primary_tag_mut() {
        Some(tag) => tag,
        None => {
            let tag_type = tagged_file.primary_tag_type();
            tagged_file.insert_tag(Tag::new(tag_type));
            tagged_file
                .primary_tag_mut()
                .ok_or_else(|| anyhow::anyhow!("Failed to create tag"))?
        }
    };

    tag.set_title(
        track
            .title
            .clone()
            .unwrap_or_else(|| "Unknown Title".to_string()),
    );
    tag.set_artist(
        track
            .artist
            .clone()
            .unwrap_or_else(|| "Unknown Artist".to_string()),
    );
    tag.set_album(
        track
            .album
            .clone()
            .unwrap_or_else(|| "Unknown Album".to_string()),
    );

    if let Some(artist) = &track.album_artist {
        tag.insert_text(ItemKey::AlbumArtist, artist.to_string());
    } else if let Some(artist) = &track.artist {
        tag.insert_text(ItemKey::AlbumArtist, artist.to_string());
    }

    if let Some(n) = track.track_number {
        tag.set_track(n as u32);
    }
    if let Some(n) = track.disc_number {
        tag.set_disk(n as u32);
    }

    if let Some(art_url) = &track.art_url {
        info!("Downloading album art from {}", art_url);
        match reqwest::blocking::get(art_url) {
            Ok(response) => {
                if let Ok(bytes) = response.bytes() {
                    match Picture::from_reader(&mut Cursor::new(bytes)) {
                        Ok(mut picture) => {
                            picture.set_pic_type(PictureType::CoverFront);
                            tag.push_picture(picture);
                        }
                        Err(e) => error!("Failed to parse downloaded album art: {e}"),
                    }
                }
            }
            Err(e) => error!("Failed to download album art: {e}"),
        }
    }

    tagged_file
        .save_to_path(path, WriteOptions::default())
        .map_err(|e| anyhow::anyhow!("Failed to save tags: {e}"))?;

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
