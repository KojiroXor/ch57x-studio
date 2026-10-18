pub mod wayland_stub;
pub mod x11_backend;

use crate::traits::*;
use ch57x_domain::actions::{
    ForegroundAppInfo, WindowActionResult, WindowPlacement, WindowSelector,
};
use ch57x_domain::capabilities::CapabilityMatrix;
use ch57x_domain::errors::PlatformError;
use ch57x_domain::triggers::PhantomTrigger;
use std::path::Path;

pub struct LinuxPlatform;

impl LinuxPlatform {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LinuxPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl PlatformBackend for LinuxPlatform {
    fn capability_matrix(&self) -> CapabilityMatrix {
        if wayland_stub::is_wayland_session() {
            wayland_stub::wayland_capabilities()
        } else {
            x11_backend::x11_capabilities()
        }
    }

    fn hotkeys(&self) -> &dyn HotkeyBackend {
        self
    }

    fn windows(&self) -> &dyn WindowManagerBackend {
        self
    }

    fn input(&self) -> &dyn InputInjectorBackend {
        self
    }

    fn autostart(&self) -> &dyn AutostartBackend {
        self
    }
}

impl HotkeyBackend for LinuxPlatform {
    fn register_triggers(
        &self,
        _triggers: &[PhantomTrigger],
    ) -> Result<Vec<PhantomTrigger>, PlatformError> {
        if wayland_stub::is_wayland_session() {
            Err(PlatformError::Unsupported(
                "Global hotkeys require X11 or desktop portal under Wayland".into(),
            ))
        } else {
            Ok(Vec::new())
        }
    }

    fn unregister_all(&self) -> Result<(), PlatformError> {
        Ok(())
    }
}

impl WindowManagerBackend for LinuxPlatform {
    fn capture_foreground_window(
        &self,
        _exclude_pid: u32,
    ) -> Result<ForegroundAppInfo, PlatformError> {
        if wayland_stub::is_wayland_session() {
            Err(PlatformError::Unsupported(
                "Arbitrary window capture is restricted by Wayland security protocol".into(),
            ))
        } else {
            Err(PlatformError::Unsupported(
                "X11 active window capture requires X11 session running".into(),
            ))
        }
    }

    fn list_visible_windows(
        &self,
        _exclude_pid: u32,
    ) -> Result<Vec<ForegroundAppInfo>, PlatformError> {
        Ok(Vec::new())
    }

    fn launch_or_place_window(
        &self,
        _selector: &WindowSelector,
        _placement: &WindowPlacement,
        _launch_cmd: Option<(&Path, &[String], Option<&Path>)>,
        _timeout_ms: u64,
    ) -> Result<WindowActionResult, PlatformError> {
        if wayland_stub::is_wayland_session() {
            Err(PlatformError::Unsupported(
                "Window placement is restricted on Wayland compositors".into(),
            ))
        } else {
            Err(PlatformError::Unsupported(
                "X11 window placement requires X11 session running".into(),
            ))
        }
    }
}

impl InputInjectorBackend for LinuxPlatform {
    fn can_inject_into_foreground(&self) -> bool {
        false
    }

    fn tap_key(&self, _key_token: &str) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported("Linux input injection".into()))
    }

    fn chord_keys(&self, _modifiers: &[String], _key_token: &str) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported("Linux input injection".into()))
    }

    fn type_unicode_text(&self, _text: &str) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported("Linux input injection".into()))
    }
}

impl AutostartBackend for LinuxPlatform {
    fn is_enabled(&self) -> Result<bool, PlatformError> {
        Ok(false)
    }

    fn set_enabled(&self, _enabled: bool, _exe_path: &Path) -> Result<(), PlatformError> {
        Ok(())
    }
}
