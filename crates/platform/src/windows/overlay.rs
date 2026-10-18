//! Windows Click-through and No-Activate Overlay window styling

use ch57x_domain::errors::PlatformError;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, HWND_TOPMOST,
    SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, WS_EX_LAYERED, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT,
};

/// Configures a native Win32 HWND as a transparent, click-through, always-on-top HUD overlay.
/// Applies `WS_EX_TOPMOST | WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE`.
pub fn configure_hud_overlay(hwnd_raw: usize) -> Result<(), PlatformError> {
    if hwnd_raw == 0 {
        return Err(PlatformError::SystemApi("Invalid HWND 0".into()));
    }
    let hwnd = HWND(hwnd_raw as *mut core::ffi::c_void);

    unsafe {
        let current_ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let overlay_flags = (WS_EX_TOPMOST.0
            | WS_EX_TRANSPARENT.0
            | WS_EX_LAYERED.0
            | WS_EX_TOOLWINDOW.0
            | WS_EX_NOACTIVATE.0) as isize;

        let new_ex = current_ex | overlay_flags;
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_ex);

        // Apply style changes and set topmost without stealing focus
        let res = SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );

        if res.is_err() {
            return Err(PlatformError::SystemApi(
                "Failed to set window pos for overlay".into(),
            ));
        }
    }

    Ok(())
}
