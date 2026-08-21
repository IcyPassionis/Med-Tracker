use ice::widget::button::Style;
use iced::widget::button::Status;
use iced::{self as ice, Border};
use iced::{Background, Color, Theme};

use crate::ui::style::color::darken;

pub fn navbar_button(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let text_color = palette.background.base.text;
    match status {
        Status::Active => {
            navbar_button_background(darken(palette.background.base.color, 0.1), text_color)
        }
        Status::Disabled => {
            navbar_button_background(darken(palette.background.base.color, 0.1), text_color)
        }
        Status::Hovered => navbar_button_background(palette.background.weak.color, text_color),
        Status::Pressed => navbar_button_background(palette.background.strong.color, text_color),
    }
}
fn navbar_button_background(background_color: Color, text_color: Color) -> Style {
    Style {
        background: Some(Background::Color(background_color)),
        text_color,
        ..Default::default()
    }
}
pub fn close_button(theme: &Theme, status: Status) -> Style {
    bordered_button_size_radius(100.0, 1.0, theme, status)
}
pub fn bordered_button(theme: &Theme, status: Status) -> Style {
    bordered_button_size_radius(30.0, 2.0, theme, status)
}
fn bordered_button_size_radius(
    border_radius_size: f32,
    border_width_size: f32,
    theme: &Theme,
    status: Status,
) -> Style {
    let palette = theme.extended_palette();
    let text_color = palette.background.base.text;
    match status {
        Status::Active => bordered_button_color(
            border_radius_size,
            border_width_size,
            palette.background.weak.color,
            palette.background.weak.color,
            text_color,
        ),
        Status::Disabled => bordered_button_color(
            border_radius_size,
            border_width_size,
            palette.background.strong.color,
            palette.background.weak.color,
            text_color,
        ),
        Status::Hovered => bordered_button_color(
            border_radius_size,
            border_width_size,
            palette.primary.strong.color,
            palette.background.weak.color,
            text_color,
        ),
        Status::Pressed => bordered_button_color(
            border_radius_size,
            border_width_size,
            palette.primary.weak.color,
            palette.background.weak.color,
            text_color,
        ),
    }
}
fn bordered_button_color(
    border_radius_size: f32,
    border_width_size: f32,
    background_color: Color,
    border_color: Color,
    text_color: Color,
) -> Style {
    Style {
        background: Some(Background::Color(background_color)),
        border: Border {
            color: border_color,
            width: border_width_size,
            radius: iced::border::Radius {
                top_left: (border_radius_size),
                top_right: (border_radius_size),
                bottom_right: (border_radius_size),
                bottom_left: (border_radius_size),
            },
        },
        text_color,
        ..Default::default()
    }
}
