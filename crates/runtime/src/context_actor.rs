//! Context actor: Evaluates active foreground window against ContextRule priorities

use ch57x_domain::actions::{ActionId, ContextDispatchAction, ContextRule, ForegroundAppInfo};
use ch57x_domain::errors::PlatformError;
use ch57x_platform::traits::WindowManagerBackend;
use std::sync::Arc;

/// Actor that resolves context-sensitive actions based on the active foreground window.
pub struct ContextActor {
    window_manager: Arc<dyn WindowManagerBackend>,
}

impl ContextActor {
    pub fn new(window_manager: Arc<dyn WindowManagerBackend>) -> Self {
        Self { window_manager }
    }

    /// Evaluates the current foreground window against the rules in `action`.
    /// Returns the matched `target_action_id` or `fallback_action_id`.
    pub fn evaluate(
        &self,
        action: &ContextDispatchAction,
        exclude_pid: u32,
    ) -> Result<ActionId, PlatformError> {
        let fg_info = match self.window_manager.capture_foreground_window(exclude_pid) {
            Ok(fg) => fg,
            Err(err) => {
                tracing::warn!(
                    "Failed to capture foreground window for context dispatch: {err}, using fallback"
                );
                return Ok(action.fallback_action_id);
            }
        };

        Ok(Self::evaluate_against_foreground(action, &fg_info))
    }

    /// Pure evaluation of rules against provided `ForegroundAppInfo`.
    pub fn evaluate_against_foreground(
        action: &ContextDispatchAction,
        fg_info: &ForegroundAppInfo,
    ) -> ActionId {
        for rule in &action.rules {
            if Self::matches_rule(rule, fg_info) {
                return rule.target_action_id;
            }
        }
        action.fallback_action_id
    }

    /// Checks if a single rule matches the foreground window metadata.
    pub fn matches_rule(rule: &ContextRule, fg_info: &ForegroundAppInfo) -> bool {
        // Exe name matching
        if let Some(ref expected_exe) = rule.match_exe_name {
            let exp_clean = expected_exe.strip_suffix(".exe").unwrap_or(expected_exe);
            let actual_clean = fg_info
                .exe_name
                .strip_suffix(".exe")
                .unwrap_or(&fg_info.exe_name);
            if !exp_clean.eq_ignore_ascii_case(actual_clean) {
                return false;
            }
        }

        // Window title matching
        if let Some(ref title_substr) = rule.match_title_contains {
            if !fg_info
                .window_title
                .to_lowercase()
                .contains(&title_substr.to_lowercase())
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
    use ch57x_domain::actions::{WindowActionResult, WindowPlacement, WindowSelector};
    use std::path::{Path, PathBuf};

    struct ContextMockWindowManager {
        fg: Result<ForegroundAppInfo, PlatformError>,
    }

    impl WindowManagerBackend for ContextMockWindowManager {
        fn capture_foreground_window(
            &self,
            _exclude_pid: u32,
        ) -> Result<ForegroundAppInfo, PlatformError> {
            self.fg.clone()
        }

        fn list_visible_windows(
            &self,
            _exclude_pid: u32,
        ) -> Result<Vec<ForegroundAppInfo>, PlatformError> {
            Ok(vec![])
        }

        fn launch_or_place_window(
            &self,
            _selector: &WindowSelector,
            _placement: &WindowPlacement,
            _launch_cmd: Option<(&Path, &[String], Option<&Path>)>,
            _timeout_ms: u64,
        ) -> Result<WindowActionResult, PlatformError> {
            unimplemented!()
        }
    }

    fn make_fg(exe: &str, title: &str) -> ForegroundAppInfo {
        ForegroundAppInfo {
            pid: 1234,
            exe_name: exe.into(),
            exe_path: Some(PathBuf::from(exe)),
            window_title: title.into(),
            hwnd_raw: 9999,
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        }
    }

    #[test]
    fn test_context_rule_matching_priority_and_fallback() {
        let action_id_code = ActionId::new_v4();
        let action_id_browser_yt = ActionId::new_v4();
        let action_id_browser_gen = ActionId::new_v4();
        let action_id_fallback = ActionId::new_v4();

        let dispatch_action = ContextDispatchAction {
            rules: vec![
                // Rule 1: Specific Chrome YouTube title
                ContextRule {
                    match_exe_name: Some("chrome.exe".into()),
                    match_title_contains: Some("YouTube".into()),
                    target_action_id: action_id_browser_yt,
                },
                // Rule 2: Generic Chrome browser
                ContextRule {
                    match_exe_name: Some("chrome".into()),
                    match_title_contains: None,
                    target_action_id: action_id_browser_gen,
                },
                // Rule 3: VS Code
                ContextRule {
                    match_exe_name: Some("code.exe".into()),
                    match_title_contains: None,
                    target_action_id: action_id_code,
                },
            ],
            fallback_action_id: action_id_fallback,
        };

        // Test 1: Chrome with YouTube in title -> matches Rule 1
        let fg_yt = make_fg(
            "chrome.exe",
            "Rick Astley - Never Gonna Give You Up - YouTube",
        );
        let res_yt = ContextActor::evaluate_against_foreground(&dispatch_action, &fg_yt);
        assert_eq!(res_yt, action_id_browser_yt);

        // Test 2: Chrome with GitHub in title -> matches Rule 2 (generic Chrome)
        let fg_gh = make_fg("chrome.exe", "GitHub - Dashboard");
        let res_gh = ContextActor::evaluate_against_foreground(&dispatch_action, &fg_gh);
        assert_eq!(res_gh, action_id_browser_gen);

        // Test 3: VS Code without .exe in lookup -> matches Rule 3
        let fg_code = make_fg("Code.exe", "dispatcher.rs - Tastiera bl");
        let res_code = ContextActor::evaluate_against_foreground(&dispatch_action, &fg_code);
        assert_eq!(res_code, action_id_code);

        // Test 4: Unmatched app (e.g. notepad) -> returns fallback
        let fg_notepad = make_fg("notepad.exe", "Untitled - Notepad");
        let res_fallback = ContextActor::evaluate_against_foreground(&dispatch_action, &fg_notepad);
        assert_eq!(res_fallback, action_id_fallback);
    }

    #[test]
    fn test_context_actor_window_manager_error_fallback() {
        let fallback_id = ActionId::new_v4();
        let mock = Arc::new(ContextMockWindowManager {
            fg: Err(PlatformError::SystemApi("Foreground unavailable".into())),
        });
        let actor = ContextActor::new(mock);

        let dispatch = ContextDispatchAction {
            rules: vec![],
            fallback_action_id: fallback_id,
        };

        let result = actor
            .evaluate(&dispatch, 0)
            .expect("Should fallback gracefully");
        assert_eq!(result, fallback_id);
    }
}
