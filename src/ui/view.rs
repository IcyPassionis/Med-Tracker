use iced::widget::{button, column, container, text};
use iced::{self as ice, window, Element, Length::Fill, Theme};

use crate::application::app::App;
use crate::application::message::Message;
use crate::ui::content::main_content;
use crate::ui::sidebar::{side_bar, sidebar_border};
use ice::widget::row;

pub fn title(_state: &App, _window_id: window::Id) -> String {
    String::from("Med-Tracker")
}

pub fn view(state: &App, window_id: window::Id) -> Element<Message> {
    if Some(window_id) == state.popup_window_id {
        tray_popup_view()
    } else {
        main_view(state)
    }
}

fn main_view(state: &App) -> Element<Message> {
    row![
        side_bar(&state.state.panel),
        sidebar_border(),
        main_content(state),
    ]
    .width(Fill)
    .height(Fill)
    .into()
}

fn tray_popup_view() -> Element<'static, Message> {
    container(
        column![
            button(text("Show Application").center())
                .width(Fill)
                .on_press(Message::TrayMenuShow),
            button(text("Exit").center())
                .width(Fill)
                .on_press(Message::Quit),
        ]
        .spacing(4),
    )
    .width(Fill)
    .height(Fill)
    .padding(8)
    .into()
}

pub fn theme(state: &App, _window_id: window::Id) -> Option<Theme> {
    let theme = if state.settings.is_theme_changed {
        state.settings.theme.clone()
    } else {
        state.system_theme.clone()
    };
    Some(theme)
}
