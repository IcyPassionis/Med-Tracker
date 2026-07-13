use iced::widget::button::{Status, Style};
use iced::{Background, Border, Color, Shadow, Theme, Vector};

pub fn button(percentage: u8) -> impl Fn(&Theme, Status) -> Style {
    move |theme, status| {
        let palette = theme.extended_palette();
        let progress = percentage as f32 / 100.0;
        let base = palette.background.weak.color;
        let blue = palette.primary.base.color;
        let background = Color::from_rgb(
            base.r + (blue.r - base.r) * progress,
            base.g + (blue.g - base.g) * progress,
            base.b + (blue.b - base.b) * progress,
        );
        let background = match status {
            Status::Hovered => palette.primary.weak.color,
            Status::Pressed => palette.primary.strong.color,
            _ => background,
        };
        Style {
            background: Some(Background::Color(background)),
            text_color: if palette.is_dark {
                Color::WHITE
            } else {
                Color::from_rgb8(20, 20, 20)
            },
            border: Border {
                color: Color::BLACK,
                width: 1.0,
                radius: 12.0.into(),
            },
            shadow: Shadow {
                color: Color::BLACK,
                offset: Vector { x: 0.0, y: 3.0 },
                blur_radius: 4.0,
            },
            ..Default::default()
        }
    }
}

pub fn navigation_button(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    Style {
        background: Some(Background::Color(match status {
            Status::Hovered => palette.secondary.base.color,
            Status::Pressed => palette.secondary.weak.color,
            _ => palette.secondary.strong.color,
        })),
        text_color: palette.background.base.text,
        border: Border {
            color: Color::BLACK,
            width: 1.0,
            radius: 14.0.into(),
        },
        ..Default::default()
    }
}
