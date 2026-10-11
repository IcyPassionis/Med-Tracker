use crate::ui::panel::{alarm, calendar, home, medications, settings};
use iced;
#[derive(Debug, Clone)]
pub enum Message {
    OpenTime,
    OpenManageMeds,
    OpenCalendar,
    OpenSettings,
    Settings(settings::Message),
    Time(home::time::Message),
    Medications(medications::medicationsmain::Message),
    Calendar(calendar::calendarui::Message),
    Alarm(alarm::Message),
    TimeCheck,
    Tray {
        backend: crate::tray::backend::TrayBackend,
        generation: u64,
        event: crate::tray::backend::TrayEvent,
    },
    ShowRequested,
    TrayStartupTimeout {
        generation: u64,
    },
    TrayLossTimeout {
        generation: u64,
        loss_generation: u64,
    },
    CloseRequested(iced::window::Id),
    WindowOpened(iced::window::Id),
    HideSidebar,
    OpenSidebar,
}
