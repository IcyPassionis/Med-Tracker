use iced::widget::container::Style;
use iced::{Background, Border, Color, Shadow, Theme, Vector};

pub fn settings_surface(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    Style {
        background: Some(Background::Color(palette.background.weak.color)),
        border: Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: 30.0.into(),
        },
        shadow: Shadow {
            color: Color::BLACK,
            offset: Vector { x: 0.0, y: 6.0 },
            blur_radius: 12.0,
        },
        ..Default::default()
    }
}

pub fn setting_row(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    Style {
        background: Some(Background::Color(palette.background.strong.color)),
        text_color: Some(palette.background.strong.text),
        border: Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: 20.0.into(),
        },
        ..Default::default()
    }
}
