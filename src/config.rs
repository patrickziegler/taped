use clap::{Args, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioFormat {
    Mp3,
    Flac,
    Wav,
    M4a,
}

impl std::fmt::Display for AudioFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioFormat::Mp3 => write!(f, "mp3"),
            AudioFormat::Flac => write!(f, "flac"),
            AudioFormat::Wav => write!(f, "wav"),
            AudioFormat::M4a => write!(f, "m4a"),
        }
    }
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BitrateMode {
    Cbr,
    Vbr,
}

impl std::fmt::Display for BitrateMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BitrateMode::Cbr => write!(f, "cbr"),
            BitrateMode::Vbr => write!(f, "vbr"),
        }
    }
}

#[derive(Args, Clone, Debug, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Audio quality format (mp3, flac, wav, m4a)
    #[arg(long, value_enum, default_value_t = AudioFormat::Mp3)]
    pub format: AudioFormat,

    /// Bitrate mode for lossy compression formats (cbr, vbr)
    #[arg(long, value_enum, default_value_t = BitrateMode::Vbr)]
    pub bitrate_mode: BitrateMode,

    /// VBR quality level (0-9, 0 is best) for MP3/AAC VBR mode
    #[arg(long, default_value_t = 2)]
    pub vbr_quality: u32,

    /// CBR Bitrate in kbps (e.g. 128, 192, 256, 320)
    #[arg(long, default_value_t = 320)]
    pub bitrate: u32,

    /// Audio sample rate in Hz (e.g. 44100, 48000)
    #[arg(long, default_value_t = 48000)]
    pub sample_rate: u32,

    /// Number of audio channels (1 for mono, 2 for stereo)
    #[arg(long, default_value_t = 2)]
    pub channels: u32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            format: AudioFormat::Mp3,
            bitrate_mode: BitrateMode::Vbr,
            vbr_quality: 2,
            bitrate: 320,
            sample_rate: 48000,
            channels: 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_config_serialization() {
        let config = AudioConfig::default();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: AudioConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(config.format, deserialized.format);
        assert_eq!(config.bitrate_mode, deserialized.bitrate_mode);
        assert_eq!(config.vbr_quality, deserialized.vbr_quality);
        assert_eq!(config.bitrate, deserialized.bitrate);
        assert_eq!(config.sample_rate, deserialized.sample_rate);
        assert_eq!(config.channels, deserialized.channels);
    }
}
