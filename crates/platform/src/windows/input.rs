//! Windows SendInput and UIPI integrity check

use ch57x_domain::errors::PlatformError;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{
    GetSidSubAuthority, GetSidSubAuthorityCount, GetTokenInformation, TokenIntegrityLevel,
    TOKEN_MANDATORY_LABEL, TOKEN_QUERY,
};
use windows::Win32::System::Threading::{
    GetCurrentProcess, OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
    KEYEVENTF_UNICODE, VIRTUAL_KEY, VK_BACK, VK_CAPITAL, VK_DELETE, VK_DOWN, VK_END, VK_ESCAPE,
    VK_F1, VK_F10, VK_F11, VK_F12, VK_F13, VK_F14, VK_F15, VK_F16, VK_F17, VK_F18, VK_F19, VK_F2,
    VK_F20, VK_F21, VK_F22, VK_F23, VK_F24, VK_F3, VK_F4, VK_F5, VK_F6, VK_F7, VK_F8, VK_F9,
    VK_HOME, VK_INSERT, VK_LCONTROL, VK_LEFT, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_MEDIA_NEXT_TRACK,
    VK_MEDIA_PLAY_PAUSE, VK_MEDIA_PREV_TRACK, VK_MEDIA_STOP, VK_NEXT, VK_PAUSE, VK_PRIOR,
    VK_RCONTROL, VK_RETURN, VK_RIGHT, VK_RMENU, VK_RSHIFT, VK_RWIN, VK_SCROLL, VK_SNAPSHOT,
    VK_SPACE, VK_TAB, VK_UP, VK_VOLUME_DOWN, VK_VOLUME_MUTE, VK_VOLUME_UP,
};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

/// Helper to query the integrity RID of a process token.
unsafe fn get_token_integrity_rid(proc_handle: HANDLE) -> Result<u32, ()> {
    let mut token = HANDLE::default();
    if OpenProcessToken(proc_handle, TOKEN_QUERY, &mut token).is_err() {
        return Err(());
    }

    let mut len_needed = 0u32;
    let _ = GetTokenInformation(token, TokenIntegrityLevel, None, 0, &mut len_needed);
    if len_needed == 0 {
        let _ = CloseHandle(token);
        return Err(());
    }

    let mut buf = vec![0u8; len_needed as usize];
    let res = GetTokenInformation(
        token,
        TokenIntegrityLevel,
        Some(buf.as_mut_ptr() as *mut _),
        len_needed,
        &mut len_needed,
    );
    let _ = CloseHandle(token);

    if res.is_err() {
        return Err(());
    }

    let p_label = buf.as_ptr() as *const TOKEN_MANDATORY_LABEL;
    let sid = (*p_label).Label.Sid;
    if sid.0.is_null() {
        return Err(());
    }

    let count_ptr = GetSidSubAuthorityCount(sid);
    if count_ptr.is_null() || *count_ptr == 0 {
        return Err(());
    }

    let rid_ptr = GetSidSubAuthority(sid, (*count_ptr - 1) as u32);
    if rid_ptr.is_null() {
        return Err(());
    }

    Ok(*rid_ptr)
}

/// Checks whether the current process can inject input into the current foreground window
/// under Windows User Interface Privilege Isolation (UIPI).
pub fn check_uipi_integrity() -> bool {
    unsafe {
        let fg_hwnd = GetForegroundWindow();
        if fg_hwnd.0.is_null() {
            return true;
        }

        let mut fg_pid = 0u32;
        GetWindowThreadProcessId(fg_hwnd, Some(&mut fg_pid));
        if fg_pid == 0 || fg_pid == std::process::id() {
            return true;
        }

        let current_proc = GetCurrentProcess();
        let our_rid = get_token_integrity_rid(current_proc).unwrap_or(0x2000);

        let fg_proc = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, fg_pid) {
            Ok(h) => h,
            Err(_) => {
                // If opening the process failed with Access Denied, foreground is elevated
                return false;
            }
        };

        let fg_rid = match get_token_integrity_rid(fg_proc) {
            Ok(rid) => {
                let _ = CloseHandle(fg_proc);
                rid
            }
            Err(_) => {
                let _ = CloseHandle(fg_proc);
                // If querying token failed, assume elevated
                return false;
            }
        };

        our_rid >= fg_rid
    }
}

/// Maps key token strings to Win32 Virtual Key codes.
pub fn token_to_virtual_key(token: &str) -> Option<VIRTUAL_KEY> {
    let lower = token.trim().to_lowercase();
    match lower.as_str() {
        // Function keys
        "f1" => Some(VK_F1),
        "f2" => Some(VK_F2),
        "f3" => Some(VK_F3),
        "f4" => Some(VK_F4),
        "f5" => Some(VK_F5),
        "f6" => Some(VK_F6),
        "f7" => Some(VK_F7),
        "f8" => Some(VK_F8),
        "f9" => Some(VK_F9),
        "f10" => Some(VK_F10),
        "f11" => Some(VK_F11),
        "f12" => Some(VK_F12),
        "f13" => Some(VK_F13),
        "f14" => Some(VK_F14),
        "f15" => Some(VK_F15),
        "f16" => Some(VK_F16),
        "f17" => Some(VK_F17),
        "f18" => Some(VK_F18),
        "f19" => Some(VK_F19),
        "f20" => Some(VK_F20),
        "f21" => Some(VK_F21),
        "f22" => Some(VK_F22),
        "f23" => Some(VK_F23),
        "f24" => Some(VK_F24),

        // Modifiers
        "ctrl" | "control" | "lctrl" => Some(VK_LCONTROL),
        "rctrl" => Some(VK_RCONTROL),
        "shift" | "lshift" => Some(VK_LSHIFT),
        "rshift" => Some(VK_RSHIFT),
        "alt" | "lalt" => Some(VK_LMENU),
        "ralt" => Some(VK_RMENU),
        "win" | "super" | "lwin" => Some(VK_LWIN),
        "rwin" => Some(VK_RWIN),

        // Navigation and editing
        "enter" | "return" => Some(VK_RETURN),
        "tab" => Some(VK_TAB),
        "space" => Some(VK_SPACE),
        "backspace" => Some(VK_BACK),
        "esc" | "escape" => Some(VK_ESCAPE),
        "delete" | "del" => Some(VK_DELETE),
        "insert" | "ins" => Some(VK_INSERT),
        "home" => Some(VK_HOME),
        "end" => Some(VK_END),
        "pageup" | "pgup" => Some(VK_PRIOR),
        "pagedown" | "pgdown" => Some(VK_NEXT),
        "left" => Some(VK_LEFT),
        "right" => Some(VK_RIGHT),
        "up" => Some(VK_UP),
        "down" => Some(VK_DOWN),
        "capslock" => Some(VK_CAPITAL),
        "printscreen" | "prtscr" => Some(VK_SNAPSHOT),
        "scrolllock" => Some(VK_SCROLL),
        "pause" => Some(VK_PAUSE),

        // Media keys
        "volumeup" => Some(VK_VOLUME_UP),
        "volumedown" => Some(VK_VOLUME_DOWN),
        "mute" => Some(VK_VOLUME_MUTE),
        "play" | "playpause" => Some(VK_MEDIA_PLAY_PAUSE),
        "next" | "nextsong" => Some(VK_MEDIA_NEXT_TRACK),
        "prev" | "prevsong" => Some(VK_MEDIA_PREV_TRACK),
        "stop" => Some(VK_MEDIA_STOP),

        // Letters and numbers
        s if s.len() == 1 => {
            let ch = s.chars().next().unwrap();
            if ch.is_ascii_alphabetic() {
                Some(VIRTUAL_KEY(ch.to_ascii_uppercase() as u16))
            } else if ch.is_ascii_digit() {
                Some(VIRTUAL_KEY(ch as u16))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn make_key_input(vk: VIRTUAL_KEY, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn make_unicode_input(ch: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: ch,
                dwFlags: flags | KEYEVENTF_UNICODE,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

pub fn send_tap(key_token: &str) -> Result<(), PlatformError> {
    if !check_uipi_integrity() {
        return Err(PlatformError::PermissionDenied);
    }

    if let Some(vk) = token_to_virtual_key(key_token) {
        let inputs = [
            make_key_input(vk, KEYBD_EVENT_FLAGS(0)),
            make_key_input(vk, KEYEVENTF_KEYUP),
        ];
        unsafe {
            let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            if sent != inputs.len() as u32 {
                return Err(PlatformError::SystemApi("SendInput tap failed".into()));
            }
        }
    } else {
        send_unicode(key_token)?;
    }

    Ok(())
}

pub fn send_chord(modifiers: &[String], key_token: &str) -> Result<(), PlatformError> {
    if !check_uipi_integrity() {
        return Err(PlatformError::PermissionDenied);
    }

    let mut mod_vks = Vec::new();
    for m in modifiers {
        if let Some(vk) = token_to_virtual_key(m) {
            mod_vks.push(vk);
        } else {
            return Err(PlatformError::SystemApi(format!(
                "Unknown modifier token: {m}"
            )));
        }
    }

    let target_vk = token_to_virtual_key(key_token)
        .ok_or_else(|| PlatformError::SystemApi(format!("Unknown key token: {key_token}")))?;

    let mut inputs = Vec::new();

    // Key down for modifiers
    for &m_vk in &mod_vks {
        inputs.push(make_key_input(m_vk, KEYBD_EVENT_FLAGS(0)));
    }

    // Key down + up for target
    inputs.push(make_key_input(target_vk, KEYBD_EVENT_FLAGS(0)));
    inputs.push(make_key_input(target_vk, KEYEVENTF_KEYUP));

    // Key up for modifiers in reverse
    for &m_vk in mod_vks.iter().rev() {
        inputs.push(make_key_input(m_vk, KEYEVENTF_KEYUP));
    }

    unsafe {
        let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        if sent != inputs.len() as u32 {
            return Err(PlatformError::SystemApi("SendInput chord failed".into()));
        }
    }

    Ok(())
}

pub fn send_unicode(text: &str) -> Result<(), PlatformError> {
    if !check_uipi_integrity() {
        return Err(PlatformError::PermissionDenied);
    }

    let mut inputs = Vec::new();
    for code_unit in text.encode_utf16() {
        inputs.push(make_unicode_input(code_unit, KEYBD_EVENT_FLAGS(0)));
        inputs.push(make_unicode_input(code_unit, KEYEVENTF_KEYUP));
    }

    if inputs.is_empty() {
        return Ok(());
    }

    unsafe {
        let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        if sent != inputs.len() as u32 {
            return Err(PlatformError::SystemApi("SendInput unicode failed".into()));
        }
    }

    Ok(())
}
