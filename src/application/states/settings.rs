use iced as ice;
pub struct Settings {
    pub font_size: u16,
    pub language: String,
    pub is_theme_changed: bool,
    pub theme: ice::Theme,
    pub sound_enabled: bool,
    pub sound_volume: f32,
    pub alarm_sound_path: String,
    pub alarm_time_minutes: u32,
    pub is_auto_startup: bool,
    pub is_minimize_to_tray: bool,
    pub is_24_hour_format: bool,
}
impl Settings {
    pub fn new() -> Settings {
        Settings {
            font_size: 14,
            language: "English".to_string(),
            is_theme_changed: false,
            theme: ice::Theme::Dark,
            sound_enabled: true,
            sound_volume: 1.0,
            alarm_sound_path: "audio/alarm.wav".to_string(),
            alarm_time_minutes: 15,
            is_auto_startup: false,
            is_minimize_to_tray: true,
            is_24_hour_format: true,
        }
    }
}
impl Default for Settings {
    fn default() -> Self {
        Settings::new()
    }
}
