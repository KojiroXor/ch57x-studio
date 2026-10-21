//! Windows Display Brightness control (WMI for laptops/Surface + SetDeviceGammaRamp for instant hardware response)

use ch57x_domain::errors::PlatformError;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::Once;
use std::thread;
use std::time::Duration;

static CURRENT_BRIGHTNESS: AtomicI32 = AtomicI32::new(-1);
static INIT_ONCE: Once = Once::new();
static mut SENDER: Option<Sender<i32>> = None;

fn init_brightness_worker() {
    INIT_ONCE.call_once(|| {
        let (tx, rx) = channel::<i32>();
        unsafe {
            SENDER = Some(tx);
        }

        thread::spawn(move || {
            // Read initial brightness once
            let init_b = query_wmi_brightness().unwrap_or(50);
            CURRENT_BRIGHTNESS.store(init_b as i32, Ordering::SeqCst);

            while let Ok(target) = rx.recv() {
                // Debounce rapid rotations: drain pending values and take the latest
                let mut final_target = target;
                thread::sleep(Duration::from_millis(40));
                while let Ok(newer) = rx.try_recv() {
                    final_target = newer;
                }

                // Apply to hardware
                apply_wmi_brightness(final_target);
                apply_gamma_brightness(final_target);
            }
        });
    });
}

pub fn adjust_brightness(delta: i32) -> Result<u32, PlatformError> {
    init_brightness_worker();

    let cur = CURRENT_BRIGHTNESS.load(Ordering::SeqCst);
    let base = if cur < 0 { 50 } else { cur };
    let new_val = (base + delta).clamp(0, 100);
    CURRENT_BRIGHTNESS.store(new_val, Ordering::SeqCst);

    unsafe {
        if let Some(ref sender) = SENDER {
            let _ = sender.send(new_val);
        }
    }

    Ok(new_val as u32)
}

fn query_wmi_brightness() -> Option<u32> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let output = std::process::Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-WindowStyle",
                "Hidden",
                "-Command",
                "(Get-WmiObject -Namespace root/wmi -Class WmiMonitorBrightness).CurrentBrightness",
            ])
            .creation_flags(0x08000000)
            .output()
            .ok()?;
        let txt = String::from_utf8_lossy(&output.stdout).trim().to_string();
        txt.parse::<u32>().ok()
    }
    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

fn apply_wmi_brightness(level: i32) {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let cmd_str = format!(
            "(Get-WmiObject -Namespace root/wmi -Class WmiMonitorBrightnessMethods).WmiSetBrightness(1, {})",
            level
        );
        let _ = std::process::Command::new("powershell.exe")
            .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &cmd_str])
            .creation_flags(0x08000000)
            .status();
    }
}

fn apply_gamma_brightness(level: i32) {
    #[cfg(target_os = "windows")]
    unsafe {
        use std::ffi::c_void;
        type GetDCFn = unsafe extern "system" fn(isize) -> isize;
        type ReleaseDCFn = unsafe extern "system" fn(isize, isize) -> i32;
        type SetDeviceGammaRampFn = unsafe extern "system" fn(isize, *const c_void) -> i32;

        let user32 =
            windows::Win32::System::LibraryLoader::LoadLibraryA(windows::core::s!("user32.dll"));
        let gdi32 =
            windows::Win32::System::LibraryLoader::LoadLibraryA(windows::core::s!("gdi32.dll"));

        if let (Ok(u32_mod), Ok(gdi_mod)) = (user32, gdi32) {
            let get_dc: Option<GetDCFn> =
                std::mem::transmute(windows::Win32::System::LibraryLoader::GetProcAddress(
                    u32_mod,
                    windows::core::s!("GetDC"),
                ));
            let release_dc: Option<ReleaseDCFn> =
                std::mem::transmute(windows::Win32::System::LibraryLoader::GetProcAddress(
                    u32_mod,
                    windows::core::s!("ReleaseDC"),
                ));
            let set_ramp: Option<SetDeviceGammaRampFn> =
                std::mem::transmute(windows::Win32::System::LibraryLoader::GetProcAddress(
                    gdi_mod,
                    windows::core::s!("SetDeviceGammaRamp"),
                ));

            if let (Some(get_dc_fn), Some(rel_dc_fn), Some(set_ramp_fn)) =
                (get_dc, release_dc, set_ramp)
            {
                let hdc = get_dc_fn(0);
                if hdc != 0 {
                    let factor = (level as f32) / 100.0;
                    let mut ramp = [0u16; 768];
                    for i in 0..256 {
                        let v = ((i as f32 * 257.0 * factor).min(65535.0)) as u16;
                        ramp[i] = v;
                        ramp[i + 256] = v;
                        ramp[i + 512] = v;
                    }
                    let _ = set_ramp_fn(hdc, ramp.as_ptr() as *const c_void);
                    let _ = rel_dc_fn(0, hdc);
                }
            }
        }
    }
}
