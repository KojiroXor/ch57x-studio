//! Linux Wayland probe and capability detection

use ch57x_domain::capabilities::CapabilityMatrix;

/// Checks if the current session is running under a Wayland compositor.
pub fn is_wayland_session() -> bool {
    std::env::var("WAYLAND_DISPLAY").is_ok()
        || std::env::var("XDG_SESSION_TYPE")
            .map(|s| s.eq_ignore_ascii_case("wayland"))
            .unwrap_or(false)
}

/// Returns the capability matrix for Wayland sessions.
pub fn wayland_capabilities() -> CapabilityMatrix {
    CapabilityMatrix::linux_wayland()
}
