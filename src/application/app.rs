use super::states::medicationtracker::MedicationTracker;
use super::states::settings::Settings;
use super::states::state::State;
use super::states::uistate::UIState;
use iced;
pub struct App {
    pub uistate: UIState,
    pub state: State,
    pub settings: Settings,
    pub system_theme: iced::Theme,
    pub medicationtracker: MedicationTracker,
    pub window_id: Option<iced::window::Id>,
    pub popup_window_id: Option<iced::window::Id>,
    pub tray_icon: Option<tray::TrayIcon>,
}
impl App {
    pub fn new() -> Self {
        App {
            state: State::new(),
            settings: Settings::new(),
            system_theme: iced::Theme::CatppuccinMocha,
            uistate: UIState::new(),
            medicationtracker: MedicationTracker::new(),
            window_id: None,
            popup_window_id: None,
            tray_icon: None,
        }
    }

    pub fn refresh_system_theme(&mut self) {
        self.system_theme = crate::ui::theme::system();
    }
}
impl Default for App {
    fn default() -> Self {
        App {
            state: State::new(),
            settings: Settings::new(),
            system_theme: iced::Theme::CatppuccinMocha,
            uistate: UIState::new(),
            medicationtracker: MedicationTracker::new(),
            window_id: None,
            popup_window_id: None,
            tray_icon: None,
        }
    }
}
