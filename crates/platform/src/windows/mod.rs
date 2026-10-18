pub mod hotkeys;
pub mod input;
pub mod monitors;
pub mod overlay;
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
use windows::core::{w, PCWSTR};
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SZ,
};

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

const RUN_KEY_PATH: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
const APP_VALUE_NAME: PCWSTR = w!("CH57xStudio");

impl AutostartBackend for WindowsAutostart {
    fn is_enabled(&self) -> Result<bool, PlatformError> {
        unsafe {
            let mut hkey = HKEY::default();
            if RegOpenKeyExW(HKEY_CURRENT_USER, RUN_KEY_PATH, None, KEY_READ, &mut hkey).is_err() {
                return Ok(false);
            }

            let mut len = 0u32;
            let query_res =
                RegQueryValueExW(hkey, APP_VALUE_NAME, None, None, None, Some(&mut len));
            let _ = RegCloseKey(hkey);

            Ok(query_res.is_ok() && len > 0)
        }
    }

    fn set_enabled(&self, enabled: bool, exe_path: &Path) -> Result<(), PlatformError> {
        unsafe {
            let mut hkey = HKEY::default();
            if RegOpenKeyExW(HKEY_CURRENT_USER, RUN_KEY_PATH, None, KEY_WRITE, &mut hkey).is_err() {
                return Err(PlatformError::SystemApi(
                    "Failed to open HKCU Run registry key".into(),
                ));
            }

            let res = if enabled {
                let cmd_str = format!("\"{}\" --background\0", exe_path.display());
                let utf16_units: Vec<u16> = cmd_str.encode_utf16().collect();
                let bytes = std::slice::from_raw_parts(
                    utf16_units.as_ptr() as *const u8,
                    utf16_units.len() * 2,
                );

                RegSetValueExW(hkey, APP_VALUE_NAME, None, REG_SZ, Some(bytes)).ok()
            } else {
                let _ = RegDeleteValueW(hkey, APP_VALUE_NAME);
                Ok(())
            };

            let _ = RegCloseKey(hkey);

            res.map_err(|e| {
                PlatformError::SystemApi(format!("Failed to update autostart in registry: {e}"))
            })
        }
    }
}
