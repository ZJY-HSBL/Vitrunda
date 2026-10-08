use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub hot_corner_size: i32,
    pub animation_ms: u64,
    pub glass_alpha: u8,
    pub autostart: bool,
    pub close_on_escape: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hot_corner_size: 14,
            animation_ms: 420,
            glass_alpha: 214,
            autostart: false,
            close_on_escape: true,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = Self::path();
        let Ok(text) = fs::read_to_string(path) else {
            return Self::default();
        };
        serde_json::from_str(&text).unwrap_or_default()
    }

    pub fn save(&self) -> io::Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(path, text)
    }

    pub fn path() -> PathBuf {
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            return PathBuf::from(local).join("Vitrunda").join("config.json");
        }
        PathBuf::from("config.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_safe() {
        let cfg = Config::default();
        assert!(cfg.hot_corner_size >= 8);
        assert!((180..=900).contains(&cfg.animation_ms));
        assert!(cfg.glass_alpha >= 128);
    }
}
