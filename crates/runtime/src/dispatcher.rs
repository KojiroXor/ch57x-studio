//! Runtime dispatcher: Central engine mapping PhantomTriggers to Profile actions and delegating to actors

use crate::context_actor::ContextActor;
use crate::macro_actor::{MacroActor, MacroRunOutcome};
use crate::window_actor::WindowActor;
use crate::workspace_actor::{WorkspaceActor, WorkspaceRestoreReport};
use ch57x_domain::actions::{ActionDefinition, ActionId, HidAction, SlotBinding, WindowActionResult};
use ch57x_domain::profile::Profile;
use ch57x_domain::triggers::PhantomTrigger;
use ch57x_platform::traits::{InputInjectorBackend, WindowManagerBackend};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

/// Result of evaluating and delegating a trigger or action in the runtime engine.
#[derive(Debug, Clone, PartialEq)]
pub enum DispatchOutcome {
    Paused,
    UnmappedTrigger(PhantomTrigger),
    SlotDisabled(usize),
    HidNative(HidAction),
    ActionNotFound(ActionId),
    Window(WindowActionResult),
    Workspace(WorkspaceRestoreReport),
    Macro(MacroRunOutcome),
    ExecProcess {
        pid: u32,
    },
    ContextDispatched {
        resolved_action_id: ActionId,
        outcome: Box<DispatchOutcome>,
    },
    ShowHud {
        duration_ms: u64,
    },
    RecursionLimitExceeded,
    Error(String),
}

/// Central orchestrator that handles incoming hardware triggers and routes them to appropriate actors.
pub struct RuntimeDispatcher {
    profile: Arc<RwLock<Profile>>,
    window_actor: Arc<WindowActor>,
    workspace_actor: Arc<WorkspaceActor>,
    macro_actor: Arc<MacroActor>,
    context_actor: Arc<ContextActor>,
    is_paused: Arc<AtomicBool>,
}

impl RuntimeDispatcher {
    pub fn new(
        profile: Profile,
        window_manager: Arc<dyn WindowManagerBackend>,
        input: Arc<dyn InputInjectorBackend>,
    ) -> Self {
        let win_actor = Arc::new(WindowActor::new(window_manager.clone()));
        let ws_actor = Arc::new(WorkspaceActor::with_window_actor(win_actor.clone()));
        let mac_actor = Arc::new(MacroActor::new(input, window_manager.clone()));
        let ctx_actor = Arc::new(ContextActor::new(window_manager));

        Self {
            profile: Arc::new(RwLock::new(profile)),
            window_actor: win_actor,
            workspace_actor: ws_actor,
            macro_actor: mac_actor,
            context_actor: ctx_actor,
            is_paused: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn with_actors(
        profile: Profile,
        window_actor: Arc<WindowActor>,
        workspace_actor: Arc<WorkspaceActor>,
        macro_actor: Arc<MacroActor>,
        context_actor: Arc<ContextActor>,
    ) -> Self {
        Self {
            profile: Arc::new(RwLock::new(profile)),
            window_actor,
            workspace_actor,
            macro_actor,
            context_actor,
            is_paused: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Sets the emergency pause state. If pausing, any currently running macro is cancelled.
    pub fn set_paused(&self, paused: bool) {
        self.is_paused.store(paused, Ordering::SeqCst);
        if paused {
            self.macro_actor.cancel_current();
        }
    }

    /// Returns whether the engine is currently paused.
    pub fn is_paused(&self) -> bool {
        self.is_paused.load(Ordering::SeqCst)
    }

    /// Toggles the emergency pause state and returns the new value.
    pub fn toggle_pause(&self) -> bool {
        let current = self.is_paused();
        let next = !current;
        self.set_paused(next);
        next
    }

    /// Updates the active profile.
    pub fn set_profile(&self, profile: Profile) {
        if let Ok(mut lock) = self.profile.write() {
            *lock = profile;
        }
    }

    /// Retrieves a clone of the active profile.
    pub fn active_profile(&self) -> Profile {
        self.profile.read().unwrap().clone()
    }

    /// Main entry point: receives a PhantomTrigger, maps it through the active profile,
    /// and delegates to the appropriate actor.
    pub fn trigger_received(&self, trigger: PhantomTrigger) -> DispatchOutcome {
        if self.is_paused() {
            return DispatchOutcome::Paused;
        }

        let profile = match self.profile.read() {
            Ok(p) => p,
            Err(_) => return DispatchOutcome::Error("Profile lock poisoned".into()),
        };

        // Find slot index mapped to this trigger
        let slot_idx = profile
            .trigger_allocation
            .iter()
            .find(|(_, &t)| t == trigger)
            .map(|(&idx, _)| idx);

        let slot_idx = match slot_idx {
            Some(idx) => idx,
            None => return DispatchOutcome::UnmappedTrigger(trigger),
        };

        let slot_config = match profile.slots.get(&slot_idx) {
            Some(cfg) => cfg,
            None => return DispatchOutcome::SlotDisabled(slot_idx),
        };

        match &slot_config.binding {
            SlotBinding::Disabled => DispatchOutcome::SlotDisabled(slot_idx),
            SlotBinding::HidNative(hid) => DispatchOutcome::HidNative(hid.clone()),
            SlotBinding::HostActionRef(action_id) => {
                self.dispatch_action_id_internal(*action_id, &profile, 0)
            }
        }
    }

    /// Directly dispatches an ActionId by looking it up in the active profile.
    pub fn dispatch_action_id(&self, action_id: ActionId) -> DispatchOutcome {
        if self.is_paused() {
            return DispatchOutcome::Paused;
        }

        let profile = match self.profile.read() {
            Ok(p) => p,
            Err(_) => return DispatchOutcome::Error("Profile lock poisoned".into()),
        };

        self.dispatch_action_id_internal(action_id, &profile, 0)
    }

    fn dispatch_action_id_internal(
        &self,
        action_id: ActionId,
        profile: &Profile,
        depth: usize,
    ) -> DispatchOutcome {
        if depth > 5 {
            return DispatchOutcome::RecursionLimitExceeded;
        }

        let action_def = match profile.actions.get(&action_id) {
            Some(def) => def,
            None => return DispatchOutcome::ActionNotFound(action_id),
        };

        match action_def {
            ActionDefinition::LaunchOrFocus(action) => {
                match self.window_actor.execute(action) {
                    Ok(res) => DispatchOutcome::Window(res),
                    Err(err) => DispatchOutcome::Error(err.to_string()),
                }
            }
            ActionDefinition::RestoreWorkspace(action) => {
                match self.workspace_actor.execute(action) {
                    Ok(report) => DispatchOutcome::Workspace(report),
                    Err(err) => DispatchOutcome::Error(err.to_string()),
                }
            }
            ActionDefinition::MacroSequence(action) => {
                let outcome = self.macro_actor.submit(action.clone());
                DispatchOutcome::Macro(outcome)
            }
            ActionDefinition::ExecProcess(action) => {
                let mut cmd = std::process::Command::new(&action.program);
                cmd.args(&action.args);
                if let Some(ref dir) = action.working_dir {
                    cmd.current_dir(dir);
                }

                #[cfg(target_os = "windows")]
                if action.hide_console_window {
                    use std::os::windows::process::CommandExt;
                    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
                }

                match cmd.spawn() {
                    Ok(child) => DispatchOutcome::ExecProcess { pid: child.id() },
                    Err(err) => DispatchOutcome::Error(format!("Process spawn error: {err}")),
                }
            }
            ActionDefinition::ContextDispatch(action) => {
                match self.context_actor.evaluate(action, 0) {
                    Ok(target_id) => {
                        let sub_outcome =
                            self.dispatch_action_id_internal(target_id, profile, depth + 1);
                        DispatchOutcome::ContextDispatched {
                            resolved_action_id: target_id,
                            outcome: Box::new(sub_outcome),
                        }
                    }
                    Err(err) => DispatchOutcome::Error(err.to_string()),
                }
            }
            ActionDefinition::ShowHud { duration_ms } => {
                DispatchOutcome::ShowHud {
                    duration_ms: *duration_ms,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ch57x_domain::actions::{
        ContextDispatchAction, ContextRule, FocusOutcome, ForegroundAppInfo,
        MacroConcurrencyPolicy, MacroSequenceAction, MacroStep, PlacementOutcome,
        RestoreWorkspaceAction, WindowCoordinates, WindowPlacement, WindowSelector,
        WorkspaceTarget,
    };
    use ch57x_domain::errors::PlatformError;
    use ch57x_domain::triggers::TriggerBank;
    use std::path::Path;
    use std::sync::Mutex;

    struct DispatcherMockWm {
        fg: Mutex<ForegroundAppInfo>,
    }

    impl WindowManagerBackend for DispatcherMockWm {
        fn capture_foreground_window(
            &self,
            _exclude_pid: u32,
        ) -> Result<ForegroundAppInfo, PlatformError> {
            Ok(self.fg.lock().unwrap().clone())
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
            Ok(WindowActionResult {
                placement: PlacementOutcome::Applied,
                focus: FocusOutcome::Granted,
                message: None,
            })
        }
    }

    #[derive(Default)]
    struct DispatcherMockInput;

    impl InputInjectorBackend for DispatcherMockInput {
        fn can_inject_into_foreground(&self) -> bool {
            true
        }

        fn tap_key(&self, _key_token: &str) -> Result<(), PlatformError> {
            Ok(())
        }

        fn chord_keys(&self, _modifiers: &[String], _key_token: &str) -> Result<(), PlatformError> {
            Ok(())
        }

        fn type_unicode_text(&self, _text: &str) -> Result<(), PlatformError> {
            Ok(())
        }
    }

    fn create_test_profile() -> (Profile, ActionId, ActionId, ActionId) {
        let mut profile = Profile::default();

        let action_win = ActionId::new_v4();
        let action_macro = ActionId::new_v4();
        let action_context = ActionId::new_v4();
        let action_hud = ActionId::new_v4();

        // 1. Window action
        profile.actions.insert(
            action_win,
            ActionDefinition::RestoreWorkspace(RestoreWorkspaceAction {
                targets: vec![WorkspaceTarget {
                    selector: WindowSelector {
                        exe_name: Some("test.exe".into()),
                        title_contains: None,
                        class_name: None,
                    },
                    placement: WindowPlacement {
                        monitor_index: None,
                        coordinates: WindowCoordinates::Absolute {
                            x: 0,
                            y: 0,
                            width: 100,
                            height: 100,
                        },
                        bring_to_foreground: true,
                    },
                    launch: None,
                }],
                post_delay_ms: 0,
            }),
        );

        // 2. Macro action
        profile.actions.insert(
            action_macro,
            ActionDefinition::MacroSequence(MacroSequenceAction {
                steps: vec![MacroStep::DelayMs(10)],
                concurrency: MacroConcurrencyPolicy::IgnoreIfRunning,
                require_initial_foreground_match: false,
            }),
        );

        // 3. HUD action
        profile.actions.insert(
            action_hud,
            ActionDefinition::ShowHud {
                duration_ms: 1500,
            },
        );

        // 4. Context action referencing HUD
        profile.actions.insert(
            action_context,
            ActionDefinition::ContextDispatch(ContextDispatchAction {
                rules: vec![ContextRule {
                    match_exe_name: Some("editor.exe".into()),
                    match_title_contains: None,
                    target_action_id: action_hud,
                }],
                fallback_action_id: action_win,
            }),
        );

        // Map triggers to slots
        let trig_slot_0 = PhantomTrigger::new(TriggerBank::BankA, 13).unwrap();
        let trig_slot_1 = PhantomTrigger::new(TriggerBank::BankA, 14).unwrap();
        let trig_slot_2 = PhantomTrigger::new(TriggerBank::BankA, 15).unwrap();

        profile.trigger_allocation.insert(0, trig_slot_0);
        profile.trigger_allocation.insert(1, trig_slot_1);
        profile.trigger_allocation.insert(2, trig_slot_2);

        // Slot 0 -> Window action
        profile.slots.get_mut(&0).unwrap().binding = SlotBinding::HostActionRef(action_win);
        // Slot 1 -> Macro action
        profile.slots.get_mut(&1).unwrap().binding = SlotBinding::HostActionRef(action_macro);
        // Slot 2 -> Context action
        profile.slots.get_mut(&2).unwrap().binding = SlotBinding::HostActionRef(action_context);

        (profile, action_win, action_macro, action_context)
    }

    #[test]
    fn test_dispatcher_event_routing_and_pause() {
        let (profile, _act_win, _act_mac, _act_ctx) = create_test_profile();
        let wm = Arc::new(DispatcherMockWm {
            fg: Mutex::new(ForegroundAppInfo {
                pid: 1,
                exe_name: "editor.exe".into(),
                exe_path: None,
                window_title: "My Code".into(),
                hwnd_raw: 123,
                x: 0,
                y: 0,
                width: 100,
                height: 100,
            }),
        });
        let input = Arc::new(DispatcherMockInput);

        let dispatcher = RuntimeDispatcher::new(profile, wm, input);

        // Trigger 0 -> Workspace
        let trig_0 = PhantomTrigger::new(TriggerBank::BankA, 13).unwrap();
        let out_0 = dispatcher.trigger_received(trig_0);
        match out_0 {
            DispatchOutcome::Workspace(report) => {
                assert_eq!(report.total_targets, 1);
                assert!(report.all_succeeded);
            }
            other => panic!("Expected Workspace outcome, got {:?}", other),
        }

        // Trigger 2 -> Context dispatch -> matched editor.exe -> ShowHud
        let trig_2 = PhantomTrigger::new(TriggerBank::BankA, 15).unwrap();
        let out_2 = dispatcher.trigger_received(trig_2);
        match out_2 {
            DispatchOutcome::ContextDispatched {
                resolved_action_id: _,
                outcome,
            } => match *outcome {
                DispatchOutcome::ShowHud { duration_ms } => {
                    assert_eq!(duration_ms, 1500);
                }
                other => panic!("Expected ShowHud sub-outcome, got {:?}", other),
            },
            other => panic!("Expected ContextDispatched outcome, got {:?}", other),
        }

        // Test Unmapped Trigger
        let trig_unmapped = PhantomTrigger::new(TriggerBank::BankE, 24).unwrap();
        assert_eq!(
            dispatcher.trigger_received(trig_unmapped),
            DispatchOutcome::UnmappedTrigger(trig_unmapped)
        );

        // Test Global Pause
        assert!(!dispatcher.is_paused());
        dispatcher.set_paused(true);
        assert!(dispatcher.is_paused());

        // Any trigger received while paused must return DispatchOutcome::Paused
        assert_eq!(dispatcher.trigger_received(trig_0), DispatchOutcome::Paused);

        // Resume and verify works again
        dispatcher.set_paused(false);
        assert!(!dispatcher.is_paused());
        assert!(matches!(
            dispatcher.trigger_received(trig_0),
            DispatchOutcome::Workspace(_)
        ));
    }
}
