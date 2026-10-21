//! Genera collegamenti .lnk in %APPDATA%\Microsoft\Windows\Start Menu\Programs\VortexDeck\
//! con hotkey globale assegnata a Windows Explorer per consentire l'apertura immediata di agy
//! e processi anche ad applicazione VortexDeck completamente chiusa!

use ch57x_domain::actions::{ActionDefinition, SlotBinding};
use ch57x_domain::errors::PlatformError;
use ch57x_domain::profile::Profile;
use ch57x_domain::triggers::TriggerBank;
use std::collections::HashSet;
use std::path::PathBuf;

pub fn sync_start_menu_shortcuts(profile: &Profile) -> Result<(), PlatformError> {
    #[cfg(target_os = "windows")]
    unsafe {
        use windows::core::{Interface, PCWSTR};
        use windows::Win32::System::Com::{
            CoCreateInstance, CoInitializeEx, CoUninitialize, IPersistFile, CLSCTX_INPROC_SERVER,
            COINIT_APARTMENTTHREADED,
        };
        use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};

        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| "C:\\".into());
        let dir =
            PathBuf::from(appdata).join("Microsoft\\Windows\\Start Menu\\Programs\\VortexDeck");

        if let Err(e) = std::fs::create_dir_all(&dir) {
            tracing::warn!("Impossibile creare cartella scorciatoie Start Menu: {}", e);
            return Ok(());
        }

        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        let mut active_files = HashSet::new();

        for (&slot_idx, trigger) in &profile.trigger_allocation {
            // Solo Banchi A..D che usano combinazioni Ctrl+Alt+Shift standard riconosciute da Explorer
            if matches!(
                trigger.bank,
                TriggerBank::BankA | TriggerBank::BankB | TriggerBank::BankC | TriggerBank::BankD
            ) {
                if let Some(slot_cfg) = profile.slots.get(&slot_idx) {
                    if let SlotBinding::HostActionRef(action_id) = &slot_cfg.binding {
                        if let Some(ActionDefinition::ExecProcess(action)) =
                            profile.actions.get(action_id)
                        {
                            let file_name = format!("slot_{}.lnk", slot_idx);
                            let file_path = dir.join(&file_name);
                            active_files.insert(file_name);

                            if let Ok(shell_link) = CoCreateInstance::<_, IShellLinkW>(
                                &ShellLink,
                                None,
                                CLSCTX_INPROC_SERVER,
                            ) {
                                let prog_wide: Vec<u16> = action
                                    .program
                                    .to_string_lossy()
                                    .encode_utf16()
                                    .chain(std::iter::once(0))
                                    .collect();
                                let _ = shell_link.SetPath(PCWSTR(prog_wide.as_ptr()));

                                if !action.args.is_empty() {
                                    let args_str = action.args.join(" ");
                                    let args_wide: Vec<u16> =
                                        args_str.encode_utf16().chain(std::iter::once(0)).collect();
                                    let _ = shell_link.SetArguments(PCWSTR(args_wide.as_ptr()));
                                }

                                if let Some(ref wd) = action.working_dir {
                                    let wd_wide: Vec<u16> = wd
                                        .to_string_lossy()
                                        .encode_utf16()
                                        .chain(std::iter::once(0))
                                        .collect();
                                    let _ =
                                        shell_link.SetWorkingDirectory(PCWSTR(wd_wide.as_ptr()));
                                }

                                let (_, vk) = crate::windows::hotkeys::trigger_to_win32(trigger);
                                // In Windows Shell Hotkey:
                                // Byte alto: HOTKEYF_SHIFT (0x01) | HOTKEYF_CONTROL (0x02) | HOTKEYF_ALT (0x04) = 0x07
                                // Byte basso: Virtual Key code
                                let hotkey_word: u16 = (0x07u16 << 8) | (vk.0 & 0xFF);
                                let _ = shell_link.SetHotkey(hotkey_word);

                                if let Ok(persist_file) = shell_link.cast::<IPersistFile>() {
                                    let path_wide: Vec<u16> = file_path
                                        .to_string_lossy()
                                        .encode_utf16()
                                        .chain(std::iter::once(0))
                                        .collect();
                                    let _ = persist_file.Save(PCWSTR(path_wide.as_ptr()), true);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Pulisci scorciatoie obsolete per slot rimossi o riconfigurati
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("slot_")
                    && name.ends_with(".lnk")
                    && !active_files.contains(&name)
                {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }

        CoUninitialize();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = profile;
    }
    Ok(())
}
