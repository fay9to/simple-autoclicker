//! Settings are stored in `%LOCALAPPDATA%\SimpleAutoClicker\settings.ini`.
//! The format and keys match the previous C# build, so existing files keep
//! working and deleting the file resets everything.

use std::fs;
use std::path::PathBuf;

pub const VK_F6: u32 = 0x75;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AppTheme {
    System,
    Light,
    Dark,
}

impl AppTheme {
    pub fn as_str(self) -> &'static str {
        match self {
            AppTheme::System => "system",
            AppTheme::Light => "light",
            AppTheme::Dark => "dark",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "system" => Some(AppTheme::System),
            "light" => Some(AppTheme::Light),
            "dark" => Some(AppTheme::Dark),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub theme: AppTheme,
    /// Virtual-key code for Start/Stop, F6 by default.
    pub hotkey: u32,
    pub playback_hotkey: u32,
    pub record_hotkey: u32,
    pub minimize_when_playing: bool,
    pub minimize_when_recording: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: AppTheme::System,
            hotkey: VK_F6,
            playback_hotkey: 0,
            record_hotkey: 0,
            minimize_when_playing: false,
            minimize_when_recording: false,
        }
    }
}

fn folder() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|base| PathBuf::from(base).join("SimpleAutoClicker"))
}

fn file_path() -> Option<PathBuf> {
    folder().map(|dir| dir.join("settings.ini"))
}

impl Settings {
    pub fn load() -> Self {
        let mut settings = Settings::default();
        let Some(path) = file_path() else {
            return settings;
        };
        let Ok(contents) = fs::read_to_string(path) else {
            return settings;
        };

        for line in contents.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let key = key.trim();
            let value = value.trim();

            match key {
                "theme" => {
                    if let Some(theme) = AppTheme::parse(value) {
                        settings.theme = theme;
                    }
                }
                "hotkey" => {
                    if let Ok(vk) = value.parse::<u32>() {
                        settings.hotkey = vk;
                    }
                }
                "playback_hotkey" => {
                    if let Ok(vk) = value.parse::<u32>() {
                        settings.playback_hotkey = vk;
                    }
                }
                "record_hotkey" => {
                    if let Ok(vk) = value.parse::<u32>() {
                        settings.record_hotkey = vk;
                    }
                }
                "minimize_when_playing" => settings.minimize_when_playing = value == "1",
                "minimize_when_recording" => settings.minimize_when_recording = value == "1",
                _ => {}
            }
        }

        settings
    }

    pub fn save(&self) {
        let Some(dir) = folder() else {
            return;
        };
        if fs::create_dir_all(&dir).is_err() {
            return;
        }

        let text = format!(
            "theme={}\nhotkey={}\nplayback_hotkey={}\nrecord_hotkey={}\nminimize_when_playing={}\nminimize_when_recording={}\n",
            self.theme.as_str(),
            self.hotkey,
            self.playback_hotkey,
            self.record_hotkey,
            if self.minimize_when_playing { 1 } else { 0 },
            if self.minimize_when_recording { 1 } else { 0 },
        );

        let _ = fs::write(dir.join("settings.ini"), text);
    }
}
