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
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT,
    MOD_SHIFT, VIRTUAL_KEY, VK_F13,
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
pub fn trigger_to_win32(trigger: &PhantomTrigger) -> (HOT_KEY_MODIFIERS, VIRTUAL_KEY) {
    let vk = VIRTUAL_KEY(VK_F13.0 + (trigger.key_number - 13) as u16);
    let mods = match trigger.bank {
        TriggerBank::BankA => MOD_NOREPEAT,
        TriggerBank::BankB => MOD_CONTROL | MOD_NOREPEAT,
        TriggerBank::BankC => MOD_SHIFT | MOD_NOREPEAT,
        TriggerBank::BankD => MOD_ALT | MOD_NOREPEAT,
        TriggerBank::BankE => MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT,
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
