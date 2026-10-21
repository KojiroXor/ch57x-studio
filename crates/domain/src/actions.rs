use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActionId(pub Uuid);

impl ActionId {
    pub fn new_v4() -> Self {
        Self(Uuid::new_v4())
    }
}

impl std::fmt::Display for ActionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotBinding {
    Disabled,
    HidNative(HidAction),
    HostActionRef(ActionId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HidMod {
    Ctrl,
    Shift,
    Alt,
    Win,
    RCtrl,
    RShift,
    RAlt,
    RWin,
}

impl HidMod {
    pub fn to_token(&self) -> &'static str {
        match self {
            HidMod::Ctrl => "ctrl",
            HidMod::Shift => "shift",
            HidMod::Alt => "alt",
            HidMod::Win => "win",
            HidMod::RCtrl => "rctrl",
            HidMod::RShift => "rshift",
            HidMod::RAlt => "ralt",
            HidMod::RWin => "rwin",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaToken {
    VolumeUp,
    VolumeDown,
    Mute,
    PlayPause,
    NextSong,
    PrevSong,
    Stop,
    BrightnessUp,
    BrightnessDown,
}

impl MediaToken {
    pub fn yaml_token(&self) -> &'static str {
        match self {
            MediaToken::VolumeUp => "volumeup",
            MediaToken::VolumeDown => "volumedown",
            MediaToken::Mute => "mute",
            MediaToken::PlayPause => "play",
            MediaToken::NextSong => "next",
            MediaToken::PrevSong => "prev",
            MediaToken::Stop => "stop",
            MediaToken::BrightnessUp => "macbrightnessup",
            MediaToken::BrightnessDown => "macbrightnessdown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseToken {
    WheelUp,
    WheelDown,
    Click,
    RightClick,
    MiddleClick,
}

impl MouseToken {
    pub fn yaml_token(&self) -> &'static str {
        match self {
            MouseToken::WheelUp => "wheel(1)",
            MouseToken::WheelDown => "wheel(-1)",
            MouseToken::Click => "click",
            MouseToken::RightClick => "click(right)",
            MouseToken::MiddleClick => "click(middle)",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HidAction {
    KeyboardChord { modifiers: Vec<HidMod>, key: String },
    ConsumerMedia(MediaToken),
    MouseAction(MouseToken),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowSelector {
    pub exe_name: Option<String>,
    pub title_contains: Option<String>,
    pub class_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WindowCoordinates {
    Absolute {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    },
    Normalized {
        x: f32, // 0.0 .. 1.0
        y: f32,
        width: f32,
        height: f32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowPlacement {
    pub monitor_index: Option<usize>,
    pub coordinates: WindowCoordinates,
    pub bring_to_foreground: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LaunchRecipe {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub working_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LaunchOrFocusAction {
    pub selector: WindowSelector,
    pub placement: Option<WindowPlacement>,
    pub launch: Option<LaunchRecipe>,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceTarget {
    pub selector: WindowSelector,
    pub placement: WindowPlacement,
    pub launch: Option<LaunchRecipe>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RestoreWorkspaceAction {
    pub targets: Vec<WorkspaceTarget>,
    pub post_delay_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MacroConcurrencyPolicy {
    IgnoreIfRunning,
    RestartCurrent,
    QueueBounded(usize),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MacroStep {
    KeyTap {
        key: String,
    },
    KeyChord {
        modifiers: Vec<String>,
        key: String,
    },
    TypeText {
        text: String,
    },
    ClipboardPaste {
        text: String,
        restore_previous_clipboard: bool,
    },
    DelayMs(u64),
    WaitForWindow {
        selector: WindowSelector,
        timeout_ms: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacroSequenceAction {
    pub steps: Vec<MacroStep>,
    pub concurrency: MacroConcurrencyPolicy,
    pub require_initial_foreground_match: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecProcessAction {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub working_dir: Option<PathBuf>,
    pub timeout_ms: u64,
    pub hide_console_window: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextRule {
    pub match_exe_name: Option<String>,
    pub match_title_contains: Option<String>,
    pub target_action_id: ActionId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextDispatchAction {
    pub rules: Vec<ContextRule>,
    pub fallback_action_id: ActionId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActionDefinition {
    LaunchOrFocus(LaunchOrFocusAction),
    RestoreWorkspace(RestoreWorkspaceAction),
    MacroSequence(MacroSequenceAction),
    ExecProcess(ExecProcessAction),
    ContextDispatch(ContextDispatchAction),
    ShowHud { duration_ms: u64 },
    AdjustBrightness { delta: i32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForegroundAppInfo {
    pub pid: u32,
    pub exe_name: String,
    pub exe_path: Option<PathBuf>,
    pub window_title: String,
    pub hwnd_raw: usize,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlacementOutcome {
    Applied,
    AlreadyInPosition,
    WindowNotFound,
    ProcessLaunchedAwaitingWindow,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FocusOutcome {
    Granted,
    DeniedByOS,
    NotRequested,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowActionResult {
    pub placement: PlacementOutcome,
    pub focus: FocusOutcome,
    pub message: Option<String>,
}
