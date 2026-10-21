//! System Tray integration for Windows (with clean non-Windows stubs)
//!
//! Visualizza l'icona VortexDeck nella barra delle notifiche di sistema
//! e fornisce un menu contestuale per controllo rapido:
//! - "Apri VortexDeck"
//! - "Pausa Automazioni (ON/OFF)"
//! - "Visualizza HUD OSD"
//! - Separatore
//! - "Esci"

use crossbeam_channel::{unbounded, Receiver};
use tracing::info;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    ShowWindow,
    TogglePause,
    ToggleAutostart,
    ShowHud,
    Quit,
}

pub struct TrayHandle {
    pub receiver: Receiver<TrayAction>,
    #[cfg(target_os = "windows")]
    _thread: std::thread::JoinHandle<()>,
}

impl TrayHandle {
    /// Inizializza il System Tray su Windows in un thread dedicato
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();

        #[cfg(target_os = "windows")]
        let _thread = {
            let tx = sender.clone();
            std::thread::spawn(move || {
                windows_tray::run_tray_loop(tx);
            })
        };

        #[cfg(not(target_os = "windows"))]
        let _ = sender;

        info!("System Tray inizializzato.");
        Self {
            receiver,
            #[cfg(target_os = "windows")]
            _thread,
        }
    }

    /// Collega il contesto egui per svegliare l'event loop di eframe all'arrivo di comandi tray
    pub fn set_repaint_ctx(&self, ctx: egui::Context) {
        #[cfg(target_os = "windows")]
        windows_tray::set_repaint_context(ctx);
        #[cfg(not(target_os = "windows"))]
        let _ = ctx;
    }
}

impl Drop for TrayHandle {
    fn drop(&mut self) {
        #[cfg(target_os = "windows")]
        windows_tray::cleanup();
    }
}

pub fn notify_repaint() {
    #[cfg(target_os = "windows")]
    windows_tray::notify_repaint();
}

pub fn update_tray_icon(is_paused: bool) {
    #[cfg(target_os = "windows")]
    windows_tray::update_tray_icon(is_paused);
    #[cfg(not(target_os = "windows"))]
    let _ = is_paused;
}

impl Default for TrayHandle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "windows")]
mod windows_tray {
    use super::TrayAction;
    use crossbeam_channel::Sender;
    use windows::core::w;
    use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM};
    use windows::Win32::UI::Shell::{
        Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu,
        DispatchMessageW, GetCursorPos, GetMessageW, LoadIconW, PostQuitMessage, RegisterClassW,
        SetForegroundWindow, TrackPopupMenu, IDI_APPLICATION, MF_SEPARATOR, MF_STRING, MSG,
        TPM_BOTTOMALIGN, TPM_LEFTALIGN, TPM_RIGHTBUTTON, WM_APP, WM_DESTROY, WM_LBUTTONDBLCLK,
        WM_LBUTTONUP, WM_RBUTTONUP, WNDCLASSW, WS_OVERLAPPEDWINDOW,
        LoadImageW, HICON, IMAGE_ICON, LR_DEFAULTSIZE, LR_LOADFROMFILE,
    };

    use std::sync::OnceLock;

    const TRAY_ICON_ICO: &[u8] = include_bytes!("../../../assets/icons/vortexdeck_tray_icon.ico");
    const TRAY_PAUSED_ICO: &[u8] = include_bytes!("../../../assets/icons/vortexdeck_tray_paused.ico");

    fn resolve_icon_path(is_paused: bool) -> std::path::PathBuf {
        let filename = if is_paused {
            "vortexdeck_tray_paused.ico"
        } else {
            "vortexdeck_tray_icon.ico"
        };

        let local_path = std::path::Path::new("assets").join("icons").join(filename);
        if local_path.exists() {
            return local_path;
        }

        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(parent) = exe_path.parent() {
                let exe_rel = parent.join("assets").join("icons").join(filename);
                if exe_rel.exists() {
                    return exe_rel;
                }
            }
        }

        let temp_path = std::env::temp_dir().join(filename);
        let bytes = if is_paused {
            TRAY_PAUSED_ICO
        } else {
            TRAY_ICON_ICO
        };
        let _ = std::fs::write(&temp_path, bytes);
        temp_path
    }

    pub unsafe fn load_tray_icon(is_paused: bool) -> HICON {
        let ico_path = resolve_icon_path(is_paused);
        let wide_path: Vec<u16> = ico_path
            .to_string_lossy()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        match LoadImageW(
            None,
            windows::core::PCWSTR(wide_path.as_ptr()),
            IMAGE_ICON,
            0,
            0,
            LR_LOADFROMFILE | LR_DEFAULTSIZE,
        ) {
            Ok(handle) if !handle.is_invalid() => HICON(handle.0),
            _ => LoadIconW(None, IDI_APPLICATION).unwrap_or_default(),
        }
    }

    pub fn update_tray_icon(is_paused: bool) {
        if let Some(&raw_hwnd) = TRAY_HWND.get() {
            unsafe {
                let hwnd = HWND(raw_hwnd as *mut _);
                let icon = load_tray_icon(is_paused);
                let nid = NOTIFYICONDATAW {
                    cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                    hWnd: hwnd,
                    uID: 1,
                    uFlags: NIF_ICON,
                    hIcon: icon,
                    ..Default::default()
                };
                let _ = Shell_NotifyIconW(NIM_MODIFY, &nid);
            }
        }
    }

    const WM_TRAY_CALLBACK: u32 = WM_APP + 101;
    const ID_TRAY_OPEN: usize = 1001;
    const ID_TRAY_PAUSE: usize = 1002;
    const ID_TRAY_AUTOSTART: usize = 1003;
    const ID_TRAY_HUD: usize = 1004;
    const ID_TRAY_EXIT: usize = 1005;

    static GLOBAL_TRAY_SENDER: OnceLock<Sender<TrayAction>> = OnceLock::new();
    static REPAINT_CTX: OnceLock<egui::Context> = OnceLock::new();
    static TRAY_HWND: OnceLock<isize> = OnceLock::new();

    pub fn set_repaint_context(ctx: egui::Context) {
        let _ = REPAINT_CTX.set(ctx);
    }

    pub fn notify_repaint() {
        if let Some(ctx) = REPAINT_CTX.get() {
            ctx.request_repaint();
        }
    }

    pub fn remove_tray_icon(hwnd: HWND) {
        unsafe {
            let nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: hwnd,
                uID: 1,
                ..Default::default()
            };
            let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
        }
    }

    pub fn cleanup() {
        if let Some(&raw_hwnd) = TRAY_HWND.get() {
            let hwnd = HWND(raw_hwnd as *mut _);
            remove_tray_icon(hwnd);
            unsafe {
                let _ = windows::Win32::UI::WindowsAndMessaging::PostMessageW(
                    Some(hwnd),
                    windows::Win32::UI::WindowsAndMessaging::WM_CLOSE,
                    WPARAM(0),
                    LPARAM(0),
                );
            }
        }
    }

    unsafe fn restore_main_window() {
        if let Ok(main_hwnd) = windows::Win32::UI::WindowsAndMessaging::FindWindowW(
            None,
            windows::core::w!("VortexDeck — CH57x Studio"),
        )
        .or_else(|_| {
            windows::Win32::UI::WindowsAndMessaging::FindWindowW(
                None,
                windows::core::w!("VortexDeck"),
            )
        }) {
            if !main_hwnd.0.is_null() {
                let _ = windows::Win32::UI::WindowsAndMessaging::ShowWindow(
                    main_hwnd,
                    windows::Win32::UI::WindowsAndMessaging::SW_RESTORE,
                );
                let _ = windows::Win32::UI::WindowsAndMessaging::ShowWindow(
                    main_hwnd,
                    windows::Win32::UI::WindowsAndMessaging::SW_SHOW,
                );
                let _ = windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow(main_hwnd);
            }
        }
    }

    pub fn run_tray_loop(sender: Sender<TrayAction>) {
        unsafe {
            let _ = GLOBAL_TRAY_SENDER.set(sender);

            let class_name = w!("VortexDeckTrayClass");
            let icon = load_tray_icon(false);

            let wc = WNDCLASSW {
                lpfnWndProc: Some(tray_wnd_proc),
                hInstance: HINSTANCE::default(),
                lpszClassName: class_name,
                ..Default::default()
            };
            let _ = RegisterClassW(&wc);

            let hwnd = match CreateWindowExW(
                windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
                class_name,
                w!("VortexDeckTrayWindow"),
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                0,
                0,
                None,
                None,
                None,
                None,
            ) {
                Ok(h) => {
                    let _ = TRAY_HWND.set(h.0 as isize);
                    h
                }
                Err(_) => return,
            };

            let mut nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: hwnd,
                uID: 1,
                uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
                uCallbackMessage: WM_TRAY_CALLBACK,
                hIcon: icon,
                ..Default::default()
            };

            let tip = "VortexDeck — CH57x Studio\0"
                .encode_utf16()
                .collect::<Vec<u16>>();
            let len = tip.len().min(nid.szTip.len());
            nid.szTip[..len].copy_from_slice(&tip[..len]);

            let _ = Shell_NotifyIconW(NIM_ADD, &nid);

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = DispatchMessageW(&msg);
            }

            let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
        }
    }

    unsafe extern "system" fn tray_wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_TRAY_CALLBACK => {
                let event = lparam.0 as u32;
                if event == WM_RBUTTONUP {
                    let mut pt = POINT::default();
                    let _ = GetCursorPos(&mut pt);
                    let _ = SetForegroundWindow(hwnd);

                    let is_auto = ch57x_platform::is_autostart_enabled();
                    let check_flag = if is_auto {
                        windows::Win32::UI::WindowsAndMessaging::MF_CHECKED
                    } else {
                        windows::Win32::UI::WindowsAndMessaging::MF_UNCHECKED
                    };

                    let hmenu = CreatePopupMenu().unwrap_or_default();
                    if !hmenu.0.is_null() {
                        let _ = AppendMenuW(hmenu, MF_STRING, ID_TRAY_OPEN, w!("Apri VortexDeck"));
                        let _ =
                            AppendMenuW(hmenu, MF_STRING, ID_TRAY_PAUSE, w!("Pausa Automazioni"));
                        let _ = AppendMenuW(
                            hmenu,
                            MF_STRING | check_flag,
                            ID_TRAY_AUTOSTART,
                            w!("Avvio automatico con Windows"),
                        );
                        let _ =
                            AppendMenuW(hmenu, MF_STRING, ID_TRAY_HUD, w!("Visualizza HUD OSD"));
                        let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, None);
                        let _ = AppendMenuW(hmenu, MF_STRING, ID_TRAY_EXIT, w!("Esci"));

                        let cmd = TrackPopupMenu(
                            hmenu,
                            TPM_LEFTALIGN | TPM_BOTTOMALIGN | TPM_RIGHTBUTTON,
                            pt.x,
                            pt.y,
                            None,
                            hwnd,
                            None,
                        );

                        if cmd.as_bool() {
                            // Handled in WM_COMMAND
                        }
                        let _ = DestroyMenu(hmenu);
                    }
                } else if event == WM_LBUTTONUP || event == WM_LBUTTONDBLCLK {
                    restore_main_window();
                    if let Some(sender) = GLOBAL_TRAY_SENDER.get() {
                        let _ = sender.send(TrayAction::ShowWindow);
                        notify_repaint();
                    }
                }
                LRESULT(0)
            }
            windows::Win32::UI::WindowsAndMessaging::WM_COMMAND => {
                let cmd_id = wparam.0 & 0xffff;
                if let Some(sender) = GLOBAL_TRAY_SENDER.get() {
                    match cmd_id {
                        ID_TRAY_OPEN => {
                            restore_main_window();
                            let _ = sender.send(TrayAction::ShowWindow);
                            notify_repaint();
                        }
                        ID_TRAY_PAUSE => {
                            let _ = sender.send(TrayAction::TogglePause);
                            notify_repaint();
                        }
                        ID_TRAY_AUTOSTART => {
                            let current = ch57x_platform::is_autostart_enabled();
                            let _ = ch57x_platform::set_autostart_enabled(!current);
                            let _ = sender.send(TrayAction::ToggleAutostart);
                            notify_repaint();
                        }
                        ID_TRAY_HUD => {
                            let _ = sender.send(TrayAction::ShowHud);
                            notify_repaint();
                        }
                        ID_TRAY_EXIT => {
                            remove_tray_icon(hwnd);
                            let _ = sender.send(TrayAction::Quit);
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                }
                LRESULT(0)
            }
            windows::Win32::UI::WindowsAndMessaging::WM_CLOSE => {
                let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(hwnd);
                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
