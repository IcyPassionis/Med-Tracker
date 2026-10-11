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
    pub tray_backend: Option<crate::tray::backend::TrayBackend>,
    pub tray_ready: bool,
    pub tray_generation: u64,
    pub tray_loss_generation: u64,
    pub startup_waiting_for_tray: bool,
    pub resource_startup_error: Option<String>,
    pub startup_registration_error: Option<String>,
    pub sidebar_visible: bool,
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
            tray_backend: None,
            tray_ready: false,
            tray_generation: 0,
            tray_loss_generation: 0,
            startup_waiting_for_tray: false,
            resource_startup_error: None,
            startup_registration_error: None,
            sidebar_visible: true,
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
            tray_backend: None,
            tray_ready: false,
            tray_generation: 0,
            tray_loss_generation: 0,
            startup_waiting_for_tray: false,
            resource_startup_error: None,
            startup_registration_error: None,
            sidebar_visible: true,
        }
    }
}
