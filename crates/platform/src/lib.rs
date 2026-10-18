//! Platform abstraction layer for Windows and Linux

pub mod traits;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

pub use traits::*;

#[cfg(target_os = "windows")]
pub use windows::WindowsPlatform as CurrentPlatform;

#[cfg(target_os = "linux")]
pub use linux::LinuxPlatform as CurrentPlatform;

pub fn get_platform() -> CurrentPlatform {
    CurrentPlatform::default()
}
