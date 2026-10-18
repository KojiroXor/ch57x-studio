//! Linux X11 backend detection and capabilities

use ch57x_domain::capabilities::CapabilityMatrix;

/// Checks if the current session has an available X11 display.
pub fn is_x11_session() -> bool {
    std::env::var("DISPLAY").is_ok()
        && !std::env::var("XDG_SESSION_TYPE")
            .map(|s| s.eq_ignore_ascii_case("wayland"))
            .unwrap_or(false)
}

/// Returns the capability matrix for X11 sessions.
pub fn x11_capabilities() -> CapabilityMatrix {
    CapabilityMatrix::linux_x11()
}
