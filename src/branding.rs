use image::imageops::FilterType;

const APP_ICON_PATH: &str = "icons/med-tracker.png";
const TRAY_ICON_PATH: &str = "icons/med-tracker-tray.png";

pub fn app_window_icon() -> Option<iced::window::icon::Icon> {
    let rgba = scaled_rgba(APP_ICON_PATH, 256)?;
    iced::window::icon::from_rgba(rgba, 256, 256).ok()
}

pub fn tray_icon_rgba() -> Option<Vec<u8>> {
    scaled_rgba(TRAY_ICON_PATH, 24)
}

fn scaled_rgba(path: &str, size: u32) -> Option<Vec<u8>> {
    let image = image::open(path).ok()?.into_rgba8();
    Some(image::imageops::resize(&image, size, size, FilterType::Lanczos3).into_raw())
}
