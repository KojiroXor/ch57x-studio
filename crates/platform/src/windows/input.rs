//! Windows SendInput and UIPI integrity check

use ch57x_domain::errors::PlatformError;

pub fn check_uipi_integrity() -> bool {
    true
}

pub fn send_tap(_key_token: &str) -> Result<(), PlatformError> {
    Ok(())
}

pub fn send_chord(_modifiers: &[String], _key_token: &str) -> Result<(), PlatformError> {
    Ok(())
}

pub fn send_unicode(_text: &str) -> Result<(), PlatformError> {
    Ok(())
}
