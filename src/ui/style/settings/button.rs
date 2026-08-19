use iced::widget::button::{Status, Style};
use iced::{Background, Border, Color, Shadow, Theme, Vector};

pub fn category_button(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let (background, text_color) = match status {
        Status::Active => (palette.background.weak.color, palette.background.base.text),
        Status::Hovered => (palette.primary.strong.color, palette.primary.strong.text),
        Status::Pressed => (palette.primary.base.color, palette.primary.base.text),
        Status::Disabled => (
            palette.background.strong.color,
            palette.background.strong.text,
        ),
    };

    Style {
        background: Some(Background::Color(background)),
        text_color,
        border: Border {
            color: Color::BLACK,
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
