use iced::widget::container::Style;
use iced::{Background, Border, Color, Shadow, Theme, Vector};

pub fn pill_icon_container(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    Style {
        background: Some(Background::Color(palette.secondary.strong.color)),
        border: Border {
            radius: 10.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn medication_card(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    Style {
        background: Some(Background::Color(palette.background.weak.color)),
        text_color: Some(palette.background.base.text),
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

pub fn backdrop(_theme: &Theme) -> Style {
    Style {
        background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.4))),
        ..Default::default()
    }
}

pub fn delete_dialog(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    Style {
        background: Some(Background::Color(palette.background.base.color)),
        border: Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: 25.0.into(),
        },
        shadow: Shadow {
            color: Color::BLACK,
            offset: Vector { x: 0.0, y: 6.0 },
            blur_radius: 12.0,
        },
        ..Default::default()
    }
}
