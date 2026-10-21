pub mod brightness;
pub mod hotkeys;
pub mod input;
pub mod monitors;
pub mod overlay;
pub mod shortcut_sync;
pub mod windows_api;

use crate::traits::*;
use ch57x_domain::actions::{
    ForegroundAppInfo, WindowActionResult, WindowPlacement, WindowSelector,
};
use ch57x_domain::capabilities::CapabilityMatrix;
use ch57x_domain::errors::PlatformError;
use ch57x_domain::triggers::PhantomTrigger;
use hotkeys::WindowsHotkeysBackend;
use std::path::Path;

pub struct WindowsPlatform {
    hotkeys: WindowsHotkeysBackend,
    windows: WindowsWindowManager,
    input: WindowsInputInjector,
    autostart: WindowsAutostart,
}

impl WindowsPlatform {
    pub fn new() -> Self {
        Self {
            hotkeys: WindowsHotkeysBackend::new(),
            windows: WindowsWindowManager,
            input: WindowsInputInjector,
            autostart: WindowsAutostart,
        }
    }

    pub fn is_autostart_enabled(&self) -> bool {
        windows_api::is_autostart_enabled()
    }

    pub fn set_autostart_enabled(&self, enabled: bool) -> Result<(), PlatformError> {
        windows_api::set_autostart_enabled(enabled)
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

    fn adjust_brightness(&self, delta: i32) -> Result<u32, PlatformError> {
        brightness::adjust_brightness(delta)
    }

    fn sync_shortcuts(
        &self,
        profile: &ch57x_domain::profile::Profile,
    ) -> Result<(), PlatformError> {
        shortcut_sync::sync_start_menu_shortcuts(profile)
    }

    fn is_autostart_enabled(&self) -> bool {
        windows_api::is_autostart_enabled()
    }

    fn set_autostart_enabled(&self, enabled: bool) -> Result<(), PlatformError> {
        windows_api::set_autostart_enabled(enabled)
    }
}

impl WindowManagerBackend for WindowsPlatform {
    fn capture_foreground_window(
        &self,
        exclude_pid: u32,
    ) -> Result<ForegroundAppInfo, PlatformError> {
        self.windows.capture_foreground_window(exclude_pid)
    }

    fn list_visible_windows(
        &self,
        exclude_pid: u32,
    ) -> Result<Vec<ForegroundAppInfo>, PlatformError> {
        self.windows.list_visible_windows(exclude_pid)
    }

    fn launch_or_place_window(
        &self,
        selector: &WindowSelector,
        placement: &WindowPlacement,
        launch_cmd: Option<(&Path, &[String], Option<&Path>)>,
        timeout_ms: u64,
    ) -> Result<WindowActionResult, PlatformError> {
        self.windows
            .launch_or_place_window(selector, placement, launch_cmd, timeout_ms)
    }
}

impl InputInjectorBackend for WindowsPlatform {
    fn can_inject_into_foreground(&self) -> bool {
        self.input.can_inject_into_foreground()
    }

    fn tap_key(&self, token: &str) -> Result<(), PlatformError> {
        self.input.tap_key(token)
    }

    fn chord_keys(&self, modifiers: &[String], key_token: &str) -> Result<(), PlatformError> {
        self.input.chord_keys(modifiers, key_token)
    }

    fn type_unicode_text(&self, text: &str) -> Result<(), PlatformError> {
        self.input.type_unicode_text(text)
    }
}

impl HotkeyBackend for WindowsHotkeysBackend {
    fn register_triggers(
        &self,
        triggers: &[PhantomTrigger],
    ) -> Result<Vec<PhantomTrigger>, PlatformError> {
        self.register_triggers(triggers)
    }

    fn unregister_all(&self) -> Result<(), PlatformError> {
        self.unregister_all()
    }

    fn subscribe(&self) -> crossbeam_channel::Receiver<PhantomTrigger> {
        hotkeys::subscribe_hotkeys()
    }
}

pub struct WindowsWindowManager;

impl WindowManagerBackend for WindowsWindowManager {
    fn capture_foreground_window(
        &self,
        exclude_pid: u32,
    ) -> Result<ForegroundAppInfo, PlatformError> {
        windows_api::capture_active_window(exclude_pid)
    }

    fn list_visible_windows(
        &self,
        exclude_pid: u32,
    ) -> Result<Vec<ForegroundAppInfo>, PlatformError> {
        windows_api::list_all_visible_windows(exclude_pid)
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

    fn is_autostart_enabled(&self) -> bool {
        windows_api::is_autostart_enabled()
    }

    fn set_autostart_enabled(&self, enabled: bool) -> Result<(), PlatformError> {
        windows_api::set_autostart_enabled(enabled)
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
        Ok(windows_api::is_autostart_enabled())
    }

    fn set_enabled(&self, enabled: bool, _exe_path: &Path) -> Result<(), PlatformError> {
        windows_api::set_autostart_enabled(enabled)
    }
}
