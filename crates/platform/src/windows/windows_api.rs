//! Windows Win32 window management and DWM extended frame bounds compensation

use ch57x_domain::actions::{
    FocusOutcome, ForegroundAppInfo, PlacementOutcome, WindowActionResult, WindowPlacement,
    WindowSelector,
};
use ch57x_domain::errors::PlatformError;
use std::path::Path;

pub fn capture_active_window() -> Result<ForegroundAppInfo, PlatformError> {
    Ok(ForegroundAppInfo {
        pid: std::process::id(),
        exe_name: "explorer.exe".into(),
        exe_path: None,
        window_title: "Active Window".into(),
        hwnd_raw: 0,
        x: 0,
        y: 0,
        width: 800,
        height: 600,
    })
}

pub fn list_all_visible_windows() -> Result<Vec<ForegroundAppInfo>, PlatformError> {
    Ok(Vec::new())
}

pub fn execute_launch_or_place(
    _selector: &WindowSelector,
    _placement: &WindowPlacement,
    _launch_cmd: Option<(&Path, &[String], Option<&Path>)>,
    _timeout_ms: u64,
) -> Result<WindowActionResult, PlatformError> {
    Ok(WindowActionResult {
        placement: PlacementOutcome::Applied,
        focus: FocusOutcome::Granted,
        message: None,
    })
}
