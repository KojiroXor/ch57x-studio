//! Windows Win32 window management and DWM extended frame bounds compensation

use crate::windows::monitors;
use ch57x_domain::actions::{
    FocusOutcome, ForegroundAppInfo, PlacementOutcome, WindowActionResult, WindowCoordinates,
    WindowPlacement, WindowSelector,
};
use ch57x_domain::errors::PlatformError;
use std::ffi::c_void;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, Instant};
use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, BOOL, HWND, LPARAM, RECT};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AllowSetForegroundWindow, EnumWindows, GetClassNameW, GetForegroundWindow, GetWindow,
    GetWindowLongW, GetWindowRect, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
    IsIconic, IsWindowVisible, IsZoomed, SetForegroundWindow, SetWindowPos, ShowWindow, ASFW_ANY,
    GWL_EXSTYLE, GW_OWNER, SWP_NOACTIVATE, SWP_NOZORDER, SW_RESTORE, WS_EX_TOOLWINDOW,
};

const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Reads the full image path for a given PID.
fn get_process_image_path(pid: u32) -> Option<PathBuf> {
    if pid == 0 {
        return None;
    }
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buffer = vec![0u16; 1024];
        let mut size = buffer.len() as u32;
        let res = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_FORMAT(0),
            PWSTR(buffer.as_mut_ptr()),
            &mut size,
        );
        let _ = CloseHandle(handle);

        if res.is_ok() && size > 0 {
            let path_str = String::from_utf16_lossy(&buffer[..size as usize]);
            Some(PathBuf::from(path_str))
        } else {
            None
        }
    }
}

/// Reads the title text for a window.
fn get_window_title(hwnd: HWND) -> String {
    unsafe {
        let len = GetWindowTextLengthW(hwnd);
        if len <= 0 {
            return String::new();
        }
        let mut buffer = vec![0u16; (len + 1) as usize];
        let read_len = GetWindowTextW(hwnd, &mut buffer);
        if read_len > 0 {
            String::from_utf16_lossy(&buffer[..read_len as usize])
        } else {
            String::new()
        }
    }
}

/// Reads the window class name.
fn get_window_class_name(hwnd: HWND) -> String {
    unsafe {
        let mut buffer = [0u16; 256];
        let len = GetClassNameW(hwnd, &mut buffer);
        if len > 0 {
            String::from_utf16_lossy(&buffer[..len as usize])
        } else {
            String::new()
        }
    }
}

/// Returns the visible bounds (x, y, width, height) of a window compensating for DWM shadow borders.
fn get_window_visible_bounds(hwnd: HWND) -> (i32, i32, i32, i32) {
    unsafe {
        let mut rect_dwm = RECT::default();
        let hr = DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut rect_dwm as *mut _ as *mut c_void,
            std::mem::size_of::<RECT>() as u32,
        );
        if hr.is_ok() {
            (
                rect_dwm.left,
                rect_dwm.top,
                rect_dwm.right - rect_dwm.left,
                rect_dwm.bottom - rect_dwm.top,
            )
        } else {
            let mut rect_win32 = RECT::default();
            let _ = GetWindowRect(hwnd, &mut rect_win32);
            (
                rect_win32.left,
                rect_win32.top,
                rect_win32.right - rect_win32.left,
                rect_win32.bottom - rect_win32.top,
            )
        }
    }
}

/// Computes Win32 SetWindowPos coordinates that compensate for invisible DWM resize borders,
/// ensuring the visible frame matches the requested target (x, y, width, height).
fn compute_dwm_compensated_rect(
    hwnd: HWND,
    target_x: i32,
    target_y: i32,
    target_w: i32,
    target_h: i32,
) -> (i32, i32, i32, i32) {
    unsafe {
        let mut rect_win32 = RECT::default();
        let _ = GetWindowRect(hwnd, &mut rect_win32);

        let mut rect_dwm = RECT::default();
        let hr = DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut rect_dwm as *mut _ as *mut c_void,
            std::mem::size_of::<RECT>() as u32,
        );

        if hr.is_ok() {
            let offset_left = rect_dwm.left - rect_win32.left;
            let offset_top = rect_dwm.top - rect_win32.top;
            let offset_right = rect_win32.right - rect_dwm.right;
            let offset_bottom = rect_win32.bottom - rect_dwm.bottom;

            (
                target_x - offset_left,
                target_y - offset_top,
                target_w + offset_left + offset_right,
                target_h + offset_top + offset_bottom,
            )
        } else {
            (target_x, target_y, target_w, target_h)
        }
    }
}

/// Captures the current active foreground window.
pub fn capture_active_window(exclude_pid: u32) -> Result<ForegroundAppInfo, PlatformError> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return Err(PlatformError::WindowNotFound(
                "No foreground window active".into(),
            ));
        }

        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 || (exclude_pid != 0 && pid == exclude_pid) {
            return Err(PlatformError::WindowNotFound(
                "Foreground window excluded or invalid".into(),
            ));
        }

        let window_title = get_window_title(hwnd);
        let exe_path = get_process_image_path(pid);
        let exe_name = exe_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown.exe".into());

        let (x, y, width, height) = get_window_visible_bounds(hwnd);

        Ok(ForegroundAppInfo {
            pid,
            exe_name,
            exe_path,
            window_title,
            hwnd_raw: hwnd.0 as usize,
            x,
            y,
            width,
            height,
        })
    }
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let data_ptr = lparam.0 as *mut (u32, Vec<ForegroundAppInfo>);
    if data_ptr.is_null() {
        return BOOL(0);
    }
    let (exclude_pid, ref mut windows) = &mut *data_ptr;

    if !IsWindowVisible(hwnd).as_bool() {
        return BOOL(1);
    }

    let owner = GetWindow(hwnd, GW_OWNER).unwrap_or(HWND(std::ptr::null_mut()));
    if !owner.0.is_null() {
        return BOOL(1);
    }

    let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
    if (ex_style & WS_EX_TOOLWINDOW.0) != 0 {
        return BOOL(1);
    }

    let title_len = GetWindowTextLengthW(hwnd);
    if title_len <= 0 {
        return BOOL(1);
    }

    let mut pid = 0u32;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    if pid == 0 || (*exclude_pid != 0 && pid == *exclude_pid) {
        return BOOL(1);
    }

    let (x, y, width, height) = get_window_visible_bounds(hwnd);
    if width <= 0 || height <= 0 {
        return BOOL(1);
    }

    let window_title = get_window_title(hwnd);
    let exe_path = get_process_image_path(pid);
    let exe_name = exe_path
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "unknown.exe".into());

    windows.push(ForegroundAppInfo {
        pid,
        exe_name,
        exe_path,
        window_title,
        hwnd_raw: hwnd.0 as usize,
        x,
        y,
        width,
        height,
    });

    BOOL(1)
}

/// Enumerates all visible top-level windows suitable for workspace snapshots.
pub fn list_all_visible_windows(exclude_pid: u32) -> Result<Vec<ForegroundAppInfo>, PlatformError> {
    let mut data: (u32, Vec<ForegroundAppInfo>) = (exclude_pid, Vec::new());
    let lparam = LPARAM(&mut data as *mut _ as isize);

    unsafe {
        let _ = EnumWindows(Some(enum_windows_proc), lparam);
    }

    Ok(data.1)
}

struct WindowSearchContext<'a> {
    selector: &'a WindowSelector,
    found_hwnd: Option<HWND>,
}

unsafe extern "system" fn search_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let ctx = &mut *(lparam.0 as *mut WindowSearchContext);

    if !IsWindowVisible(hwnd).as_bool() {
        return BOOL(1);
    }

    let title = get_window_title(hwnd);

    if let Some(ref title_sub) = ctx.selector.title_contains {
        if !title.to_lowercase().contains(&title_sub.to_lowercase()) {
            return BOOL(1);
        }
    }

    if let Some(ref class_name) = ctx.selector.class_name {
        let actual_class = get_window_class_name(hwnd);
        if !actual_class.eq_ignore_ascii_case(class_name) {
            return BOOL(1);
        }
    }

    if let Some(ref expected_exe) = ctx.selector.exe_name {
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        let exe_path = get_process_image_path(pid);
        let exe_name = exe_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if !exe_name.eq_ignore_ascii_case(expected_exe) {
            return BOOL(1);
        }
    }

    ctx.found_hwnd = Some(hwnd);
    BOOL(0) // Stop enumeration
}

fn find_window(selector: &WindowSelector) -> Option<HWND> {
    let mut ctx = WindowSearchContext {
        selector,
        found_hwnd: None,
    };
    let lparam = LPARAM(&mut ctx as *mut _ as isize);
    unsafe {
        let _ = EnumWindows(Some(search_window_proc), lparam);
    }
    ctx.found_hwnd
}

/// Executes launch or placement logic: finds target window (or launches process),
/// restores if iconic/zoomed, compensates DWM shadows with SetWindowPos, and requests focus.
pub fn execute_launch_or_place(
    selector: &WindowSelector,
    placement: &WindowPlacement,
    launch_cmd: Option<(&Path, &[String], Option<&Path>)>,
    timeout_ms: u64,
) -> Result<WindowActionResult, PlatformError> {
    let mut target_hwnd = find_window(selector);

    if target_hwnd.is_none() {
        if let Some((program, args, working_dir)) = launch_cmd {
            let mut cmd = std::process::Command::new(program);
            cmd.args(args);
            if let Some(dir) = working_dir {
                cmd.current_dir(dir);
            }

            // Launch without showing console window if requested/applicable
            cmd.creation_flags(CREATE_NO_WINDOW);

            cmd.spawn().map_err(|e| {
                PlatformError::ProcessLaunchFailed(format!(
                    "Failed to spawn {}: {}",
                    program.display(),
                    e
                ))
            })?;

            let start = Instant::now();
            let poll_duration = Duration::from_millis(50);
            let max_wait = Duration::from_millis(timeout_ms.max(500));

            while start.elapsed() < max_wait {
                sleep(poll_duration);
                if let Some(hwnd) = find_window(selector) {
                    target_hwnd = Some(hwnd);
                    break;
                }
            }

            if target_hwnd.is_none() {
                return Ok(WindowActionResult {
                    placement: PlacementOutcome::ProcessLaunchedAwaitingWindow,
                    focus: FocusOutcome::NotRequested,
                    message: Some(
                        "Process launched successfully, but window was not found within timeout"
                            .into(),
                    ),
                });
            }
        } else {
            return Ok(WindowActionResult {
                placement: PlacementOutcome::WindowNotFound,
                focus: FocusOutcome::NotRequested,
                message: Some("Window matching selector not found".into()),
            });
        }
    }

    let hwnd = match target_hwnd {
        Some(h) => h,
        None => {
            return Ok(WindowActionResult {
                placement: PlacementOutcome::WindowNotFound,
                focus: FocusOutcome::NotRequested,
                message: Some("Target window unavailable".into()),
            });
        }
    };

    unsafe {
        // If window is minimized, restore it
        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }

        // If window is maximized, restore it so custom dimensions are applied
        if IsZoomed(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }

        // Calculate target geometry
        let (target_x, target_y, target_w, target_h) = match &placement.coordinates {
            WindowCoordinates::Absolute {
                x,
                y,
                width,
                height,
            } => (*x, *y, *width, *height),
            WindowCoordinates::Normalized {
                x,
                y,
                width,
                height,
            } => {
                let work_area = monitors::get_monitor_work_area(placement.monitor_index);
                (
                    work_area.x + (*x * work_area.width as f32).round() as i32,
                    work_area.y + (*y * work_area.height as f32).round() as i32,
                    (*width * work_area.width as f32).round() as i32,
                    (*height * work_area.height as f32).round() as i32,
                )
            }
        };

        // Compensate for DWM shadow borders
        let (adj_x, adj_y, adj_w, adj_h) =
            compute_dwm_compensated_rect(hwnd, target_x, target_y, target_w, target_h);

        let _ = SetWindowPos(
            hwnd,
            None,
            adj_x,
            adj_y,
            adj_w,
            adj_h,
            SWP_NOZORDER | SWP_NOACTIVATE,
        );

        let focus = if placement.bring_to_foreground {
            let _ = AllowSetForegroundWindow(ASFW_ANY);
            let _ = SetForegroundWindow(hwnd);

            // Verify if focus was granted by the operating system
            let current_fg = GetForegroundWindow();
            if current_fg == hwnd {
                FocusOutcome::Granted
            } else {
                FocusOutcome::DeniedByOS
            }
        } else {
            FocusOutcome::NotRequested
        };

        Ok(WindowActionResult {
            placement: PlacementOutcome::Applied,
            focus,
            message: None,
        })
    }
}
