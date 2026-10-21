use crate::actions::{
    ActionDefinition, ActionId, HidAction, MacroStep, SlotBinding, WindowCoordinates,
};
use crate::errors::DomainError;
use crate::hardware::{SlotId, TOTAL_SLOTS};
use crate::profile::Profile;
use crate::triggers::PhantomTrigger;
use std::collections::HashSet;

/// Validates the semantic integrity of a `Profile`.
///
/// Ensures that:
/// 1. Exactly 54 slots are defined, matching all canonical `SlotId`s.
/// 2. All referenced `ActionId`s in slots exist in `profile.actions`.
/// 3. All `ContextDispatch` actions reference existing actions for rules and fallback.
/// 4. Direct self-cycles in `ContextDispatch` are prevented.
/// 5. Trigger allocations are within valid slot bounds and have no collisions.
/// 6. Action parameters (paths, coordinates, step values) are semantically valid.
pub fn validate_profile(profile: &Profile) -> Result<(), DomainError> {
    if profile.version == 0 {
        return Err(DomainError::Validation(
            "Profile version must be greater than or equal to 1".into(),
        ));
    }

    validate_slots(profile)?;
    validate_actions(profile)?;
    validate_trigger_allocations(profile)?;

    Ok(())
}

fn validate_slots(profile: &Profile) -> Result<(), DomainError> {
    if profile.slots.len() != TOTAL_SLOTS {
        return Err(DomainError::Validation(format!(
            "Invalid slot count: expected {} slots, found {}",
            TOTAL_SLOTS,
            profile.slots.len()
        )));
    }

    let all_canonical_slots = SlotId::all_slots();
    for canonical_slot in &all_canonical_slots {
        let global_idx = canonical_slot.global_index();
        let slot_cfg = profile.slots.get(&global_idx).ok_or_else(|| {
            DomainError::Validation(format!(
                "Missing canonical slot at global index {}",
                global_idx
            ))
        })?;

        if slot_cfg.slot != *canonical_slot {
            return Err(DomainError::Validation(format!(
                "Slot at index {} has mismatched SlotId: expected {:?}, found {:?}",
                global_idx, canonical_slot, slot_cfg.slot
            )));
        }

        match &slot_cfg.binding {
            SlotBinding::Disabled => {}
            SlotBinding::HidNative(hid) => match hid {
                HidAction::KeyboardChord { key, .. } => {
                    if key.trim().is_empty() {
                        return Err(DomainError::Validation(format!(
                            "Slot {} has empty key in KeyboardChord",
                            global_idx
                        )));
                    }
                }
                HidAction::ConsumerMedia(_) | HidAction::MouseAction(_) => {}
            },
            SlotBinding::HostActionRef(action_id) => {
                if !profile.actions.contains_key(action_id) {
                    return Err(DomainError::Validation(format!(
                        "Slot {} references nonexistent ActionId: {}",
                        global_idx, action_id
                    )));
                }
            }
        }
    }

    Ok(())
}

fn validate_actions(profile: &Profile) -> Result<(), DomainError> {
    for (action_id, action_def) in &profile.actions {
        match action_def {
            ActionDefinition::LaunchOrFocus(action) => {
                let has_selector = action.selector.exe_name.is_some()
                    || action.selector.title_contains.is_some()
                    || action.selector.class_name.is_some();
                let has_launch = action.launch.is_some();

                if !has_selector && !has_launch {
                    return Err(DomainError::Validation(format!(
                        "LaunchOrFocus action {} has neither window selector nor launch recipe",
                        action_id
                    )));
                }

                if let Some(ref placement) = action.placement {
                    validate_coordinates(action_id, &placement.coordinates)?;
                }
            }
            ActionDefinition::RestoreWorkspace(action) => {
                for (target_idx, target) in action.targets.iter().enumerate() {
                    let has_selector = target.selector.exe_name.is_some()
                        || target.selector.title_contains.is_some()
                        || target.selector.class_name.is_some();
                    let has_launch = target.launch.is_some();

                    if !has_selector && !has_launch {
                        return Err(DomainError::Validation(format!(
                            "RestoreWorkspace action {} target {} has neither selector nor launch recipe",
                            action_id, target_idx
                        )));
                    }

                    validate_coordinates(action_id, &target.placement.coordinates)?;
                }
            }
            ActionDefinition::MacroSequence(action) => {
                for (step_idx, step) in action.steps.iter().enumerate() {
                    match step {
                        MacroStep::KeyTap { key } => {
                            if key.trim().is_empty() {
                                return Err(DomainError::Validation(format!(
                                    "MacroSequence action {} step {} KeyTap has empty key",
                                    action_id, step_idx
                                )));
                            }
                        }
                        MacroStep::KeyChord { key, .. } => {
                            if key.trim().is_empty() {
                                return Err(DomainError::Validation(format!(
                                    "MacroSequence action {} step {} KeyChord has empty key",
                                    action_id, step_idx
                                )));
                            }
                        }
                        MacroStep::TypeText { text } => {
                            if text.is_empty() {
                                return Err(DomainError::Validation(format!(
                                    "MacroSequence action {} step {} TypeText has empty text",
                                    action_id, step_idx
                                )));
                            }
                        }
                        MacroStep::ClipboardPaste { text, .. } => {
                            if text.is_empty() {
                                return Err(DomainError::Validation(format!(
                                    "MacroSequence action {} step {} ClipboardPaste has empty text",
                                    action_id, step_idx
                                )));
                            }
                        }
                        MacroStep::WaitForWindow {
                            selector,
                            timeout_ms,
                        } => {
                            let has_sel = selector.exe_name.is_some()
                                || selector.title_contains.is_some()
                                || selector.class_name.is_some();
                            if !has_sel {
                                return Err(DomainError::Validation(format!(
                                    "MacroSequence action {} step {} WaitForWindow has empty selector",
                                    action_id, step_idx
                                )));
                            }
                            if *timeout_ms == 0 {
                                return Err(DomainError::Validation(format!(
                                    "MacroSequence action {} step {} WaitForWindow timeout must be > 0",
                                    action_id, step_idx
                                )));
                            }
                        }
                        MacroStep::DelayMs(_) => {}
                    }
                }
            }
            ActionDefinition::ExecProcess(action) => {
                if action.program.as_os_str().is_empty() {
                    return Err(DomainError::Validation(format!(
                        "ExecProcess action {} has empty program path",
                        action_id
                    )));
                }
            }
            ActionDefinition::ContextDispatch(action) => {
                if !profile.actions.contains_key(&action.fallback_action_id) {
                    return Err(DomainError::Validation(format!(
                        "ContextDispatch action {} references nonexistent fallback action {}",
                        action_id, action.fallback_action_id
                    )));
                }
                if action.fallback_action_id == *action_id {
                    return Err(DomainError::Validation(format!(
                        "ContextDispatch action {} references itself as fallback (cycle detected)",
                        action_id
                    )));
                }

                for (rule_idx, rule) in action.rules.iter().enumerate() {
                    if !profile.actions.contains_key(&rule.target_action_id) {
                        return Err(DomainError::Validation(format!(
                            "ContextDispatch action {} rule {} references nonexistent target action {}",
                            action_id, rule_idx, rule.target_action_id
                        )));
                    }
                    if rule.target_action_id == *action_id {
                        return Err(DomainError::Validation(format!(
                            "ContextDispatch action {} rule {} references itself as target (cycle detected)",
                            action_id, rule_idx
                        )));
                    }
                    let has_match =
                        rule.match_exe_name.is_some() || rule.match_title_contains.is_some();
                    if !has_match {
                        return Err(DomainError::Validation(format!(
                            "ContextDispatch action {} rule {} has no match condition",
                            action_id, rule_idx
                        )));
                    }
                }
            }
            ActionDefinition::ShowHud { duration_ms } => {
                if *duration_ms == 0 {
                    return Err(DomainError::Validation(format!(
                        "ShowHud action {} duration must be > 0",
                        action_id
                    )));
                }
            }
            ActionDefinition::AdjustBrightness { delta } => {
                if *delta == 0 {
                    return Err(DomainError::Validation(format!(
                        "AdjustBrightness action {} delta must be non-zero",
                        action_id
                    )));
                }
            }
        }
    }

    Ok(())
}

fn validate_coordinates(
    action_id: &ActionId,
    coords: &WindowCoordinates,
) -> Result<(), DomainError> {
    match coords {
        WindowCoordinates::Absolute { width, height, .. } => {
            if *width <= 0 || *height <= 0 {
                return Err(DomainError::Validation(format!(
                    "Window dimensions must be positive in action {}, got width={}, height={}",
                    action_id, width, height
                )));
            }
        }
        WindowCoordinates::Normalized {
            x,
            y,
            width,
            height,
        } => {
            if *width <= 0.0 || *height <= 0.0 {
                return Err(DomainError::Validation(format!(
                    "Normalized window dimensions must be > 0.0 in action {}, got width={}, height={}",
                    action_id, width, height
                )));
            }
            if *x < 0.0 || *y < 0.0 || (*x + *width) > 1.05 || (*y + *height) > 1.05 {
                return Err(DomainError::Validation(format!(
                    "Normalized coordinates out of [0.0, 1.0] bounds in action {}: x={}, y={}, w={}, h={}",
                    action_id, x, y, width, height
                )));
            }
        }
    }
    Ok(())
}

fn validate_trigger_allocations(profile: &Profile) -> Result<(), DomainError> {
    let mut seen_triggers: HashSet<PhantomTrigger> = HashSet::new();

    for (slot_idx, trigger) in &profile.trigger_allocation {
        if *slot_idx >= TOTAL_SLOTS {
            return Err(DomainError::Validation(format!(
                "Trigger allocation for out-of-range slot index {}",
                slot_idx
            )));
        }

        if !seen_triggers.insert(*trigger) {
            return Err(DomainError::Validation(format!(
                "Duplicate trigger {:?} allocated to multiple slots (detected at slot index {})",
                trigger, slot_idx
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::*;
    use crate::hardware::SlotId;
    use crate::triggers::TriggerBank;
    use std::path::PathBuf;

    #[test]
    fn test_default_profile_is_valid() {
        let profile = Profile::default();
        let result = validate_profile(&profile);
        assert!(
            result.is_ok(),
            "Default profile must be valid: {:?}",
            result
        );
    }

    #[test]
    fn test_missing_slot_fails_validation() {
        let mut profile = Profile::default();
        profile.slots.remove(&0);
        let result = validate_profile(&profile);
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => assert!(msg.contains("Invalid slot count")),
            other => panic!("Expected DomainError::Validation, got {:?}", other),
        }
    }

    #[test]
    fn test_corrupted_slot_id_fails_validation() {
        let mut profile = Profile::default();
        // Replace slot 0 with wrong SlotId
        let slot_cfg = profile.slots.get_mut(&0).unwrap();
        slot_cfg.slot = SlotId::new_button(1, 0, 0).unwrap(); // layer 1 instead of layer 0
        let result = validate_profile(&profile);
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => assert!(msg.contains("mismatched SlotId")),
            other => panic!("Expected DomainError::Validation, got {:?}", other),
        }
    }

    #[test]
    fn test_nonexistent_action_reference_fails() {
        let mut profile = Profile::default();
        let non_existent_id = ActionId::new_v4();
        let slot_cfg = profile.slots.get_mut(&5).unwrap();
        slot_cfg.binding = SlotBinding::HostActionRef(non_existent_id);

        let result = validate_profile(&profile);
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert!(msg.contains("references nonexistent ActionId"))
            }
            other => panic!("Expected DomainError::Validation, got {:?}", other),
        }
    }

    #[test]
    fn test_context_dispatch_validation() {
        let mut profile = Profile::default();

        let fallback_id = ActionId::new_v4();
        profile
            .actions
            .insert(fallback_id, ActionDefinition::ShowHud { duration_ms: 1000 });

        let target_id = ActionId::new_v4();
        profile.actions.insert(
            target_id,
            ActionDefinition::ExecProcess(ExecProcessAction {
                program: PathBuf::from("calc.exe"),
                args: vec![],
                working_dir: None,
                timeout_ms: 1000,
                hide_console_window: true,
            }),
        );

        let dispatch_id = ActionId::new_v4();
        profile.actions.insert(
            dispatch_id,
            ActionDefinition::ContextDispatch(ContextDispatchAction {
                rules: vec![ContextRule {
                    match_exe_name: Some("code.exe".into()),
                    match_title_contains: None,
                    target_action_id: target_id,
                }],
                fallback_action_id: fallback_id,
            }),
        );

        let slot_cfg = profile.slots.get_mut(&0).unwrap();
        slot_cfg.binding = SlotBinding::HostActionRef(dispatch_id);

        // Valid setup passes
        assert!(validate_profile(&profile).is_ok());

        // Missing fallback fails
        let bad_fallback_id = ActionId::new_v4();
        if let Some(ActionDefinition::ContextDispatch(ref mut d)) =
            profile.actions.get_mut(&dispatch_id)
        {
            d.fallback_action_id = bad_fallback_id;
        }
        let err = validate_profile(&profile).unwrap_err();
        match err {
            DomainError::Validation(msg) => {
                assert!(msg.contains("references nonexistent fallback action"))
            }
            other => panic!("Expected DomainError::Validation, got {:?}", other),
        }

        // Self-referencing fallback fails
        if let Some(ActionDefinition::ContextDispatch(ref mut d)) =
            profile.actions.get_mut(&dispatch_id)
        {
            d.fallback_action_id = dispatch_id;
        }
        let err = validate_profile(&profile).unwrap_err();
        match err {
            DomainError::Validation(msg) => assert!(msg.contains("cycle detected")),
            other => panic!("Expected DomainError::Validation, got {:?}", other),
        }
    }

    #[test]
    fn test_duplicate_trigger_allocation_fails() {
        let mut profile = Profile::default();
        let trigger = PhantomTrigger::new(TriggerBank::BankA, 13).unwrap();
        profile.trigger_allocation.insert(0, trigger);
        profile.trigger_allocation.insert(1, trigger); // Duplicate!

        let result = validate_profile(&profile);
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => assert!(msg.contains("Duplicate trigger")),
            other => panic!("Expected DomainError::Validation, got {:?}", other),
        }
    }

    #[test]
    fn test_invalid_window_coordinates_fails() {
        let mut profile = Profile::default();
        let act_id = ActionId::new_v4();
        profile.actions.insert(
            act_id,
            ActionDefinition::LaunchOrFocus(LaunchOrFocusAction {
                selector: WindowSelector {
                    exe_name: Some("notepad.exe".into()),
                    title_contains: None,
                    class_name: None,
                },
                placement: Some(WindowPlacement {
                    monitor_index: Some(0),
                    coordinates: WindowCoordinates::Absolute {
                        x: 0,
                        y: 0,
                        width: -100, // Invalid negative width
                        height: 500,
                    },
                    bring_to_foreground: true,
                }),
                launch: None,
                timeout_ms: 1000,
            }),
        );

        let result = validate_profile(&profile);
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert!(msg.contains("Window dimensions must be positive"))
            }
            other => panic!("Expected DomainError::Validation, got {:?}", other),
        }
    }
}
