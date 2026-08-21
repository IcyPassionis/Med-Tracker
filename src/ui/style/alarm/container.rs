use ice::widget::container::Style;
use ice::{Background, Color, Theme};
use iced::{self as ice, Border, Shadow, Vector};

pub fn alarm_panel_container(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    Style {
        background: Some(Background::Color(palette.background.weak.color)),
        border: Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: ice::border::Radius::from(45.0),
        },
        shadow: Shadow {
            color: Color::BLACK,
            offset: Vector { x: 0.0, y: 4.0 },
            blur_radius: 8.0,
        },
        ..Default::default()
    }
}

pub fn medication_item_container(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    Style {
        background: Some(Background::Color(palette.background.weak.color)),
        border: Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: ice::border::Radius::from(35.0),
        },
        ..Default::default()
    }
}
