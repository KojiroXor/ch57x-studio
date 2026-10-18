//! Safe process execution wrapper for ch57x-keyboard-tool without shell

use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("CLI binary not found at: {0}")]
    NotFound(PathBuf),

    #[error("CLI process failed with code {0:?}: {1}")]
    ProcessFailed(Option<i32>, String),

    #[error("IO error executing CLI: {0}")]
    Io(#[from] std::io::Error),
}

pub struct Ch57xCli {
    binary_path: PathBuf,
}

impl Ch57xCli {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            binary_path: path.as_ref().to_path_buf(),
        }
    }

    pub fn binary_path(&self) -> &Path {
        &self.binary_path
    }
}
