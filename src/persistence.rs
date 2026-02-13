// persistence.rs
// Save and load application state (settings, history) to disk

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::settings::{AppTheme, Settings};
use crate::ui::Language;

const APP_NAME: &str = "klinscore";
const SETTINGS_FILE: &str = "settings.json";
const HISTORY_FILE: &str = "history.json";

/// Persistable settings (subset of Settings that should survive restarts)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedSettings {
    pub theme: AppTheme,
    pub language: Language,
    pub show_help_hints: bool,
    pub auto_calculate: bool,
}

impl From<(&Settings, Language)> for PersistedSettings {
    fn from((settings, language): (&Settings, Language)) -> Self {
        Self {
            theme: settings.theme,
            language,
            show_help_hints: settings.show_help_hints,
            auto_calculate: settings.auto_calculate,
        }
    }
}

/// Get the application data directory, creating it if needed
fn data_dir() -> Option<PathBuf> {
    let proj_dirs = directories::ProjectDirs::from("org", "klinscore", APP_NAME)?;
    let dir = proj_dirs.data_dir().to_path_buf();
    fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

/// Save settings to disk
pub fn save_settings(settings: &Settings, language: Language) {
    let Some(dir) = data_dir() else { return };
    let persisted = PersistedSettings::from((settings, language));
    let path = dir.join(SETTINGS_FILE);
    if let Ok(json) = serde_json::to_string_pretty(&persisted) {
        let _ = fs::write(path, json);
    }
}

/// Load settings from disk
pub fn load_settings() -> Option<PersistedSettings> {
    let dir = data_dir()?;
    let path = dir.join(SETTINGS_FILE);
    let data = fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

/// Save history to disk
pub fn save_history<T: Serialize>(history: &[T]) {
    let Some(dir) = data_dir() else { return };
    let path = dir.join(HISTORY_FILE);
    if let Ok(json) = serde_json::to_string_pretty(history) {
        let _ = fs::write(path, json);
    }
}

/// Load history from disk
pub fn load_history<T: for<'de> Deserialize<'de>>() -> Vec<T> {
    let Some(dir) = data_dir() else {
        return Vec::new();
    };
    let path = dir.join(HISTORY_FILE);
    let Ok(data) = fs::read_to_string(path) else {
        return Vec::new();
    };
    serde_json::from_str(&data).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persisted_settings_roundtrip() {
        let settings = Settings::new();
        let persisted = PersistedSettings::from((&settings, Language::German));
        let json = serde_json::to_string(&persisted).unwrap();
        let loaded: PersistedSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.language, Language::German);
        assert_eq!(loaded.theme, AppTheme::Light);
    }

    #[test]
    fn test_data_dir_creation() {
        // Should return Some on most systems
        let dir = data_dir();
        // In Docker container this should work
        if let Some(d) = dir {
            assert!(d.exists());
        }
    }
}
