//! USB presence and capability probe
//!
//! Rileva la presenza del Macropad (VID 0x1189, PID 0x8890 / 0x8840 / 0x8842)
//! e la disponibilita' del driver UsbDk su Windows.

use serde::{Deserialize, Serialize};

pub const TARGET_VID: u16 = 0x1189;
pub const KNOWN_PIDS: [u16; 3] = [0x8890, 0x8840, 0x8842];

pub const USB_DEVINTERFACE_GUID: &str = "{a5dcbf10-6530-11d2-901f-00c04fb951ed}";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UsbStatus {
    /// Dispositivo connesso e pronto per il flashing (UsbDk OK su Windows o permessi OK su Linux)
    Ready { vid: u16, pid: u16 },
    /// Dispositivo fisicamente collegato ma manca il driver UsbDk / permessi
    DetectedAccessDenied,
    /// Cavo USB scollegato (funzionamento solo Host / Bluetooth)
    Disconnected,
}

impl UsbStatus {
    pub fn is_ready(&self) -> bool {
        matches!(self, UsbStatus::Ready { .. })
    }

    pub fn is_detected(&self) -> bool {
        matches!(
            self,
            UsbStatus::Ready { .. } | UsbStatus::DetectedAccessDenied
        )
    }

    pub fn vid_pid(&self) -> Option<(u16, u16)> {
        match self {
            UsbStatus::Ready { vid, pid } => Some((*vid, *pid)),
            _ => None,
        }
    }
}

/// Esegue il rilevamento dello stato USB attuale del dispositivo
pub fn probe_usb_status() -> UsbStatus {
    #[cfg(target_os = "windows")]
    {
        windows_probe::probe()
    }

    #[cfg(target_os = "linux")]
    {
        linux_probe::probe()
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        UsbStatus::Disconnected
    }
}

/// Verifica se il servizio driver UsbDk e' installato nel sistema (Windows)
pub fn is_usbdk_installed() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_probe::check_usbdk_service()
    }

    #[cfg(not(target_os = "windows"))]
    {
        true // Su Linux non serve UsbDk, servono solo le regole udev
    }
}

#[cfg(target_os = "windows")]
mod windows_probe {
    use super::*;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "advapi32")]
    extern "system" {
        fn RegOpenKeyExW(
            hKey: isize,
            lpSubKey: *const u16,
            ulOptions: u32,
            samDesired: u32,
            phkResult: *mut isize,
        ) -> i32;

        fn RegEnumKeyExW(
            hKey: isize,
            dwIndex: u32,
            lpName: *mut u16,
            lpcchName: *mut u32,
            lpReserved: *mut u32,
            lpClass: *mut u16,
            lpcchClass: *mut u32,
            lpftLastWriteTime: *mut u64,
        ) -> i32;

        fn RegQueryValueExW(
            hKey: isize,
            lpValueName: *const u16,
            lpReserved: *mut u32,
            lpType: *mut u32,
            lpData: *mut u8,
            lpcbData: *mut u32,
        ) -> i32;

        fn RegCloseKey(hKey: isize) -> i32;
    }

    const HKEY_LOCAL_MACHINE: isize = -2147483646; // 0x80000002 as isize
    const KEY_READ: u32 = 0x20019;

    fn to_wide(s: &str) -> Vec<u16> {
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    pub fn check_usbdk_service() -> bool {
        let subkey = to_wide(r"SYSTEM\CurrentControlSet\Services\UsbDk");
        let mut hkey: isize = 0;
        unsafe {
            let res = RegOpenKeyExW(HKEY_LOCAL_MACHINE, subkey.as_ptr(), 0, KEY_READ, &mut hkey);
            if res == 0 {
                RegCloseKey(hkey);
                true
            } else {
                false
            }
        }
    }

    pub fn probe() -> UsbStatus {
        let path = format!(
            r"SYSTEM\CurrentControlSet\Control\DeviceClasses\{}",
            USB_DEVINTERFACE_GUID
        );
        let path_wide = to_wide(&path);
        let mut root_key: isize = 0;

        let res = unsafe {
            RegOpenKeyExW(
                HKEY_LOCAL_MACHINE,
                path_wide.as_ptr(),
                0,
                KEY_READ,
                &mut root_key,
            )
        };

        if res != 0 {
            return fallback_probe();
        }

        let mut found_pid: Option<u16> = None;
        let mut is_linked = false;

        let mut index = 0;
        let mut key_name_buf = [0u16; 512];

        loop {
            let mut name_len = key_name_buf.len() as u32;
            let enum_res = unsafe {
                RegEnumKeyExW(
                    root_key,
                    index,
                    key_name_buf.as_mut_ptr(),
                    &mut name_len,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            };

            if enum_res != 0 {
                break;
            }

            let key_str = String::from_utf16_lossy(&key_name_buf[..name_len as usize]);
            let upper = key_str.to_uppercase();

            // Cerca VID_1189
            if upper.contains("VID_1189") {
                for &pid in &KNOWN_PIDS {
                    let pid_pat = format!("PID_{:04X}", pid);
                    if upper.contains(&pid_pat) {
                        found_pid = Some(pid);

                        // Controlla se la sottochiave #\Control ha Linked == 1
                        let control_path = format!("{}\\{}\\#\\Control", path, key_str);
                        let control_path_w = to_wide(&control_path);
                        let mut control_key: isize = 0;
                        unsafe {
                            if RegOpenKeyExW(
                                HKEY_LOCAL_MACHINE,
                                control_path_w.as_ptr(),
                                0,
                                KEY_READ,
                                &mut control_key,
                            ) == 0
                            {
                                let val_name = to_wide("Linked");
                                let mut val_data: u32 = 0;
                                let mut val_type: u32 = 0;
                                let mut val_size = std::mem::size_of::<u32>() as u32;

                                if RegQueryValueExW(
                                    control_key,
                                    val_name.as_ptr(),
                                    std::ptr::null_mut(),
                                    &mut val_type,
                                    &mut val_data as *mut u32 as *mut u8,
                                    &mut val_size,
                                ) == 0
                                    && val_data == 1
                                {
                                    is_linked = true;
                                }
                                RegCloseKey(control_key);
                            } else {
                                // Se la chiave esiste ma non possiamo leggere il controllo, assumiamo presenza
                                is_linked = true;
                            }
                        }
                        break;
                    }
                }
            }

            if found_pid.is_some() && is_linked {
                break;
            }

            index += 1;
        }

        unsafe {
            RegCloseKey(root_key);
        }

        match (found_pid, is_linked) {
            (Some(pid), true) => {
                if check_usbdk_service() {
                    UsbStatus::Ready {
                        vid: TARGET_VID,
                        pid,
                    }
                } else {
                    UsbStatus::DetectedAccessDenied
                }
            }
            _ => fallback_probe(),
        }
    }

    /// Fallback controllando HKLM\SYSTEM\CurrentControlSet\Enum\USB
    fn fallback_probe() -> UsbStatus {
        for &pid in &KNOWN_PIDS {
            let enum_path = format!(
                r"SYSTEM\CurrentControlSet\Enum\USB\VID_{:04X}&PID_{:04X}",
                TARGET_VID, pid
            );
            let enum_path_w = to_wide(&enum_path);
            let mut dev_key: isize = 0;

            let res = unsafe {
                RegOpenKeyExW(
                    HKEY_LOCAL_MACHINE,
                    enum_path_w.as_ptr(),
                    0,
                    KEY_READ,
                    &mut dev_key,
                )
            };

            if res == 0 {
                unsafe {
                    RegCloseKey(dev_key);
                }
                // Il device e' noto al registro. Verifichiamo se UsbDk e' presente
                return if check_usbdk_service() {
                    UsbStatus::Ready {
                        vid: TARGET_VID,
                        pid,
                    }
                } else {
                    UsbStatus::DetectedAccessDenied
                };
            }
        }

        UsbStatus::Disconnected
    }
}

#[cfg(target_os = "linux")]
mod linux_probe {
    use super::*;
    use std::fs;
    use std::path::Path;

    pub fn probe() -> UsbStatus {
        let sys_bus = Path::new("/sys/bus/usb/devices");
        if !sys_bus.exists() {
            return UsbStatus::Disconnected;
        }

        if let Ok(entries) = fs::read_dir(sys_bus) {
            for entry in entries.flatten() {
                let path = entry.path();
                let vid_path = path.join("idVendor");
                let pid_path = path.join("idProduct");

                if vid_path.exists() && pid_path.exists() {
                    let vid_str = fs::read_to_string(vid_path).unwrap_or_default();
                    let pid_str = fs::read_to_string(pid_path).unwrap_or_default();

                    let vid = u16::from_str_radix(vid_str.trim(), 16).unwrap_or(0);
                    let pid = u16::from_str_radix(pid_str.trim(), 16).unwrap_or(0);

                    if vid == TARGET_VID && KNOWN_PIDS.contains(&pid) {
                        return UsbStatus::Ready { vid, pid };
                    }
                }
            }
        }

        UsbStatus::Disconnected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usb_status_methods() {
        let ready = UsbStatus::Ready {
            vid: 0x1189,
            pid: 0x8890,
        };
        assert!(ready.is_ready());
        assert!(ready.is_detected());
        assert_eq!(ready.vid_pid(), Some((0x1189, 0x8890)));

        let denied = UsbStatus::DetectedAccessDenied;
        assert!(!denied.is_ready());
        assert!(denied.is_detected());
        assert_eq!(denied.vid_pid(), None);

        let disc = UsbStatus::Disconnected;
        assert!(!disc.is_ready());
        assert!(!disc.is_detected());
        assert_eq!(disc.vid_pid(), None);
    }

    #[test]
    fn test_probe_runs_without_panic() {
        let status = probe_usb_status();
        println!("Current USB status: {:?}", status);
        assert!(status.is_detected() || status == UsbStatus::Disconnected);
    }
}
