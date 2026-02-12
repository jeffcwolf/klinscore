// settings.rs
// User preferences and application settings

use serde::{Deserialize, Serialize};
use std::fmt;

/// Application theme options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppTheme {
    Light,
    Dark,
    Sepia,
}

pub const ALL_THEMES: [AppTheme; 3] = [AppTheme::Light, AppTheme::Dark, AppTheme::Sepia];

impl AppTheme {
    pub fn all() -> &'static [AppTheme] {
        &ALL_THEMES
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            AppTheme::Light => "Light",
            AppTheme::Dark => "Dark",
            AppTheme::Sepia => "Sepia",
        }
    }

    pub fn to_german(&self) -> &'static str {
        match self {
            AppTheme::Light => "Hell",
            AppTheme::Dark => "Dunkel",
            AppTheme::Sepia => "Sepia",
        }
    }

    /// Get the Iced theme for this app theme
    pub fn to_iced_theme(&self) -> iced::Theme {
        match self {
            AppTheme::Light => iced::Theme::Light,
            AppTheme::Dark => iced::Theme::Dark,
            AppTheme::Sepia => iced::Theme::TokyoNight, // Closest to sepia
        }
    }

    /// Background color for the theme
    #[allow(dead_code)]
    pub fn background_color(&self) -> iced::Color {
        match self {
            AppTheme::Light => iced::Color::from_rgb(1.0, 1.0, 1.0),
            AppTheme::Dark => iced::Color::from_rgb(0.12, 0.12, 0.12),
            AppTheme::Sepia => iced::Color::from_rgb(0.96, 0.92, 0.85), // Warm sepia
        }
    }

    /// Text color for the theme
    #[allow(dead_code)]
    pub fn text_color(&self) -> iced::Color {
        match self {
            AppTheme::Light => iced::Color::from_rgb(0.0, 0.0, 0.0),
            AppTheme::Dark => iced::Color::from_rgb(0.9, 0.9, 0.9),
            AppTheme::Sepia => iced::Color::from_rgb(0.2, 0.15, 0.1), // Dark brown for sepia
        }
    }
}

impl Default for AppTheme {
    fn default() -> Self {
        AppTheme::Light
    }
}

impl fmt::Display for AppTheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub theme: AppTheme,
    pub show_help_hints: bool,
    pub auto_calculate: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: AppTheme::Light,
            show_help_hints: true,
            auto_calculate: false,
        }
    }
}

impl Settings {
    pub fn new() -> Self {
        Self::default()
    }
}
