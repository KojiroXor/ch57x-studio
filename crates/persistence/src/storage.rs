//! OS standard directories and atomic file writing

use std::path::PathBuf;

pub fn get_app_data_dir() -> PathBuf {
    dirs::data_dir()
        .map(|p| p.join("ch57x-studio"))
        .unwrap_or_else(|| PathBuf::from("./data"))
}
