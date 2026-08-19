use std::path::Path;

use iced::widget::{
    Image, button, column, container, pick_list, row, scrollable, slider, text, toggler,
};
use iced::{ContentFit, Element, Fill, Length, Theme, alignment};

use crate::application::states::settings::Settings;
use crate::ui::style::button::bordered_button;
use crate::ui::style::settings::button::category_button;
use crate::ui::style::settings::container::{setting_row as setting_row_style, settings_surface};

#[derive(Debug, Clone)]
pub struct Settingsui {
    section: Section,
}

impl Settingsui {
    pub fn new() -> Settingsui {
        Self {
            section: Section::Main,
        }
    }

    pub fn set_section_to_main(&mut self) {
        self.section = Section::Main;
    }

    pub fn view<'a>(&self, settings: &'a Settings) -> Element<'a, Message> {
        match self.section {
            Section::Main => self.main_view(settings),
            Section::Language => self.language_view(settings),
            Section::Theme => self.theme_view(settings),
            Section::Sound => self.sound_view(settings),
            Section::System => self.system_view(settings),
        }
    }

    fn main_view<'a>(&self, settings: &'a Settings) -> Element<'a, Message> {
        let category_grid = column![
            row![
                category_card(
                    "Language",
                    settings.language.clone(),
                    "icons/language.png",
                    Section::Language,
                ),
                category_card(
                    "Theme",
                    theme_summary(settings),
                    "icons/paint.png",
                    Section::Theme,
                ),
            ]
            .spacing(20)
            .width(Fill),
            row![
                category_card(
                    "Sound",
                    sound_summary(settings),
                    sound_icon(settings),
                    Section::Sound,
                ),
                category_card(
                    "System",
                    system_summary(settings),
                    "icons/settings.png",
                    Section::System,
                ),
            ]
            .spacing(20)
            .width(Fill),
        ]
        .spacing(20)
        .width(Fill);

        centered_scrollable(
            column![text("Options").size(30), category_grid]
                .spacing(24)
                .width(Fill)
                .align_x(alignment::Horizontal::Center),
        )
    }

    fn language_view<'a>(&self, settings: &'a Settings) -> Element<'a, Message> {
        let language_selector = pick_list(
            vec!["English".to_string()],
            Some(settings.language.clone()),
            Message::SelectLanguage,
        )
        .width(Length::Fixed(220.0));

        self.detail_view(
            "Language",
            "icons/language.png",
            column![setting_row(
                "Language",
                "Choose the application language.".to_string(),
                language_selector.into(),
            )]
            .spacing(16),
        )
    }

    fn theme_view<'a>(&self, settings: &'a Settings) -> Element<'a, Message> {
        let follow_system = toggler(!settings.is_theme_changed)
            .on_toggle(Message::FollowSystemTheme)
            .size(24);
        let theme_selector = pick_list(
            Theme::ALL,
            Some(settings.theme.clone()),
            Message::SelectTheme,
        )
        .width(Length::Fixed(260.0));
        let gtk_button = button("Apply GTK Theme")
            .style(bordered_button)
            .padding([10, 18]);

        self.detail_view(
            "Theme",
            "icons/paint.png",
            column![
                setting_row(
                    "Follow system theme",
                    "Use the operating system's light or dark preference.".to_string(),
                    follow_system.into(),
                ),
                setting_row(
                    "Iced theme",
                    "Choose one of Iced's built-in themes.".to_string(),
                    theme_selector.into(),
                ),
                setting_row(
                    "GTK theme",
                    "GTK theme support will be added later.".to_string(),
                    gtk_button.into(),
                ),
            ]
            .spacing(16),
        )
    }

    fn sound_view<'a>(&self, settings: &'a Settings) -> Element<'a, Message> {
        let volume = row![
            slider(0.0..=1.0, settings.sound_volume, Message::SetVolume)
                .step(0.01_f32)
                .width(Fill),
            text(format!("{:.0}%", settings.sound_volume * 100.0)).width(Length::Fixed(52.0)),
        ]
        .spacing(16)
        .align_y(alignment::Vertical::Center)
        .width(Fill);
        let sound_enabled = toggler(settings.sound_enabled)
            .on_toggle(Message::ToggleSound)
            .size(24);
        let sound_path = Path::new(&settings.alarm_sound_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Default alarm sound")
            .to_string();
        let choose_sound = button("Choose Sound File")
            .style(bordered_button)
            .padding([10, 18])
            .on_press(Message::ChooseSoundFile);

        self.detail_view(
            "Sound",
            sound_icon(settings),
            column![
                setting_row(
                    "Alarm sound",
                    "Enable or disable alarm playback.".to_string(),
                    sound_enabled.into(),
                ),
                setting_row(
                    "Volume",
                    "Adjust the alarm volume.".to_string(),
                    volume.into(),
                ),
                setting_row("Current alarm sound", sound_path, choose_sound.into(),),
            ]
            .spacing(16),
        )
    }

    fn system_view<'a>(&self, settings: &'a Settings) -> Element<'a, Message> {
        let minimize_to_tray = toggler(settings.is_minimize_to_tray)
            .on_toggle(Message::ToggleMinimizeToTray)
            .size(24);
        let auto_startup = toggler(settings.is_auto_startup)
            .on_toggle(Message::ToggleAutoStartup)
            .size(24);

        self.detail_view(
            "System",
            "icons/settings.png",
            column![
                setting_row(
                    "Minimize to tray icon",
                    "Keep the application available from the system tray.".to_string(),
                    minimize_to_tray.into(),
                ),
                setting_row(
                    "Start with system",
                    "Launch Med-Tracker when the system starts.".to_string(),
                    auto_startup.into(),
                ),
            ]
            .spacing(16),
        )
    }

    fn detail_view<'a>(
        &self,
        title: &'a str,
        icon_path: &'a str,
        settings: impl Into<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        let settings = settings.into();
        let header = row![
            container(
                button(
                    Image::new("icons/arrow-back-up.png")
                        .content_fit(ContentFit::Contain)
                        .width(24)
                        .height(24),
                )
                    .style(bordered_button)
                    .padding(10)
                    .on_press(Message::BackToMain),
            )
            .width(Length::FillPortion(1)),
            container(
                row![
                    Image::new(icon_path)
                        .content_fit(ContentFit::Contain)
                        .width(30)
                        .height(30),
                    text(title).size(28),
                ]
                .spacing(10)
                .align_y(alignment::Vertical::Center),
            )
            .width(Length::FillPortion(2))
            .center_x(Fill),
            container("").width(Length::FillPortion(1)),
        ]
        .width(Fill)
        .align_y(alignment::Vertical::Center);

        centered_scrollable(
            container(column![header, settings].spacing(24))
                .style(settings_surface)
                .padding(24)
                .width(Fill),
        )
    }

    pub fn update(&mut self, settings: &mut Settings, message: Message) {
        match message {
            Message::OpenSection(section) => self.section = section,
            Message::BackToMain => self.section = Section::Main,
            Message::SelectLanguage(language) => settings.language = language,
            Message::SelectTheme(theme) => {
                settings.theme = theme;
                settings.is_theme_changed = true;
            }
            Message::FollowSystemTheme(follow_system) => {
                settings.is_theme_changed = !follow_system;
            }
            Message::ToggleSound(enabled) => settings.sound_enabled = enabled,
            Message::SetVolume(volume) => settings.sound_volume = volume.clamp(0.0, 1.0),
            // The native picker will be connected here when file integration is added.
            Message::ChooseSoundFile => {}
            Message::ToggleMinimizeToTray(enabled) => settings.is_minimize_to_tray = enabled,
            Message::ToggleAutoStartup(enabled) => settings.is_auto_startup = enabled,
        }
    }
}

fn setting_row<'a>(
    label: &'a str,
    description: String,
    control: Element<'a, Message>,
) -> Element<'a, Message> {
    container(
        row![
            column![text(label).size(16), text(description).size(13)]
                .spacing(4)
                .width(Fill),
            control,
        ]
        .spacing(20)
        .align_y(alignment::Vertical::Center),
    )
    .style(setting_row_style)
    .padding([16, 20])
    .width(Fill)
    .into()
}

fn centered_scrollable<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    let content = container(content)
        .width(Fill)
        .max_width(760)
        .padding([24, 0]);

    scrollable(
        container(content).width(Fill).center_x(Fill),
    )
    .width(Fill)
    .height(Fill)
    .into()
}

fn category_card<'a>(
    title: &'a str,
    summary: String,
    icon_path: &'a str,
    section: Section,
) -> Element<'a, Message> {
    let content = row![
        Image::new(icon_path)
            .content_fit(ContentFit::Contain)
            .width(36)
            .height(36),
        column![text(title).size(20), text(summary).size(14)].spacing(6),
    ]
    .spacing(16)
    .align_y(alignment::Vertical::Center);

    button(container(content).width(Fill).center_y(Fill))
        .on_press(Message::OpenSection(section))
        .style(category_button)
        .width(Fill)
        .height(130)
        .into()
}

fn theme_summary(settings: &Settings) -> String {
    if settings.is_theme_changed {
        settings.theme.to_string()
    } else {
        "System default".to_string()
    }
}

fn sound_summary(settings: &Settings) -> String {
    if settings.sound_enabled {
        format!("Enabled - {:.0}%", settings.sound_volume * 100.0)
    } else {
        "Disabled".to_string()
    }
}

fn sound_icon(settings: &Settings) -> &'static str {
    if settings.sound_enabled {
        "icons/soundon.png"
    } else {
        "icons/soundoff.png"
    }
}

fn system_summary(settings: &Settings) -> String {
    let tray = if settings.is_minimize_to_tray {
        "Tray on"
    } else {
        "Tray off"
    };
    let startup = if settings.is_auto_startup {
        "startup on"
    } else {
        "startup off"
    };
    format!("{tray} - {startup}")
}

#[derive(Debug, Clone, PartialEq)]
pub enum Section {
    Main,
    Language,
    Theme,
    Sound,
    System,
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenSection(Section),
    BackToMain,
    SelectLanguage(String),
    SelectTheme(Theme),
    FollowSystemTheme(bool),
    ToggleSound(bool),
    SetVolume(f32),
    ChooseSoundFile,
    ToggleMinimizeToTray(bool),
    ToggleAutoStartup(bool),
}
