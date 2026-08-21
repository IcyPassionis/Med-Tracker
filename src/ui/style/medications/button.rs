use iced::widget::button::{Status, Style};
use iced::{Background, Border, Color, Shadow, Theme, Vector};

pub fn medication_card_button(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let (background, shadow_blur) = match status {
        Status::Active => (palette.background.weak.color, 12.0),
        Status::Hovered => (palette.background.strong.color, 16.0),
        Status::Pressed => (palette.background.strong.color, 6.0),
        Status::Disabled => (palette.background.weak.color, 12.0),
    };
    Style {
        background: Some(Background::Color(background)),
        text_color: palette.background.base.text,
        border: Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: 30.0.into(),
        },
        shadow: Shadow {
            color: Color::BLACK,
            offset: Vector { x: 0.0, y: 6.0 },
            blur_radius: shadow_blur,
        },
        ..Default::default()
    }
}
