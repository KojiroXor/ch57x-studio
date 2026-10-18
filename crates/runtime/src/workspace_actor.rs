//! Workspace actor: Executes RestoreWorkspaceAction sequentially across multiple targets

use crate::window_actor::WindowActor;
use ch57x_domain::actions::{
    LaunchOrFocusAction, PlacementOutcome, RestoreWorkspaceAction, WindowActionResult,
    WindowSelector,
};
use ch57x_domain::errors::PlatformError;
use ch57x_platform::traits::WindowManagerBackend;
use std::sync::Arc;
use std::time::Duration;

/// Individual outcome for a workspace target window placement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceTargetOutcome {
    pub target_index: usize,
    pub selector: WindowSelector,
    pub result: WindowActionResult,
}

/// Comprehensive report detailing the execution of a workspace restore action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceRestoreReport {
    pub total_targets: usize,
    pub successful_placements: usize,
    pub outcomes: Vec<WorkspaceTargetOutcome>,
    pub all_succeeded: bool,
}

/// Actor responsible for restoring full desktop workspaces.
pub struct WorkspaceActor {
    window_actor: Arc<WindowActor>,
}

impl WorkspaceActor {
    pub fn new(window_manager: Arc<dyn WindowManagerBackend>) -> Self {
        Self {
            window_actor: Arc::new(WindowActor::new(window_manager)),
        }
    }

    pub fn with_window_actor(window_actor: Arc<WindowActor>) -> Self {
        Self { window_actor }
    }

    /// Executes the workspace restore sequentially across all configured targets.
    pub fn execute(
        &self,
        action: &RestoreWorkspaceAction,
    ) -> Result<WorkspaceRestoreReport, PlatformError> {
        let total_targets = action.targets.len();
        let mut outcomes = Vec::with_capacity(total_targets);
        let mut successful_placements = 0;

        for (idx, target) in action.targets.iter().enumerate() {
            let lof_action = LaunchOrFocusAction {
                selector: target.selector.clone(),
                placement: Some(target.placement.clone()),
                launch: target.launch.clone(),
                timeout_ms: self.window_actor.config().default_timeout_ms,
            };

            let res = self.window_actor.execute(&lof_action)?;

            let is_success = res.placement == PlacementOutcome::Applied
                || res.placement == PlacementOutcome::AlreadyInPosition;

            if is_success {
                successful_placements += 1;
            }

            outcomes.push(WorkspaceTargetOutcome {
                target_index: idx,
                selector: target.selector.clone(),
                result: res,
            });

            // Sequential delay between targets
            if action.post_delay_ms > 0 && idx + 1 < total_targets {
                std::thread::sleep(Duration::from_millis(action.post_delay_ms));
            }
        }

        let all_succeeded = successful_placements == total_targets;

        Ok(WorkspaceRestoreReport {
            total_targets,
            successful_placements,
            outcomes,
            all_succeeded,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ch57x_domain::actions::{
        FocusOutcome, ForegroundAppInfo, WindowCoordinates, WindowPlacement, WorkspaceTarget,
    };
    use std::path::Path;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct MultiTargetMockWindowManager {
        counter: AtomicUsize,
    }

    impl WindowManagerBackend for MultiTargetMockWindowManager {
        fn capture_foreground_window(
            &self,
            _exclude_pid: u32,
        ) -> Result<ForegroundAppInfo, PlatformError> {
            unimplemented!()
        }

        fn list_visible_windows(
            &self,
            _exclude_pid: u32,
        ) -> Result<Vec<ForegroundAppInfo>, PlatformError> {
            Ok(vec![])
        }

        fn launch_or_place_window(
            &self,
            selector: &WindowSelector,
            _placement: &WindowPlacement,
            _launch_cmd: Option<(&Path, &[String], Option<&Path>)>,
            _timeout_ms: u64,
        ) -> Result<WindowActionResult, PlatformError> {
            self.counter.fetch_add(1, Ordering::SeqCst);
            if selector.exe_name.as_deref() == Some("fail.exe") {
                Ok(WindowActionResult {
                    placement: PlacementOutcome::Failed,
                    focus: FocusOutcome::DeniedByOS,
                    message: Some("Placement failed".into()),
                })
            } else {
                Ok(WindowActionResult {
                    placement: PlacementOutcome::Applied,
                    focus: FocusOutcome::Granted,
                    message: None,
                })
            }
        }
    }

    #[test]
    fn test_workspace_restore_report_partial_and_full() {
        let mock = Arc::new(MultiTargetMockWindowManager {
            counter: AtomicUsize::new(0),
        });
        let actor = WorkspaceActor::new(mock.clone());

        let target_ok = WorkspaceTarget {
            selector: WindowSelector {
                exe_name: Some("browser.exe".into()),
                title_contains: None,
                class_name: None,
            },
            placement: WindowPlacement {
                monitor_index: Some(0),
                coordinates: WindowCoordinates::Normalized {
                    x: 0.0,
                    y: 0.0,
                    width: 0.5,
                    height: 1.0,
                },
                bring_to_foreground: true,
            },
            launch: None,
        };

        let target_fail = WorkspaceTarget {
            selector: WindowSelector {
                exe_name: Some("fail.exe".into()),
                title_contains: None,
                class_name: None,
            },
            placement: WindowPlacement {
                monitor_index: Some(0),
                coordinates: WindowCoordinates::Normalized {
                    x: 0.5,
                    y: 0.0,
                    width: 0.5,
                    height: 1.0,
                },
                bring_to_foreground: false,
            },
            launch: None,
        };

        let action = RestoreWorkspaceAction {
            targets: vec![target_ok.clone(), target_fail],
            post_delay_ms: 10,
        };

        let report = actor.execute(&action).expect("Should execute workspace");
        assert_eq!(report.total_targets, 2);
        assert_eq!(report.successful_placements, 1);
        assert!(!report.all_succeeded);
        assert_eq!(report.outcomes.len(), 2);
        assert_eq!(report.outcomes[0].result.placement, PlacementOutcome::Applied);
        assert_eq!(report.outcomes[1].result.placement, PlacementOutcome::Failed);

        // Test all succeeded
        let action_all_ok = RestoreWorkspaceAction {
            targets: vec![target_ok.clone(), target_ok],
            post_delay_ms: 5,
        };

        let report_ok = actor.execute(&action_all_ok).expect("Should execute workspace");
        assert_eq!(report_ok.total_targets, 2);
        assert_eq!(report_ok.successful_placements, 2);
        assert!(report_ok.all_succeeded);
    }
}
