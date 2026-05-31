use clap::{Args, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum QualityProfile {
    Low,
    Normal,
    High,
    #[serde(rename = "very-high")]
    VeryHigh,
    Lossless,
}

impl std::fmt::Display for QualityProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QualityProfile::Low => write!(f, "low"),
            QualityProfile::Normal => write!(f, "normal"),
            QualityProfile::High => write!(f, "high"),
            QualityProfile::VeryHigh => write!(f, "very-high"),
            QualityProfile::Lossless => write!(f, "lossless"),
        }
    }
}

impl QualityProfile {
    pub fn format(&self) -> &'static str {
        match self {
            QualityProfile::Lossless => "flac",
            _ => "opus",
        }
    }

    pub fn bitrate(&self) -> Option<u32> {
        match self {
            QualityProfile::Low => Some(24),
            QualityProfile::Normal => Some(64),
            QualityProfile::High => Some(96),
            QualityProfile::VeryHigh => Some(128),
            QualityProfile::Lossless => None,
        }
    }

    pub fn sample_rate(&self) -> u32 {
        44100
    }

    pub fn channels(&self) -> u32 {
        2
    }
}

#[derive(Args, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AudioConfig {
    /// Audio quality level matching Spotify levels (low, normal, high, very-high, lossless)
    #[arg(short, long, value_enum, default_value_t = QualityProfile::VeryHigh)]
    pub quality: QualityProfile,
}

impl AudioConfig {
    pub fn format(&self) -> &'static str {
        self.quality.format()
    }

    pub fn bitrate(&self) -> Option<u32> {
        self.quality.bitrate()
    }

    pub fn sample_rate(&self) -> u32 {
        self.quality.sample_rate()
    }

    pub fn channels(&self) -> u32 {
        self.quality.channels()
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            quality: QualityProfile::VeryHigh,
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
        assert_eq!(config, deserialized);
        assert_eq!(config.quality, deserialized.quality);
        assert_eq!(config.format(), deserialized.format());
        assert_eq!(config.bitrate(), deserialized.bitrate());
        assert_eq!(config.sample_rate(), deserialized.sample_rate());
        assert_eq!(config.channels(), deserialized.channels());
    }
}
