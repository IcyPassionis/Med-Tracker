use iced::widget::{column, container, text};
use iced::{self as ice, Element, Length::Fill, Theme, window};

use crate::application::app::App;
use crate::application::message::Message;
use crate::ui::content::main_content;
use crate::ui::sidebar::{collapsed_sidebar, side_bar, sidebar_border};
use ice::widget::row;

pub fn title(_state: &App, _window_id: window::Id) -> String {
    String::from("Med-Tracker")
}

pub fn view(state: &App, _window_id: window::Id) -> Element<Message> {
    main_view(state)
}

fn main_view(state: &App) -> Element<Message> {
    let mut content = column![].width(Fill).height(Fill);
    if let Some(error) = &state.resource_startup_error {
        content = content.push(
            container(text(error.clone()).size(14))
                .padding(10)
                .width(Fill),
        );
    }
    if let Some(error) = &state.startup_registration_error {
        content = content.push(
            container(text(format!("Login startup could not be reconciled: {error}")).size(14))
                .padding(10)
                .width(Fill),
        );
    }
    let mut children = vec![];
    if state.sidebar_visible {
        children.push(side_bar(&state.state.panel));
        children.push(sidebar_border());
    } else {
        children.push(collapsed_sidebar());
        children.push(sidebar_border());
    }
    children.push(content.push(main_content(state)).into());
    row(children).width(Fill).height(Fill).into()
}

pub fn theme(state: &App, _window_id: window::Id) -> Option<Theme> {
    let theme = if state.settings.is_theme_changed {
        state.settings.theme.clone()
    } else {
        state.system_theme.clone()
    };
    Some(theme)
}
