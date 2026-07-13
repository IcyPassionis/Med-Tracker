use crate::ui::panel::{alarm, calendar, home, medications, settings};
pub struct UIState {
    pub settingsui: settings::Settingsui,
    pub timeui: home::time::TimeUI,
    pub medicationsui: medications::medicationsmain::Record,
    pub calendarui: calendar::calendarui::CalendarUI,
    pub alarmui: alarm::AlarmUI,
}
impl UIState {
    pub fn new() -> Self {
        UIState {
            settingsui: settings::Settingsui::new(),
            timeui: home::time::TimeUI::new(),
            medicationsui: medications::medicationsmain::Record::new(),
            calendarui: calendar::calendarui::CalendarUI::new(),
            alarmui: alarm::AlarmUI::new(),
        }
    }
}
