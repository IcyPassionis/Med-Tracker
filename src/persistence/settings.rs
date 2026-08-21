use crate::application::states::settings::Settings;
use iced::Theme;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;

#[derive(Serialize, Deserialize)]
#[serde(default)]
struct PersistedSettings {
    font_size: u16,
    language: String,
    is_theme_changed: bool,
    theme: String,
    sound_enabled: bool,
    sound_volume: f32,
    alarm_sound_path: String,
    is_auto_startup: bool,
    is_minimize_to_tray: bool,
    is_24_hour_format: bool,
}

impl Default for PersistedSettings {
    fn default() -> Self {
        Self::from(&Settings::new())
    }
}

impl From<&Settings> for PersistedSettings {
    fn from(settings: &Settings) -> Self {
        Self {
            font_size: settings.font_size,
            language: settings.language.clone(),
            is_theme_changed: settings.is_theme_changed,
            theme: settings.theme.to_string(),
            sound_enabled: settings.sound_enabled,
            sound_volume: settings.sound_volume,
            alarm_sound_path: settings.alarm_sound_path.clone(),
            is_auto_startup: settings.is_auto_startup,
            is_minimize_to_tray: settings.is_minimize_to_tray,
            is_24_hour_format: settings.is_24_hour_format,
        }
    }
}

impl PersistedSettings {
    fn into_runtime(self) -> Settings {
        let defaults = Settings::new();
        let sound_volume = if self.sound_volume.is_finite() {
            self.sound_volume.clamp(0.0, 1.0)
        } else {
            defaults.sound_volume
        };

        Settings {
            font_size: self.font_size,
            language: self.language,
            is_theme_changed: self.is_theme_changed,
            theme: theme_from_name(&self.theme).unwrap_or(defaults.theme),
            sound_enabled: self.sound_enabled,
            sound_volume,
            alarm_sound_path: self.alarm_sound_path,
            is_auto_startup: self.is_auto_startup,
            is_minimize_to_tray: self.is_minimize_to_tray,
            is_24_hour_format: self.is_24_hour_format,
        }
    }
}

fn theme_from_name(name: &str) -> Option<Theme> {
    Theme::ALL
        .iter()
        .find(|theme| theme.to_string() == name)
        .cloned()
}

pub fn save_settings(settings: &Settings) -> Result<(), io::Error> {
    let dir = super::data_dir().ok_or(io::Error::new(
        io::ErrorKind::NotFound,
        "Could not find data directory",
    ))?;
    fs::create_dir_all(&dir)?;
    let json = serde_json::to_string_pretty(&PersistedSettings::from(settings))?;
    let tmp_path = dir.join("settings.json.tmp");
    fs::write(&tmp_path, json)?;
    fs::rename(tmp_path, dir.join("settings.json"))?;
    Ok(())
}

pub fn load_settings() -> Option<Settings> {
    let path = super::data_dir()?.join("settings.json");
    let contents = fs::read_to_string(&path).ok()?;
    match serde_json::from_str::<PersistedSettings>(&contents) {
        Ok(settings) => Some(settings.into_runtime()),
        Err(e) => {
            eprintln!("Failed to parse {}: {e}", path.display());
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_preserves_settings() {
        let settings = Settings {
            font_size: 18,
            language: "English".to_string(),
            is_theme_changed: true,
            theme: Theme::Nord,
            sound_enabled: false,
            sound_volume: 0.35,
            alarm_sound_path: "/tmp/custom-alarm.wav".to_string(),
            is_auto_startup: true,
            is_minimize_to_tray: true,
            is_24_hour_format: false,
        };

        let json = serde_json::to_string(&PersistedSettings::from(&settings)).unwrap();
        let restored = serde_json::from_str::<PersistedSettings>(&json)
            .unwrap()
            .into_runtime();

        assert_eq!(restored.font_size, settings.font_size);
        assert_eq!(restored.language, settings.language);
        assert_eq!(restored.is_theme_changed, settings.is_theme_changed);
        assert_eq!(restored.theme, settings.theme);
        assert_eq!(restored.sound_enabled, settings.sound_enabled);
        assert_eq!(restored.sound_volume, settings.sound_volume);
        assert_eq!(restored.alarm_sound_path, settings.alarm_sound_path);
        assert_eq!(restored.is_auto_startup, settings.is_auto_startup);
        assert_eq!(restored.is_minimize_to_tray, settings.is_minimize_to_tray);
        assert_eq!(restored.is_24_hour_format, settings.is_24_hour_format);
    }

    #[test]
    fn minimize_to_tray_defaults_to_enabled() {
        let restored = serde_json::from_str::<PersistedSettings>(r#"{}"#)
            .unwrap()
            .into_runtime();
        assert!(restored.is_minimize_to_tray);
        assert!(Settings::new().is_minimize_to_tray);
    }

    #[test]
    fn missing_values_use_defaults_and_unknown_theme_falls_back() {
        let restored =
            serde_json::from_str::<PersistedSettings>(r#"{"theme":"Unknown","sound_volume":2.0}"#)
                .unwrap()
                .into_runtime();

        assert_eq!(restored.theme, Theme::Dark);
        assert_eq!(restored.sound_volume, 1.0);
        assert_eq!(restored.language, "English");
        assert!(restored.sound_enabled);
    }
}
