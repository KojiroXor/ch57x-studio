//! Gestione scorciatoie Start Menu per VortexDeck.
//! Disabilita SetHotkey statici per evitare che Windows Explorer intercetti la combinazione
//! con una directory fissa prima di VortexDeck.

use ch57x_domain::errors::PlatformError;
use ch57x_domain::profile::Profile;
use std::path::PathBuf;

pub fn sync_start_menu_shortcuts(_profile: &Profile) -> Result<(), PlatformError> {
    #[cfg(target_os = "windows")]
    {
        // Rimuovi eventuali file .lnk generati in precedenza per impedire a explorer.exe
        // di intercettare le hotkey del Macropad con working directory statica!
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| "C:\\".into());
        let dir =
            PathBuf::from(appdata).join("Microsoft\\Windows\\Start Menu\\Programs\\VortexDeck");

        if dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let _ = std::fs::remove_file(&path);
                    }
                }
            }
            let _ = std::fs::remove_dir_all(&dir);
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = _profile;
    }
    Ok(())
}
