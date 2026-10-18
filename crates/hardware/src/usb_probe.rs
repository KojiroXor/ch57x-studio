//! USB presence and capability probe

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UsbStatus {
    Ready { vid: u16, pid: u16 },
    DetectedAccessDenied,
    Disconnected,
}

pub fn probe_usb_status() -> UsbStatus {
    // In Wave 1 Worker-2 will connect Win32 SetupAPI / PnP probe
    UsbStatus::Ready {
        vid: 0x1189,
        pid: 0x8890,
    }
}
