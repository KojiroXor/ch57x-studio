//! Macro actor: Async, cancellable execution of MacroSequenceAction with focus guard and concurrency policies

use ch57x_domain::actions::{
    ForegroundAppInfo, MacroConcurrencyPolicy, MacroSequenceAction, MacroStep, WindowSelector,
};
use ch57x_platform::traits::{InputInjectorBackend, WindowManagerBackend};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Errors that can occur during macro execution.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum MacroError {
    #[error("Macro execution was cancelled")]
    Cancelled,
    #[error("Focus guard triggered: expected hwnd {expected_hwnd} ({expected_exe}), but foreground is hwnd {actual_hwnd} ({actual_exe})")]
    FocusLost {
        expected_exe: String,
        expected_hwnd: usize,
        actual_exe: String,
        actual_hwnd: usize,
    },
    #[error("Timed out waiting for target window")]
    WindowWaitTimeout,
    #[error("Clipboard operation failed: {0}")]
    Clipboard(String),
    #[error("Platform input error: {0}")]
    Platform(String),
}

/// Outcome of submitting a macro sequence action to the actor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacroRunOutcome {
    Started,
    IgnoredAlreadyRunning,
    Queued,
    QueueFull,
}

/// Abstract clipboard provider allowing unit testing and OS clipboard access.
pub trait ClipboardProvider: Send + Sync {
    fn get_text(&self) -> Result<Option<String>, MacroError>;
    fn set_text(&self, text: &str) -> Result<(), MacroError>;
}

/// In-memory clipboard for tests and headless environments.
#[derive(Default)]
pub struct InMemoryClipboard {
    content: Mutex<Option<String>>,
}

impl InMemoryClipboard {
    pub fn new() -> Self {
        Self {
            content: Mutex::new(None),
        }
    }

    pub fn with_content(text: &str) -> Self {
        Self {
            content: Mutex::new(Some(text.to_string())),
        }
    }
}

impl ClipboardProvider for InMemoryClipboard {
    fn get_text(&self) -> Result<Option<String>, MacroError> {
        let lock = self
            .content
            .lock()
            .map_err(|e| MacroError::Clipboard(e.to_string()))?;
        Ok(lock.clone())
    }

    fn set_text(&self, text: &str) -> Result<(), MacroError> {
        let mut lock = self
            .content
            .lock()
            .map_err(|e| MacroError::Clipboard(e.to_string()))?;
        *lock = Some(text.to_string());
        Ok(())
    }
}

/// OS native clipboard provider using Win32 API on Windows.
pub struct OsClipboard;

impl ClipboardProvider for OsClipboard {
    #[cfg(target_os = "windows")]
    fn get_text(&self) -> Result<Option<String>, MacroError> {
        use windows::Win32::Foundation::HGLOBAL;
        use windows::Win32::System::DataExchange::{
            CloseClipboard, GetClipboardData, OpenClipboard,
        };
        use windows::Win32::System::Memory::{GlobalLock, GlobalUnlock};

        unsafe {
            if OpenClipboard(None).is_err() {
                return Err(MacroError::Clipboard("Failed to open clipboard".into()));
            }

            // CF_UNICODETEXT = 13
            let handle = match GetClipboardData(13) {
                Ok(h) if !h.0.is_null() => h,
                _ => {
                    let _ = CloseClipboard();
                    return Ok(None);
                }
            };

            let ptr = GlobalLock(HGLOBAL(handle.0));
            if ptr.is_null() {
                let _ = CloseClipboard();
                return Ok(None);
            }

            let u16_ptr = ptr as *const u16;
            let mut len = 0;
            while *u16_ptr.add(len) != 0 {
                len += 1;
            }
            let slice = std::slice::from_raw_parts(u16_ptr, len);
            let text = String::from_utf16_lossy(slice);

            let _ = GlobalUnlock(HGLOBAL(handle.0));
            let _ = CloseClipboard();
            Ok(Some(text))
        }
    }

    #[cfg(target_os = "windows")]
    fn set_text(&self, text: &str) -> Result<(), MacroError> {
        use windows::Win32::Foundation::HANDLE;
        use windows::Win32::System::DataExchange::{
            CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
        };
        use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};

        unsafe {
            if OpenClipboard(None).is_err() {
                return Err(MacroError::Clipboard("Failed to open clipboard".into()));
            }

            let _ = EmptyClipboard();
            let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
            let bytes_len = utf16.len() * std::mem::size_of::<u16>();

            let h_mem = match GlobalAlloc(GMEM_MOVEABLE, bytes_len) {
                Ok(h) if !h.0.is_null() => h,
                _ => {
                    let _ = CloseClipboard();
                    return Err(MacroError::Clipboard("GlobalAlloc failed".into()));
                }
            };

            let ptr = GlobalLock(h_mem);
            if ptr.is_null() {
                let _ = CloseClipboard();
                return Err(MacroError::Clipboard("GlobalLock failed".into()));
            }

            std::ptr::copy_nonoverlapping(utf16.as_ptr() as *const u8, ptr as *mut u8, bytes_len);
            let _ = GlobalUnlock(h_mem);

            let set_res = SetClipboardData(13, Some(HANDLE(h_mem.0)));
            let _ = CloseClipboard();

            if set_res.is_err() {
                return Err(MacroError::Clipboard("SetClipboardData failed".into()));
            }

            Ok(())
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn get_text(&self) -> Result<Option<String>, MacroError> {
        Ok(None)
    }

    #[cfg(not(target_os = "windows"))]
    fn set_text(&self, _text: &str) -> Result<(), MacroError> {
        Ok(())
    }
}

/// Releases modifier keys in emergency / cancel situations.
pub fn emergency_release_modifiers(input: &dyn InputInjectorBackend) {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_LCONTROL,
            VK_LMENU, VK_LSHIFT, VK_LWIN, VK_RCONTROL, VK_RMENU, VK_RSHIFT, VK_RWIN,
        };

        let mod_vks = [
            VK_LCONTROL,
            VK_RCONTROL,
            VK_LSHIFT,
            VK_RSHIFT,
            VK_LMENU,
            VK_RMENU,
            VK_LWIN,
            VK_RWIN,
        ];
        let inputs: Vec<INPUT> = mod_vks
            .iter()
            .map(|&vk| INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: vk,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            })
            .collect();

        unsafe {
            let _ = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        }
    }

    // Generic fallback: tap standard modifier keys with release
    let _ = input.tap_key("ctrl");
    let _ = input.tap_key("shift");
    let _ = input.tap_key("alt");
}

struct MacroState {
    is_running: bool,
    cancel_flag: Option<Arc<AtomicBool>>,
    queue: VecDeque<MacroSequenceAction>,
}

/// MacroActor orchestrates macro sequences, concurrency policies, focus guards, and clipboard operations.
pub struct MacroActor {
    input: Arc<dyn InputInjectorBackend>,
    window_manager: Arc<dyn WindowManagerBackend>,
    clipboard: Arc<dyn ClipboardProvider>,
    state: Arc<Mutex<MacroState>>,
}

impl MacroActor {
    pub fn new(
        input: Arc<dyn InputInjectorBackend>,
        window_manager: Arc<dyn WindowManagerBackend>,
    ) -> Self {
        Self {
            input,
            window_manager,
            clipboard: Arc::new(OsClipboard),
            state: Arc::new(Mutex::new(MacroState {
                is_running: false,
                cancel_flag: None,
                queue: VecDeque::new(),
            })),
        }
    }

    pub fn with_clipboard(
        input: Arc<dyn InputInjectorBackend>,
        window_manager: Arc<dyn WindowManagerBackend>,
        clipboard: Arc<dyn ClipboardProvider>,
    ) -> Self {
        Self {
            input,
            window_manager,
            clipboard,
            state: Arc::new(Mutex::new(MacroState {
                is_running: false,
                cancel_flag: None,
                queue: VecDeque::new(),
            })),
        }
    }

    pub fn is_running(&self) -> bool {
        self.state.lock().map(|s| s.is_running).unwrap_or(false)
    }

    /// Cancels any currently running macro and flushes the queue.
    pub fn cancel_current(&self) {
        if let Ok(mut state) = self.state.lock() {
            if let Some(flag) = state.cancel_flag.take() {
                flag.store(true, Ordering::SeqCst);
            }
            state.queue.clear();
            state.is_running = false;
        }
        emergency_release_modifiers(self.input.as_ref());
    }

    /// Submits a macro action according to its concurrency policy.
    pub fn submit(&self, action: MacroSequenceAction) -> MacroRunOutcome {
        let mut state = match self.state.lock() {
            Ok(s) => s,
            Err(_) => return MacroRunOutcome::QueueFull,
        };

        if !state.is_running {
            let cancel_flag = Arc::new(AtomicBool::new(false));
            state.is_running = true;
            state.cancel_flag = Some(cancel_flag.clone());
            drop(state);

            self.spawn_execution(action, cancel_flag);
            return MacroRunOutcome::Started;
        }

        // Action submitted while another is running
        match action.concurrency {
            MacroConcurrencyPolicy::IgnoreIfRunning => MacroRunOutcome::IgnoredAlreadyRunning,
            MacroConcurrencyPolicy::RestartCurrent => {
                if let Some(flag) = state.cancel_flag.take() {
                    flag.store(true, Ordering::SeqCst);
                }
                emergency_release_modifiers(self.input.as_ref());

                let cancel_flag = Arc::new(AtomicBool::new(false));
                state.cancel_flag = Some(cancel_flag.clone());
                state.queue.clear();
                drop(state);

                self.spawn_execution(action, cancel_flag);
                MacroRunOutcome::Started
            }
            MacroConcurrencyPolicy::QueueBounded(limit) => {
                if state.queue.len() >= limit {
                    MacroRunOutcome::QueueFull
                } else {
                    state.queue.push_back(action);
                    MacroRunOutcome::Queued
                }
            }
        }
    }

    fn spawn_execution(&self, action: MacroSequenceAction, cancel_flag: Arc<AtomicBool>) {
        let actor_state = self.state.clone();
        let input = self.input.clone();
        let wm = self.window_manager.clone();
        let clip = self.clipboard.clone();

        tokio::spawn(async move {
            let res =
                Self::execute_sequence(&action, input.as_ref(), wm.as_ref(), clip.as_ref(), &cancel_flag)
                    .await;

            if let Err(err) = res {
                tracing::warn!("Macro execution finished with error: {err}");
                emergency_release_modifiers(input.as_ref());
            }

            // Check if there are queued macros
            let next_action = {
                if let Ok(mut state) = actor_state.lock() {
                    if let Some(next) = state.queue.pop_front() {
                        let new_flag = Arc::new(AtomicBool::new(false));
                        state.cancel_flag = Some(new_flag.clone());
                        Some((next, new_flag))
                    } else {
                        state.is_running = false;
                        state.cancel_flag = None;
                        None
                    }
                } else {
                    None
                }
            };

            if let Some((next, new_flag)) = next_action {
                let s_clone = actor_state.clone();
                let in_clone = input.clone();
                let wm_clone = wm.clone();
                let clip_clone = clip.clone();

                tokio::spawn(async move {
                    let _ = Self::execute_sequence(
                        &next,
                        in_clone.as_ref(),
                        wm_clone.as_ref(),
                        clip_clone.as_ref(),
                        &new_flag,
                    )
                    .await;
                    if let Ok(mut s) = s_clone.lock() {
                        s.is_running = false;
                        s.cancel_flag = None;
                    }
                });
            }
        });
    }

    /// Directly executes a macro sequence asynchronously with cancellation checks and focus guard.
    pub async fn execute_sequence(
        action: &MacroSequenceAction,
        input: &dyn InputInjectorBackend,
        window_manager: &dyn WindowManagerBackend,
        clipboard: &dyn ClipboardProvider,
        cancel_flag: &AtomicBool,
    ) -> Result<(), MacroError> {
        let initial_fg = if action.require_initial_foreground_match {
            match window_manager.capture_foreground_window(0) {
                Ok(fg) => Some(fg),
                Err(err) => return Err(MacroError::Platform(err.to_string())),
            }
        } else {
            None
        };

        for step in &action.steps {
            if cancel_flag.load(Ordering::SeqCst) {
                emergency_release_modifiers(input);
                return Err(MacroError::Cancelled);
            }

            // Verify focus guard before input steps
            if let Some(ref init) = initial_fg {
                if Self::is_input_step(step) {
                    let current = window_manager
                        .capture_foreground_window(0)
                        .map_err(|e| MacroError::Platform(e.to_string()))?;

                    if current.hwnd_raw != init.hwnd_raw {
                        emergency_release_modifiers(input);
                        return Err(MacroError::FocusLost {
                            expected_exe: init.exe_name.clone(),
                            expected_hwnd: init.hwnd_raw,
                            actual_exe: current.exe_name,
                            actual_hwnd: current.hwnd_raw,
                        });
                    }
                }
            }

            Self::execute_step(step, input, window_manager, clipboard, cancel_flag).await?;
        }

        Ok(())
    }

    fn is_input_step(step: &MacroStep) -> bool {
        matches!(
            step,
            MacroStep::KeyTap { .. }
                | MacroStep::KeyChord { .. }
                | MacroStep::TypeText { .. }
                | MacroStep::ClipboardPaste { .. }
        )
    }

    async fn execute_step(
        step: &MacroStep,
        input: &dyn InputInjectorBackend,
        window_manager: &dyn WindowManagerBackend,
        clipboard: &dyn ClipboardProvider,
        cancel_flag: &AtomicBool,
    ) -> Result<(), MacroError> {
        match step {
            MacroStep::KeyTap { key } => {
                input
                    .tap_key(key)
                    .map_err(|e| MacroError::Platform(e.to_string()))?;
            }
            MacroStep::KeyChord { modifiers, key } => {
                input
                    .chord_keys(modifiers, key)
                    .map_err(|e| MacroError::Platform(e.to_string()))?;
            }
            MacroStep::TypeText { text } => {
                input
                    .type_unicode_text(text)
                    .map_err(|e| MacroError::Platform(e.to_string()))?;
            }
            MacroStep::ClipboardPaste {
                text,
                restore_previous_clipboard,
            } => {
                let previous = if *restore_previous_clipboard {
                    clipboard.get_text().ok().flatten()
                } else {
                    None
                };

                clipboard.set_text(text)?;

                // Trigger Ctrl+V
                input
                    .chord_keys(&["ctrl".to_string()], "v")
                    .map_err(|e| MacroError::Platform(e.to_string()))?;

                // Delay for paste consumption
                tokio::time::sleep(Duration::from_millis(50)).await;

                if let Some(ref prev_text) = previous {
                    clipboard.set_text(prev_text)?;
                }
            }
            MacroStep::DelayMs(ms) => {
                let chunks = (*ms / 20).max(1);
                let per_chunk = Duration::from_millis(ms / chunks);
                for _ in 0..chunks {
                    if cancel_flag.load(Ordering::SeqCst) {
                        return Err(MacroError::Cancelled);
                    }
                    tokio::time::sleep(per_chunk).await;
                }
            }
            MacroStep::WaitForWindow {
                selector,
                timeout_ms,
            } => {
                let start = std::time::Instant::now();
                let timeout = Duration::from_millis(*timeout_ms);
                let poll_interval = Duration::from_millis(50);

                loop {
                    if cancel_flag.load(Ordering::SeqCst) {
                        return Err(MacroError::Cancelled);
                    }

                    if let Ok(windows) = window_manager.list_visible_windows(0) {
                        let found = windows.iter().any(|w| Self::matches_selector(selector, w));
                        if found {
                            return Ok(());
                        }
                    }

                    if start.elapsed() >= timeout {
                        return Err(MacroError::WindowWaitTimeout);
                    }

                    tokio::time::sleep(poll_interval).await;
                }
            }
        }

        Ok(())
    }

    fn matches_selector(selector: &WindowSelector, info: &ForegroundAppInfo) -> bool {
        if let Some(ref exe) = selector.exe_name {
            if !info.exe_name.eq_ignore_ascii_case(exe) {
                return false;
            }
        }
        if let Some(ref title) = selector.title_contains {
            if !info
                .window_title
                .to_lowercase()
                .contains(&title.to_lowercase())
            {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ch57x_domain::actions::{
        FocusOutcome, PlacementOutcome, WindowActionResult, WindowPlacement,
    };
    use ch57x_domain::errors::PlatformError;
    use std::path::{Path, PathBuf};

    #[derive(Default)]
    struct MockInput {
        taps: Mutex<Vec<String>>,
        chords: Mutex<Vec<(Vec<String>, String)>>,
        typed: Mutex<Vec<String>>,
    }

    impl InputInjectorBackend for MockInput {
        fn can_inject_into_foreground(&self) -> bool {
            true
        }

        fn tap_key(&self, key_token: &str) -> Result<(), PlatformError> {
            self.taps.lock().unwrap().push(key_token.to_string());
            Ok(())
        }

        fn chord_keys(&self, modifiers: &[String], key_token: &str) -> Result<(), PlatformError> {
            self.chords
                .lock()
                .unwrap()
                .push((modifiers.to_vec(), key_token.to_string()));
            Ok(())
        }

        fn type_unicode_text(&self, text: &str) -> Result<(), PlatformError> {
            self.typed.lock().unwrap().push(text.to_string());
            Ok(())
        }
    }

    struct MockWm {
        fg: Mutex<ForegroundAppInfo>,
        fg_queue: Mutex<VecDeque<ForegroundAppInfo>>,
        visible: Mutex<Vec<ForegroundAppInfo>>,
    }

    impl MockWm {
        fn new(initial_fg: ForegroundAppInfo) -> Self {
            Self {
                fg: Mutex::new(initial_fg),
                fg_queue: Mutex::new(VecDeque::new()),
                visible: Mutex::new(vec![]),
            }
        }

        fn with_sequence(initial_fg: ForegroundAppInfo, subsequent: Vec<ForegroundAppInfo>) -> Self {
            Self {
                fg: Mutex::new(initial_fg),
                fg_queue: Mutex::new(subsequent.into()),
                visible: Mutex::new(vec![]),
            }
        }
    }

    impl WindowManagerBackend for MockWm {
        fn capture_foreground_window(
            &self,
            _exclude_pid: u32,
        ) -> Result<ForegroundAppInfo, PlatformError> {
            let mut queue = self.fg_queue.lock().unwrap();
            if let Some(next) = queue.pop_front() {
                let mut fg = self.fg.lock().unwrap();
                *fg = next.clone();
                Ok(next)
            } else {
                Ok(self.fg.lock().unwrap().clone())
            }
        }

        fn list_visible_windows(
            &self,
            _exclude_pid: u32,
        ) -> Result<Vec<ForegroundAppInfo>, PlatformError> {
            Ok(self.visible.lock().unwrap().clone())
        }

        fn launch_or_place_window(
            &self,
            _selector: &WindowSelector,
            _placement: &WindowPlacement,
            _launch_cmd: Option<(&Path, &[String], Option<&Path>)>,
            _timeout_ms: u64,
        ) -> Result<WindowActionResult, PlatformError> {
            Ok(WindowActionResult {
                placement: PlacementOutcome::Applied,
                focus: FocusOutcome::Granted,
                message: None,
            })
        }
    }

    fn make_app_info(hwnd: usize, exe: &str, title: &str) -> ForegroundAppInfo {
        ForegroundAppInfo {
            pid: 10,
            exe_name: exe.into(),
            exe_path: Some(PathBuf::from(exe)),
            window_title: title.into(),
            hwnd_raw: hwnd,
            x: 0,
            y: 0,
            width: 800,
            height: 600,
        }
    }

    #[tokio::test]
    async fn test_macro_steps_and_clipboard_backup_restore() {
        let input = Arc::new(MockInput::default());
        let wm = Arc::new(MockWm::new(make_app_info(1001, "notepad.exe", "Untitled")));
        let clipboard = Arc::new(InMemoryClipboard::with_content("original_clip"));

        let action = MacroSequenceAction {
            steps: vec![
                MacroStep::KeyTap {
                    key: "enter".into(),
                },
                MacroStep::KeyChord {
                    modifiers: vec!["ctrl".into(), "shift".into()],
                    key: "p".into(),
                },
                MacroStep::TypeText {
                    text: "hello world".into(),
                },
                MacroStep::ClipboardPaste {
                    text: "pasted_macro_text".into(),
                    restore_previous_clipboard: true,
                },
            ],
            concurrency: MacroConcurrencyPolicy::IgnoreIfRunning,
            require_initial_foreground_match: true,
        };

        let cancel_flag = AtomicBool::new(false);
        let res = MacroActor::execute_sequence(
            &action,
            input.as_ref(),
            wm.as_ref(),
            clipboard.as_ref(),
            &cancel_flag,
        )
        .await;

        assert!(res.is_ok());
        assert_eq!(input.taps.lock().unwrap().as_slice(), &["enter"]);
        assert_eq!(
            input.chords.lock().unwrap().as_slice(),
            &[
                (vec!["ctrl".into(), "shift".into()], "p".into()),
                (vec!["ctrl".into()], "v".into())
            ]
        );
        assert_eq!(input.typed.lock().unwrap().as_slice(), &["hello world"]);
        // Verify clipboard was restored to original value
        assert_eq!(
            clipboard.get_text().unwrap(),
            Some("original_clip".to_string())
        );
    }

    #[tokio::test]
    async fn test_focus_guard_aborts_on_window_change() {
        let input = Arc::new(MockInput::default());
        let wm = Arc::new(MockWm::with_sequence(
            make_app_info(1001, "notepad.exe", "Untitled"),
            vec![
                make_app_info(1001, "notepad.exe", "Untitled"),
                make_app_info(2002, "malicious.exe", "Phishing Window"),
            ],
        ));
        let clipboard = Arc::new(InMemoryClipboard::new());

        let action = MacroSequenceAction {
            steps: vec![
                MacroStep::KeyTap { key: "a".into() },
                // Between steps, focus will change
                MacroStep::KeyTap { key: "b".into() },
            ],
            concurrency: MacroConcurrencyPolicy::IgnoreIfRunning,
            require_initial_foreground_match: true,
        };

        let cancel_flag = AtomicBool::new(false);

        let res = MacroActor::execute_sequence(
            &action,
            input.as_ref(),
            wm.as_ref(),
            clipboard.as_ref(),
            &cancel_flag,
        )
        .await;

        match res {
            Err(MacroError::FocusLost {
                expected_hwnd,
                actual_hwnd,
                ..
            }) => {
                assert_eq!(expected_hwnd, 1001);
                assert_eq!(actual_hwnd, 2002);
            }
            other => panic!("Expected FocusLost, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_macro_actor_concurrency_policies() {
        let input = Arc::new(MockInput::default());
        let wm = Arc::new(MockWm::new(make_app_info(1001, "notepad.exe", "Untitled")));
        let actor = MacroActor::with_clipboard(input, wm, Arc::new(InMemoryClipboard::new()));

        let action_ignore = MacroSequenceAction {
            steps: vec![MacroStep::DelayMs(200)],
            concurrency: MacroConcurrencyPolicy::IgnoreIfRunning,
            require_initial_foreground_match: false,
        };

        // First submit should start
        let out1 = actor.submit(action_ignore.clone());
        assert_eq!(out1, MacroRunOutcome::Started);

        // Second submit with IgnoreIfRunning should be ignored
        let out2 = actor.submit(action_ignore);
        assert_eq!(out2, MacroRunOutcome::IgnoredAlreadyRunning);

        // Submit with QueueBounded
        let action_queue = MacroSequenceAction {
            steps: vec![MacroStep::DelayMs(10)],
            concurrency: MacroConcurrencyPolicy::QueueBounded(1),
            require_initial_foreground_match: false,
        };
        let out_q1 = actor.submit(action_queue.clone());
        assert_eq!(out_q1, MacroRunOutcome::Queued);

        // Queue is now full (limit was 1)
        let out_q2 = actor.submit(action_queue);
        assert_eq!(out_q2, MacroRunOutcome::QueueFull);

        // Test cancel
        actor.cancel_current();
        assert!(!actor.is_running());
    }
}
