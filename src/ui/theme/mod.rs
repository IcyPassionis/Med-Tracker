pub mod gtk;

use dark_light::Mode;
use iced::Theme;

pub fn system() -> Theme {
    gtk::load()
        .or_else(|| detect_dark_light().ok())
        .unwrap_or(Theme::CatppuccinMocha)
}

fn detect_dark_light() -> Result<Theme, dark_light::Error> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|error| {
            dark_light::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, error))
        })?;
    let _guard = runtime.enter();
    let mode = dark_light::detect()?;
    let theme = match mode {
        Mode::Dark => Theme::Nord,
        Mode::Light => Theme::Light,
        Mode::Unspecified => Theme::TokyoNightLight,
    };
    Ok(theme)
}
