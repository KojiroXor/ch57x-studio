pub mod hotkeys;
pub mod input;
pub mod monitors;
pub mod overlay;
pub mod windows_api;

use crate::traits::*;
use ch57x_domain::actions::{ForegroundAppInfo, WindowActionResult, WindowPlacement, WindowSelector};
use ch57x_domain::capabilities::CapabilityMatrix;
use ch57x_domain::errors::PlatformError;
use ch57x_domain::triggers::PhantomTrigger;
use std::path::Path;

pub struct WindowsPlatform {
    hotkeys: WindowsHotkeys,
    windows: WindowsWindowManager,
    input: WindowsInputInjector,
    autostart: WindowsAutostart,
}

impl WindowsPlatform {
    pub fn new() -> Self {
        Self {
            hotkeys: WindowsHotkeys,
            windows: WindowsWindowManager,
            input: WindowsInputInjector,
            autostart: WindowsAutostart,
        }
    }
}

impl Default for WindowsPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl PlatformBackend for WindowsPlatform {
    fn capability_matrix(&self) -> CapabilityMatrix {
        CapabilityMatrix::windows_native()
    }

    fn hotkeys(&self) -> &dyn HotkeyBackend {
        &self.hotkeys
    }

    fn windows(&self) -> &dyn WindowManagerBackend {
        &self.windows
    }

    fn input(&self) -> &dyn InputInjectorBackend {
        &self.input
    }

    fn autostart(&self) -> &dyn AutostartBackend {
        &self.autostart
    }
}

pub struct WindowsHotkeys;

impl HotkeyBackend for WindowsHotkeys {
    fn register_triggers(&self, _triggers: &[PhantomTrigger]) -> Result<Vec<PhantomTrigger>, PlatformError> {
        Ok(Vec::new())
    }

    fn unregister_all(&self) -> Result<(), PlatformError> {
        Ok(())
    }
}

pub struct WindowsWindowManager;

impl WindowManagerBackend for WindowsWindowManager {
    fn capture_foreground_window(&self, _exclude_pid: u32) -> Result<ForegroundAppInfo, PlatformError> {
        windows_api::capture_active_window()
    }

    fn list_visible_windows(&self, _exclude_pid: u32) -> Result<Vec<ForegroundAppInfo>, PlatformError> {
        windows_api::list_all_visible_windows()
    }

    fn launch_or_place_window(
        &self,
        selector: &WindowSelector,
        placement: &WindowPlacement,
        launch_cmd: Option<(&Path, &[String], Option<&Path>)>,
        timeout_ms: u64,
    ) -> Result<WindowActionResult, PlatformError> {
        windows_api::execute_launch_or_place(selector, placement, launch_cmd, timeout_ms)
    }
}

pub struct WindowsInputInjector;

impl InputInjectorBackend for WindowsInputInjector {
    fn can_inject_into_foreground(&self) -> bool {
        input::check_uipi_integrity()
    }

    fn tap_key(&self, key_token: &str) -> Result<(), PlatformError> {
        input::send_tap(key_token)
    }

    fn chord_keys(&self, modifiers: &[String], key_token: &str) -> Result<(), PlatformError> {
        input::send_chord(modifiers, key_token)
    }

    fn type_unicode_text(&self, text: &str) -> Result<(), PlatformError> {
        input::send_unicode(text)
    }
}

pub struct WindowsAutostart;

impl AutostartBackend for WindowsAutostart {
    fn is_enabled(&self) -> Result<bool, PlatformError> {
        Ok(false)
    }

    fn set_enabled(&self, _enabled: bool, _exe_path: &Path) -> Result<(), PlatformError> {
        Ok(())
    }
}
