use iced::widget::button::{Status, Style};
use iced::{Background, Border, Color, Shadow, Theme, Vector};

pub fn weekday_button(selected: bool) -> impl Fn(&Theme, Status) -> Style {
    move |theme: &Theme, status: Status| {
        let palette = theme.extended_palette();
        let text_color = if !palette.is_dark {
            Color::from_rgb8(20, 20, 20)
        } else {
            Color::from_rgb8(220, 220, 220)
        };
        let background_color = if selected {
            match status {
                Status::Active => palette.primary.strong.color,
                Status::Disabled => palette.primary.base.color,
                Status::Hovered => palette.primary.base.color,
                Status::Pressed => palette.primary.weak.color,
            }
        } else {
            match status {
                Status::Active => palette.secondary.strong.color,
                Status::Disabled => palette.secondary.base.color,
                Status::Hovered => palette.secondary.base.color,
                Status::Pressed => palette.secondary.weak.color,
            }
        };
        Style {
            background: Some(Background::Color(background_color)),
            border: Border {
                color: Color::BLACK,
                width: if selected { 2.0 } else { 1.0 },
                radius: iced::border::Radius {
                    top_left: 60.0,
                    top_right: 60.0,
                    bottom_right: 60.0,
                    bottom_left: 60.0,
                },
            },
            shadow: Shadow {
                color: Color::BLACK,
                offset: Vector { x: 0.01, y: 4.0 },
                blur_radius: 4.0,
            },
            text_color,
            ..Default::default()
        }
    }
}
pub fn calendar_button(selected: bool, is_today: bool) -> impl Fn(&Theme, Status) -> Style {
    move |theme: &Theme, status: Status| {
        let palette = theme.extended_palette();
        let text_color = if !palette.is_dark {
            Color::from_rgb8(20, 20, 20)
        } else {
            Color::from_rgb8(220, 220, 220)
        };
        let background_color = if selected {
            match status {
                Status::Active => palette.background.strong.color,
                Status::Disabled => palette.background.base.color,
                Status::Hovered => palette.background.base.color,
                Status::Pressed => palette.background.weak.color,
            }
        } else if is_today {
            match status {
                Status::Active => Color::from_rgb8(120, 160, 220),
                Status::Disabled => Color::from_rgb8(100, 140, 200),
                Status::Hovered => Color::from_rgb8(140, 180, 240),
                Status::Pressed => Color::from_rgb8(100, 130, 190),
            }
        } else {
            match status {
                Status::Active => palette.secondary.strong.color,
                Status::Disabled => palette.secondary.base.color,
                Status::Hovered => palette.secondary.base.color,
                Status::Pressed => palette.secondary.weak.color,
            }
        };
        let border_color = if is_today && !selected {
            Color::from_rgb8(60, 130, 240)
        } else {
            Color::BLACK
        };
        let border_width = if is_today { 2.5 } else { 1.0 };
        Style {
            background: Some(Background::Color(background_color)),
            border: Border {
                color: border_color,
                width: if selected { 2.0 } else { border_width },
                radius: iced::border::Radius {
                    top_left: 60.0,
                    top_right: 60.0,
                    bottom_right: 60.0,
                    bottom_left: 60.0,
                },
            },
            shadow: Shadow {
                color: Color::BLACK,
                offset: Vector { x: 0.01, y: 4.0 },
                blur_radius: 4.0,
            },
            text_color,
            ..Default::default()
        }
    }
}
pub fn add_button(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let text_color = if !palette.is_dark {
        Color::from_rgb8(20, 20, 20)
    } else {
        Color::from_rgb8(220, 220, 220)
    };
    let background_color = match status {
        Status::Active => palette.secondary.strong.color,
        Status::Disabled => palette.secondary.strong.color,
        Status::Hovered => palette.secondary.base.color,
        Status::Pressed => palette.secondary.weak.color,
    };
    Style {
        background: Some(Background::Color(background_color)),
        border: Border {
            color: Color::BLACK,
            width: 1.0,
            radius: iced::border::Radius {
                top_left: 42.0,
                top_right: 42.0,
                bottom_right: 42.0,
                bottom_left: 42.0,
            },
        },
        text_color,
        shadow: Shadow {
            color: Color::BLACK,
            offset: Vector { x: 0.01, y: 4.0 },
            blur_radius: 4.0,
        },
        ..Default::default()
    }
}
pub fn overlay_close_button(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let text_color = if !palette.is_dark {
        Color::from_rgb8(20, 20, 20)
    } else {
        Color::from_rgb8(220, 220, 220)
    };
    let background_color = match status {
        Status::Active => palette.secondary.strong.color,
        Status::Disabled => palette.secondary.strong.color,
        Status::Hovered => palette.secondary.base.color,
        Status::Pressed => palette.secondary.weak.color,
    };
    Style {
        background: Some(Background::Color(background_color)),
        border: Border {
            color: Color::BLACK,
            width: 1.0,
            radius: iced::border::Radius {
                top_left: 15.0,
                top_right: 15.0,
                bottom_right: 15.0,
                bottom_left: 15.0,
            },
        },
        text_color,
        ..Default::default()
    }
}
pub fn record_action_button(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let text_color = if !palette.is_dark {
        Color::from_rgb8(20, 20, 20)
    } else {
        Color::from_rgb8(220, 220, 220)
    };
    let background_color = match status {
        Status::Active => palette.secondary.strong.color,
        Status::Disabled => palette.secondary.strong.color,
        Status::Hovered => palette.secondary.base.color,
        Status::Pressed => palette.secondary.weak.color,
    };
    Style {
        background: Some(Background::Color(background_color)),
        border: Border {
            color: Color::BLACK,
            width: 1.0,
            radius: iced::border::Radius {
                top_left: 10.0,
                top_right: 10.0,
                bottom_right: 10.0,
                bottom_left: 10.0,
            },
        },
        text_color,
        shadow: Shadow {
            color: Color::BLACK,
            offset: Vector { x: 0.01, y: 4.0 },
            blur_radius: 4.0,
        },
        ..Default::default()
    }
}
