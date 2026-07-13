use crate::application::app::App;
use crate::application::message::Message;
use crate::application::panel::Panel;
use iced::widget::{column, container};
use iced::{Element, Fill};

pub fn main_content<'a>(state: &'a App) -> Element<'a, Message> {
    container(
        column![match &state.state.panel {
            Panel::Time => state
                .uistate
                .timeui
                .view(&state.medicationtracker)
                .map(Message::Time),
            Panel::Calendar => state
                .uistate
                .calendarui
                .view(&state.medicationtracker)
                .map(Message::Calendar),
            Panel::ManageMeds => state
                .uistate
                .medicationsui
                .view(&state.medicationtracker)
                .map(Message::Medications),
            Panel::Settings => state.uistate.settingsui.view().map(Message::Settings),
            Panel::Alarm => state
                .uistate
                .alarmui
                .view(&state.medicationtracker)
                .map(Message::Alarm),
        }]
        .padding(5),
    )
    .width(Fill)
    .height(Fill)
    .into()
}
