mod load;
mod save;
mod settings;

use std::path::PathBuf;

pub use load::load_tracker;
pub use save::save_tracker;
pub use settings::{load_settings, save_settings};

fn data_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("med-tracker"))
}
