//! Windows hotkeys module using RegisterHotKey message loop

use ch57x_domain::errors::PlatformError;
use ch57x_domain::triggers::{PhantomTrigger, TriggerBank};
use crossbeam_channel::{bounded, Receiver, Sender};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::{self, JoinHandle};
use windows::Win32::Foundation::{BOOL, LPARAM, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN,
    VIRTUAL_KEY, VK_F1, VK_F10, VK_F11, VK_F2, VK_F3, VK_F4, VK_F5, VK_F6, VK_F7, VK_F8, VK_F9,
    VK_OEM_MINUS,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetMessageW, PeekMessageW, PostQuitMessage, PostThreadMessageW, MSG, PM_NOREMOVE, WM_HOTKEY,
    WM_QUIT, WM_USER,
};

const WM_DISPATCH_CMD: u32 = WM_USER + 101;

enum HotkeyCommand {
    Register {
        triggers: Vec<PhantomTrigger>,
        reply: Sender<Result<Vec<PhantomTrigger>, PlatformError>>,
    },
    UnregisterAll {
        reply: Sender<Result<(), PlatformError>>,
    },
    Shutdown,
}

static EVENT_SUBSCRIBERS: OnceLock<Mutex<Vec<Sender<PhantomTrigger>>>> = OnceLock::new();

fn get_subscribers() -> &'static Mutex<Vec<Sender<PhantomTrigger>>> {
    EVENT_SUBSCRIBERS.get_or_init(|| Mutex::new(Vec::new()))
}

/// Subscribe to incoming global hotkey trigger events fired by the dedicated thread.
pub fn subscribe_hotkeys() -> Receiver<PhantomTrigger> {
    let (tx, rx) = crossbeam_channel::unbounded();
    if let Ok(mut subs) = get_subscribers().lock() {
        subs.push(tx);
    }
    rx
}

fn broadcast_hotkey_event(trigger: PhantomTrigger) {
    if let Ok(mut subs) = get_subscribers().lock() {
        subs.retain(|sub| sub.send(trigger).is_ok());
    }
}

/// Converts a PhantomTrigger to Win32 modifiers and Virtual Key code.
/// Rimosso MOD_NOREPEAT per consentire ripetizione fluida su rotazione continua manopole.
pub fn trigger_to_win32(trigger: &PhantomTrigger) -> (HOT_KEY_MODIFIERS, VIRTUAL_KEY) {
    let vk = match trigger.key_token() {
        "f1" => VK_F1,
        "f2" => VK_F2,
        "f3" => VK_F3,
        "f4" => VK_F4,
        "f5" => VK_F5,
        "f6" => VK_F6,
        "f7" => VK_F7,
        "f8" => VK_F8,
        "f9" => VK_F9,
        "f10" => VK_F10,
        "f11" => VK_F11,
        "0" => VIRTUAL_KEY(0x30),
        "1" => VIRTUAL_KEY(0x31),
        "2" => VIRTUAL_KEY(0x32),
        "3" => VIRTUAL_KEY(0x33),
        "4" => VIRTUAL_KEY(0x34),
        "5" => VIRTUAL_KEY(0x35),
        "6" => VIRTUAL_KEY(0x36),
        "7" => VIRTUAL_KEY(0x37),
        "8" => VIRTUAL_KEY(0x38),
        "9" => VIRTUAL_KEY(0x39),
        "a" => VIRTUAL_KEY(0x41),
        "b" => VIRTUAL_KEY(0x42),
        "c" => VIRTUAL_KEY(0x43),
        "d" => VIRTUAL_KEY(0x44),
        "e" => VIRTUAL_KEY(0x45),
        "f" => VIRTUAL_KEY(0x46),
        "g" => VIRTUAL_KEY(0x47),
        "h" => VIRTUAL_KEY(0x48),
        "i" => VIRTUAL_KEY(0x49),
        "j" => VIRTUAL_KEY(0x4A),
        "k" => VIRTUAL_KEY(0x4B),
        "l" => VIRTUAL_KEY(0x4C),
        "m" => VIRTUAL_KEY(0x4D),
        "n" => VIRTUAL_KEY(0x4E),
        "o" => VIRTUAL_KEY(0x4F),
        "p" => VIRTUAL_KEY(0x50),
        "q" => VIRTUAL_KEY(0x51),
        "r" => VIRTUAL_KEY(0x52),
        "s" => VIRTUAL_KEY(0x53),
        "t" => VIRTUAL_KEY(0x54),
        "u" => VIRTUAL_KEY(0x55),
        "v" => VIRTUAL_KEY(0x56),
        "w" => VIRTUAL_KEY(0x57),
        "x" => VIRTUAL_KEY(0x58),
        "y" => VIRTUAL_KEY(0x59),
        "z" => VIRTUAL_KEY(0x5A),
        "minus" => VK_OEM_MINUS,
        _ => VK_F1,
    };
    let mods = match trigger.bank {
        TriggerBank::BankA | TriggerBank::BankB | TriggerBank::BankC | TriggerBank::BankD => {
            MOD_CONTROL | MOD_ALT | MOD_SHIFT
        }
        TriggerBank::BankE => MOD_CONTROL | MOD_ALT | MOD_WIN,
    };
    (mods, vk)
}

struct DedicatedHotkeyThread {
    cmd_tx: Sender<HotkeyCommand>,
    thread_id: u32,
    _handle: JoinHandle<()>,
}

impl DedicatedHotkeyThread {
    fn spawn() -> Self {
        let (cmd_tx, cmd_rx) = bounded::<HotkeyCommand>(32);
        let (init_tx, init_rx) = bounded::<u32>(1);

        let handle = thread::Builder::new()
            .name("ch57x-win32-hotkeys".into())
            .spawn(move || {
                let tid = unsafe { GetCurrentThreadId() };

                // Force creation of the message queue for this thread
                let mut dummy_msg = MSG::default();
                unsafe {
                    let _ = PeekMessageW(&mut dummy_msg, None, 0, 0, PM_NOREMOVE);
                }

                init_tx.send(tid).ok();

                let mut active_hotkeys: HashMap<i32, PhantomTrigger> = HashMap::new();
                let mut msg = MSG::default();

                loop {
                    let ret = unsafe { GetMessageW(&mut msg, None, 0, 0) };
                    if ret == BOOL(0) || ret == BOOL(-1) {
                        break;
                    }

                    if msg.message == WM_DISPATCH_CMD {
                        while let Ok(cmd) = cmd_rx.try_recv() {
                            match cmd {
                                HotkeyCommand::Register { triggers, reply } => {
                                    // First unregister existing hotkeys
                                    for &id in active_hotkeys.keys() {
                                        unsafe {
                                            let _ = UnregisterHotKey(None, id);
                                        }
                                    }
                                    active_hotkeys.clear();

                                    let mut conflicts = Vec::new();
                                    for trigger in triggers {
                                        let id = trigger.pool_index() as i32 + 1;
                                        let (mods, vk) = trigger_to_win32(&trigger);
                                        let res =
                                            unsafe { RegisterHotKey(None, id, mods, vk.0 as u32) };
                                        if res.is_ok() {
                                            active_hotkeys.insert(id, trigger);
                                        } else {
                                            conflicts.push(trigger);
                                        }
                                    }
                                    reply.send(Ok(conflicts)).ok();
                                }
                                HotkeyCommand::UnregisterAll { reply } => {
                                    for &id in active_hotkeys.keys() {
                                        unsafe {
                                            let _ = UnregisterHotKey(None, id);
                                        }
                                    }
                                    active_hotkeys.clear();
                                    reply.send(Ok(())).ok();
                                }
                                HotkeyCommand::Shutdown => {
                                    for &id in active_hotkeys.keys() {
                                        unsafe {
                                            let _ = UnregisterHotKey(None, id);
                                        }
                                    }
                                    active_hotkeys.clear();
                                    unsafe {
                                        PostQuitMessage(0);
                                    }
                                    return;
                                }
                            }
                        }
                    } else if msg.message == WM_HOTKEY {
                        let hotkey_id = msg.wParam.0 as i32;
                        if let Some(&trigger) = active_hotkeys.get(&hotkey_id) {
                            broadcast_hotkey_event(trigger);
                        }
                    } else if msg.message == WM_QUIT {
                        break;
                    }
                }
            })
            .expect("Failed to spawn dedicated hotkeys thread");

        let thread_id = init_rx.recv().expect("Failed to receive hotkey thread ID");

        Self {
            cmd_tx,
            thread_id,
            _handle: handle,
        }
    }

    fn register_triggers(
        &self,
        triggers: &[PhantomTrigger],
    ) -> Result<Vec<PhantomTrigger>, PlatformError> {
        let (reply_tx, reply_rx) = bounded(1);
        self.cmd_tx
            .send(HotkeyCommand::Register {
                triggers: triggers.to_vec(),
                reply: reply_tx,
            })
            .map_err(|e| PlatformError::Internal(format!("Hotkey thread dead: {e}")))?;

        unsafe {
            let _ = PostThreadMessageW(self.thread_id, WM_DISPATCH_CMD, WPARAM(0), LPARAM(0));
        }

        reply_rx
            .recv()
            .map_err(|e| PlatformError::Internal(format!("Failed to receive reply: {e}")))?
    }

    fn unregister_all(&self) -> Result<(), PlatformError> {
        let (reply_tx, reply_rx) = bounded(1);
        self.cmd_tx
            .send(HotkeyCommand::UnregisterAll { reply: reply_tx })
            .map_err(|e| PlatformError::Internal(format!("Hotkey thread dead: {e}")))?;

        unsafe {
            let _ = PostThreadMessageW(self.thread_id, WM_DISPATCH_CMD, WPARAM(0), LPARAM(0));
        }

        reply_rx
            .recv()
            .map_err(|e| PlatformError::Internal(format!("Failed to receive reply: {e}")))?
    }
}

impl Drop for DedicatedHotkeyThread {
    fn drop(&mut self) {
        let _ = self.cmd_tx.send(HotkeyCommand::Shutdown);
        unsafe {
            let _ = PostThreadMessageW(self.thread_id, WM_DISPATCH_CMD, WPARAM(0), LPARAM(0));
        }
    }
}

pub struct WindowsHotkeysBackend {
    thread: Arc<Mutex<DedicatedHotkeyThread>>,
}

impl WindowsHotkeysBackend {
    pub fn new() -> Self {
        Self {
            thread: Arc::new(Mutex::new(DedicatedHotkeyThread::spawn())),
        }
    }

    pub fn register_triggers(
        &self,
        triggers: &[PhantomTrigger],
    ) -> Result<Vec<PhantomTrigger>, PlatformError> {
        let thread = self
            .thread
            .lock()
            .map_err(|_| PlatformError::Internal("Hotkey thread lock poisoned".into()))?;
        thread.register_triggers(triggers)
    }

    pub fn unregister_all(&self) -> Result<(), PlatformError> {
        let thread = self
            .thread
            .lock()
            .map_err(|_| PlatformError::Internal("Hotkey thread lock poisoned".into()))?;
        thread.unregister_all()
    }
}

impl Default for WindowsHotkeysBackend {
    fn default() -> Self {
        Self::new()
    }
}
