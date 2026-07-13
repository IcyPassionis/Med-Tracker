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
    TrayLeftClick,
    TrayRightClick { x: f64, y: f64 },
    TrayMenuShow,
    CloseRequested(iced::window::Id),
    WindowOpened(iced::window::Id),
    Quit,
    HideSidebar,
    OpenSidebar,
}
