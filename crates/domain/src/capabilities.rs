use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityStatus {
    FullySupported,
    PartiallySupported(String),
    Unsupported(String),
}

impl CapabilityStatus {
    pub fn is_supported(&self) -> bool {
        matches!(self, CapabilityStatus::FullySupported | CapabilityStatus::PartiallySupported(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityMatrix {
    pub platform_name: String,
    pub window_placement: CapabilityStatus,
    pub window_picker: CapabilityStatus,
    pub global_hotkeys: CapabilityStatus,
    pub input_injection: CapabilityStatus,
    pub system_tray: CapabilityStatus,
    pub hud_overlay: CapabilityStatus,
    pub usb_flashing: CapabilityStatus,
}

impl CapabilityMatrix {
    pub fn windows_native() -> Self {
        Self {
            platform_name: "Windows 10/11 (Win32)".into(),
            window_placement: CapabilityStatus::FullySupported,
            window_picker: CapabilityStatus::FullySupported,
            global_hotkeys: CapabilityStatus::FullySupported,
            input_injection: CapabilityStatus::FullySupported,
            system_tray: CapabilityStatus::FullySupported,
            hud_overlay: CapabilityStatus::FullySupported,
            usb_flashing: CapabilityStatus::FullySupported,
        }
    }

    pub fn linux_x11() -> Self {
        Self {
            platform_name: "Linux (X11)".into(),
            window_placement: CapabilityStatus::FullySupported,
            window_picker: CapabilityStatus::FullySupported,
            global_hotkeys: CapabilityStatus::FullySupported,
            input_injection: CapabilityStatus::FullySupported,
            system_tray: CapabilityStatus::FullySupported,
            hud_overlay: CapabilityStatus::FullySupported,
            usb_flashing: CapabilityStatus::FullySupported,
        }
    }

    pub fn linux_wayland() -> Self {
        Self {
            platform_name: "Linux (Wayland)".into(),
            window_placement: CapabilityStatus::Unsupported("Window placement is restricted by Wayland security protocol".into()),
            window_picker: CapabilityStatus::Unsupported("Arbitrary window picking is restricted on Wayland".into()),
            global_hotkeys: CapabilityStatus::PartiallySupported("Requires desktop portal or compositor global shortcut support".into()),
            input_injection: CapabilityStatus::PartiallySupported("Requires uinput or wlr-protocols".into()),
            system_tray: CapabilityStatus::FullySupported,
            hud_overlay: CapabilityStatus::PartiallySupported("Layer-shell support dependent".into()),
            usb_flashing: CapabilityStatus::FullySupported,
        }
    }
}
