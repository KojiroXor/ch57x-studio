//! Window actor: Executes LaunchOrFocusAction with coordinates calculation, retries and timeouts

use ch57x_domain::actions::{
    LaunchOrFocusAction, PlacementOutcome, WindowActionResult, WindowCoordinates, WindowPlacement,
};
use ch57x_domain::errors::PlatformError;
use ch57x_platform::traits::WindowManagerBackend;
use std::sync::Arc;
use std::time::Duration;

/// Configuration for WindowActor retry and timeout behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowActorConfig {
    pub max_retries: u32,
    pub retry_interval_ms: u64,
    pub default_timeout_ms: u64,
}

impl Default for WindowActorConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_interval_ms: 50,
            default_timeout_ms: 3000,
        }
    }
}

/// Computes final pixel bounds (x, y, width, height) given WindowCoordinates and work area dimensions.
pub fn calculate_window_rect(
    coordinates: &WindowCoordinates,
    work_area_x: i32,
    work_area_y: i32,
    work_area_w: i32,
    work_area_h: i32,
) -> (i32, i32, i32, i32) {
    match coordinates {
        WindowCoordinates::Absolute {
            x,
            y,
            width,
            height,
        } => (*x, *y, *width, *height),
        WindowCoordinates::Normalized {
            x,
            y,
            width,
            height,
        } => {
            let target_x = work_area_x + (*x * work_area_w as f32).round() as i32;
            let target_y = work_area_y + (*y * work_area_h as f32).round() as i32;
            let target_w = (*width * work_area_w as f32).round() as i32;
            let target_h = (*height * work_area_h as f32).round() as i32;
            (target_x, target_y, target_w, target_h)
        }
    }
}

/// Actor that coordinates window launching, positioning, and focusing.
pub struct WindowActor {
    window_manager: Arc<dyn WindowManagerBackend>,
    config: WindowActorConfig,
}

impl WindowActor {
    pub fn new(window_manager: Arc<dyn WindowManagerBackend>) -> Self {
        Self {
            window_manager,
            config: WindowActorConfig::default(),
        }
    }

    pub fn with_config(
        window_manager: Arc<dyn WindowManagerBackend>,
        config: WindowActorConfig,
    ) -> Self {
        Self {
            window_manager,
            config,
        }
    }

    pub fn config(&self) -> &WindowActorConfig {
        &self.config
    }

    /// Executes a LaunchOrFocusAction with retry logic and returns a WindowActionResult.
    pub fn execute(
        &self,
        action: &LaunchOrFocusAction,
    ) -> Result<WindowActionResult, PlatformError> {
        let timeout = if action.timeout_ms > 0 {
            action.timeout_ms
        } else {
            self.config.default_timeout_ms
        };

        let fallback_placement = WindowPlacement {
            monitor_index: None,
            coordinates: WindowCoordinates::Absolute {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
            bring_to_foreground: true,
        };
        let placement = action.placement.as_ref().unwrap_or(&fallback_placement);

        let launch_cmd = action.launch.as_ref().map(|recipe| {
            (
                recipe.program.as_path(),
                recipe.args.as_slice(),
                recipe.working_dir.as_deref(),
            )
        });

        let mut retries_left = self.config.max_retries;
        loop {
            let res = self.window_manager.launch_or_place_window(
                &action.selector,
                placement,
                launch_cmd,
                timeout,
            );

            match res {
                Ok(outcome) => {
                    if outcome.placement == PlacementOutcome::Applied
                        || outcome.placement == PlacementOutcome::AlreadyInPosition
                    {
                        return Ok(outcome);
                    }

                    if retries_left > 0
                        && (outcome.placement == PlacementOutcome::ProcessLaunchedAwaitingWindow
                            || outcome.placement == PlacementOutcome::WindowNotFound)
                    {
                        retries_left -= 1;
                        std::thread::sleep(Duration::from_millis(self.config.retry_interval_ms));
                        continue;
                    }

                    return Ok(outcome);
                }
                Err(err) => {
                    if retries_left > 0 {
                        retries_left -= 1;
                        std::thread::sleep(Duration::from_millis(self.config.retry_interval_ms));
                        continue;
                    }
                    return Err(err);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ch57x_domain::actions::{FocusOutcome, ForegroundAppInfo, LaunchRecipe, WindowSelector};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct MockWindowManager {
        call_count: AtomicUsize,
        result_sequence: Vec<WindowActionResult>,
    }

    impl WindowManagerBackend for MockWindowManager {
        fn capture_foreground_window(
            &self,
            _exclude_pid: u32,
        ) -> Result<ForegroundAppInfo, PlatformError> {
            Ok(ForegroundAppInfo {
                pid: 100,
                exe_name: "test.exe".into(),
                exe_path: Some(PathBuf::from("C:\\test.exe")),
                window_title: "Test Window".into(),
                hwnd_raw: 1234,
                x: 10,
                y: 10,
                width: 800,
                height: 600,
            })
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
            let idx = self.call_count.fetch_add(1, Ordering::SeqCst);
            if idx < self.result_sequence.len() {
                Ok(self.result_sequence[idx].clone())
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
    fn test_coordinate_calculations() {
        // Absolute
        let abs = WindowCoordinates::Absolute {
            x: 150,
            y: 200,
            width: 1024,
            height: 768,
        };
        let rect = calculate_window_rect(&abs, 0, 0, 1920, 1080);
        assert_eq!(rect, (150, 200, 1024, 768));

        // Normalized left half of monitor
        let norm_left = WindowCoordinates::Normalized {
            x: 0.0,
            y: 0.0,
            width: 0.5,
            height: 1.0,
        };
        let rect_left = calculate_window_rect(&norm_left, 100, 50, 1920, 1080);
        assert_eq!(rect_left, (100, 50, 960, 1080));

        // Normalized right half of monitor
        let norm_right = WindowCoordinates::Normalized {
            x: 0.5,
            y: 0.0,
            width: 0.5,
            height: 1.0,
        };
        let rect_right = calculate_window_rect(&norm_right, 0, 0, 2560, 1440);
        assert_eq!(rect_right, (1280, 0, 1280, 1440));
    }

    #[test]
    fn test_window_actor_retries_and_succeeds() {
        let mock = Arc::new(MockWindowManager {
            call_count: AtomicUsize::new(0),
            result_sequence: vec![
                WindowActionResult {
                    placement: PlacementOutcome::ProcessLaunchedAwaitingWindow,
                    focus: FocusOutcome::NotRequested,
                    message: Some("Waiting for process window".into()),
                },
                WindowActionResult {
                    placement: PlacementOutcome::Applied,
                    focus: FocusOutcome::Granted,
                    message: None,
                },
            ],
        });

        let config = WindowActorConfig {
            max_retries: 2,
            retry_interval_ms: 10,
            default_timeout_ms: 500,
        };
        let actor = WindowActor::with_config(mock.clone(), config);

        let action = LaunchOrFocusAction {
            selector: WindowSelector {
                exe_name: Some("test.exe".into()),
                title_contains: None,
                class_name: None,
            },
            placement: Some(WindowPlacement {
                monitor_index: None,
                coordinates: WindowCoordinates::Normalized {
                    x: 0.0,
                    y: 0.0,
                    width: 0.5,
                    height: 1.0,
                },
                bring_to_foreground: true,
            }),
            launch: Some(LaunchRecipe {
                program: PathBuf::from("test.exe"),
                args: vec![],
                working_dir: None,
            }),
            timeout_ms: 1000,
        };

        let result = actor.execute(&action).expect("Should succeed");
        assert_eq!(result.placement, PlacementOutcome::Applied);
        assert_eq!(result.focus, FocusOutcome::Granted);
        assert_eq!(mock.call_count.load(Ordering::SeqCst), 2);
    }
}
