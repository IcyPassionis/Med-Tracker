use std::sync::atomic::{AtomicU32, Ordering};

use crate::application::states::settings::{DEFAULT_FONT_SIZE, MAX_FONT_SIZE, MIN_FONT_SIZE};

/// Font size, in pixels, that the hardcoded UI sizes were designed around.
const BASE_TEXT_SIZE: f32 = 16.0;
/// Keep very small labels readable even at the smallest setting.
const MIN_TEXT_SIZE: f32 = 8.0;

static FONT_SIZE: AtomicU32 = AtomicU32::new(DEFAULT_FONT_SIZE as u32);

/// Updates the scale used by every text widget in the current frame.
pub fn set_font_size(font_size: u16) {
    FONT_SIZE.store(
        font_size.clamp(MIN_FONT_SIZE, MAX_FONT_SIZE) as u32,
        Ordering::Relaxed,
    );
}

/// Scales a hardcoded text size for the current font size setting.
pub fn text_px(base: f32) -> f32 {
    text_px_for(FONT_SIZE.load(Ordering::Relaxed) as u16, base)
}

fn text_px_for(font_size: u16, base: f32) -> f32 {
    let scale = font_size.clamp(MIN_FONT_SIZE, MAX_FONT_SIZE) as f32 / BASE_TEXT_SIZE;

    (base * scale).max(MIN_TEXT_SIZE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scales_text_relative_to_the_body_size() {
        assert_eq!(text_px_for(14, 16.0), 14.0);
        assert_eq!(text_px_for(12, 32.0), 24.0);
        assert_eq!(text_px_for(18, 13.0), 14.625);
    }

    #[test]
    fn clamps_font_size_and_keeps_tiny_labels_readable() {
        assert_eq!(text_px_for(0, 16.0), text_px_for(MIN_FONT_SIZE, 16.0));
        assert_eq!(text_px_for(99, 16.0), text_px_for(MAX_FONT_SIZE, 16.0));
        assert_eq!(text_px_for(MIN_FONT_SIZE, 8.0), MIN_TEXT_SIZE);
    }
}
