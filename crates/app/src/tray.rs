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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    ShowWindow,
    TogglePause,
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

        info!("System Tray inizializzato.");
        Self {
            receiver,
            #[cfg(target_os = "windows")]
            _thread,
        }
    }
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
        Shell_NotifyIconW, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW, NIF_ICON, NIF_MESSAGE, NIF_TIP,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DispatchMessageW,
        GetCursorPos, GetMessageW, LoadIconW, PostQuitMessage, RegisterClassW, SetForegroundWindow,
        TrackPopupMenu, IDI_APPLICATION, MF_SEPARATOR, MF_STRING, MSG, TPM_BOTTOMALIGN,
        TPM_LEFTALIGN, TPM_RIGHTBUTTON, WM_APP, WM_DESTROY, WM_LBUTTONDBLCLK, WM_LBUTTONUP,
        WM_RBUTTONUP, WNDCLASSW, WS_OVERLAPPEDWINDOW,
    };

    const WM_TRAY_CALLBACK: u32 = WM_APP + 101;
    const ID_TRAY_OPEN: usize = 1001;
    const ID_TRAY_PAUSE: usize = 1002;
    const ID_TRAY_HUD: usize = 1003;
    const ID_TRAY_EXIT: usize = 1004;

    static mut GLOBAL_TRAY_SENDER: Option<Sender<TrayAction>> = None;

    pub fn run_tray_loop(sender: Sender<TrayAction>) {
        unsafe {
            GLOBAL_TRAY_SENDER = Some(sender);

            let class_name = w!("VortexDeckTrayClass");
            let icon = LoadIconW(None, IDI_APPLICATION).unwrap_or_default();

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
                Ok(h) => h,
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

            let tip = "VortexDeck — CH57x Studio\0".encode_utf16().collect::<Vec<u16>>();
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

                    let hmenu = CreatePopupMenu().unwrap_or_default();
                    if !hmenu.0.is_null() {
                        let _ = AppendMenuW(hmenu, MF_STRING, ID_TRAY_OPEN, w!("Apri VortexDeck"));
                        let _ = AppendMenuW(hmenu, MF_STRING, ID_TRAY_PAUSE, w!("Pausa Automazioni"));
                        let _ = AppendMenuW(hmenu, MF_STRING, ID_TRAY_HUD, w!("Visualizza HUD OSD"));
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
                    if let Some(ref sender) = GLOBAL_TRAY_SENDER {
                        let _ = sender.send(TrayAction::ShowWindow);
                    }
                }
                LRESULT(0)
            }
            windows::Win32::UI::WindowsAndMessaging::WM_COMMAND => {
                let cmd_id = wparam.0 & 0xffff;
                if let Some(ref sender) = GLOBAL_TRAY_SENDER {
                    match cmd_id {
                        ID_TRAY_OPEN => {
                            let _ = sender.send(TrayAction::ShowWindow);
                        }
                        ID_TRAY_PAUSE => {
                            let _ = sender.send(TrayAction::TogglePause);
                        }
                        ID_TRAY_HUD => {
                            let _ = sender.send(TrayAction::ShowHud);
                        }
                        ID_TRAY_EXIT => {
                            let _ = sender.send(TrayAction::Quit);
                            PostQuitMessage(0);
                        }
                        _ => {}
                    }
                }
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
