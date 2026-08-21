use ice::widget::container::Style;
use ice::{Background, Color, Theme};
use iced::{self as ice, Border, Shadow, Vector};

pub fn overlay_panel_container(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    Style {
        background: Some(Background::Color(palette.background.weak.color)),
        border: Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: ice::border::Radius {
                top_left: 45.0,
                top_right: 45.0,
                bottom_left: 45.0,
                bottom_right: 45.0,
            },
            ..Default::default()
        },
        shadow: Shadow {
            color: Color::BLACK,
            offset: Vector { x: 0.0, y: 6.0 },
            blur_radius: 12.0,
        },
        ..Default::default()
    }
}

pub fn record_status_container(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    Style {
        background: Some(Background::Color(palette.secondary.strong.color)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: ice::border::Radius {
                top_left: 10.0,
                top_right: 10.0,
                bottom_left: 10.0,
                bottom_right: 10.0,
            },
            ..Default::default()
        },
        ..Default::default()
    }
}
pub fn schedule_container(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    Style {
        background: Some(Background::Color(palette.background.weak.color)),
        border: Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: ice::border::Radius {
                top_left: 45.0,
                top_right: 45.0,
                bottom_left: 45.0,
                bottom_right: 45.0,
            },
            ..Default::default()
        },
        shadow: Shadow {
            color: Color::BLACK,
            offset: Vector { x: 0.01, y: 4.0 },
            blur_radius: 4.0,
        },
        ..Default::default()
    }
}
