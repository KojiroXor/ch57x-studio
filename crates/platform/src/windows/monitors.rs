//! Windows Per-Monitor DPI aware v2 and monitor enumeration

use windows::Win32::Foundation::{BOOL, LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO,
};
use windows::Win32::UI::HiDpi::{
    SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};

const MONITORINFOF_PRIMARY: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonitorRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalMonitor {
    pub index: usize,
    pub is_primary: bool,
    pub monitor_rect: MonitorRect,
    pub work_area: MonitorRect,
}

/// Enables Per-Monitor DPI Aware v2 for the current process.
/// Returns true if successful or already enabled.
pub fn enable_per_monitor_dpi_v2() -> bool {
    unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2).is_ok() }
}

unsafe extern "system" fn monitor_enum_proc(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _lprect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    let monitors_ptr = lparam.0 as *mut Vec<PhysicalMonitor>;
    if monitors_ptr.is_null() {
        return BOOL(0);
    }
    let monitors = &mut *monitors_ptr;

    let mut mi = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };

    if GetMonitorInfoW(hmonitor, &mut mi).as_bool() {
        let is_primary = (mi.dwFlags & MONITORINFOF_PRIMARY) != 0;
        let index = monitors.len();

        let monitor_rect = MonitorRect {
            x: mi.rcMonitor.left,
            y: mi.rcMonitor.top,
            width: mi.rcMonitor.right - mi.rcMonitor.left,
            height: mi.rcMonitor.bottom - mi.rcMonitor.top,
        };

        let work_area = MonitorRect {
            x: mi.rcWork.left,
            y: mi.rcWork.top,
            width: mi.rcWork.right - mi.rcWork.left,
            height: mi.rcWork.bottom - mi.rcWork.top,
        };

        monitors.push(PhysicalMonitor {
            index,
            is_primary,
            monitor_rect,
            work_area,
        });
    }

    BOOL(1)
}

/// Enumerate all connected physical displays in real pixel coordinates.
/// Ensures DPI awareness v2 is initialized.
pub fn enumerate_monitors() -> Vec<PhysicalMonitor> {
    let _ = enable_per_monitor_dpi_v2();

    let mut monitors = Vec::new();
    let lparam = LPARAM(&mut monitors as *mut Vec<PhysicalMonitor> as isize);

    unsafe {
        let _ = EnumDisplayMonitors(None, None, Some(monitor_enum_proc), lparam);
    }

    monitors
}

/// Gets the work area of a monitor by index, or the primary monitor work area if index is None / out of bounds.
pub fn get_monitor_work_area(index: Option<usize>) -> MonitorRect {
    let monitors = enumerate_monitors();
    if let Some(idx) = index {
        if let Some(mon) = monitors.get(idx) {
            return mon.work_area.clone();
        }
    }
    // Fallback to primary monitor
    if let Some(primary) = monitors.iter().find(|m| m.is_primary) {
        return primary.work_area.clone();
    }
    // Fallback to first monitor or reasonable default
    if let Some(first) = monitors.first() {
        return first.work_area.clone();
    }

    MonitorRect {
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
    }
}
