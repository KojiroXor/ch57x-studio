//! YAML generator compatible with ch57x-keyboard-tool
//!
//! Genera la configurazione YAML fedele per il tool di flash hardware
//! con struttura:
//! - Header: orientation: normal, rows: 3, columns: 4, knobs: 2
//! - 3 layer completi:
//!   - buttons: matrice 3x4
//!   - knobs: 2 knob (ccw, press, cw) rispettando device.encoder_order_swapped

use ch57x_domain::actions::{HidAction, SlotBinding};
use ch57x_domain::errors::DomainError;
use ch57x_domain::hardware::{COL_COUNT, LAYER_COUNT, ROW_COUNT, SLOTS_PER_LAYER};
use ch57x_domain::profile::Profile;

pub const DEFAULT_DISABLED_TOKEN: &str = "f24";

/// Risolve il binding di uno slot nel token YAML atteso da ch57x-keyboard-tool
pub fn resolve_slot_token(profile: &Profile, slot_index: usize) -> Result<String, DomainError> {
    let binding = match profile.slots.get(&slot_index) {
        Some(cfg) => &cfg.binding,
        None => &SlotBinding::Disabled,
    };

    match binding {
        SlotBinding::Disabled => Ok(DEFAULT_DISABLED_TOKEN.to_string()),
        SlotBinding::HostActionRef(_) => {
            if let Some(trigger) = profile.trigger_allocation.get(&slot_index) {
                Ok(trigger.yaml_token())
            } else {
                Err(DomainError::Validation(format!(
                    "Slot {} is configured with HostActionRef but has no PhantomTrigger allocated",
                    slot_index
                )))
            }
        }
        SlotBinding::HidNative(hid) => match hid {
            HidAction::KeyboardChord { modifiers, key } => {
                let key_norm = key.trim().to_lowercase();
                if modifiers.is_empty() {
                    Ok(key_norm)
                } else {
                    let mut parts: Vec<&str> = modifiers.iter().map(|m| m.to_token()).collect();
                    parts.push(&key_norm);
                    Ok(parts.join("-"))
                }
            }
            HidAction::ConsumerMedia(media) => Ok(media.yaml_token().to_string()),
            HidAction::MouseAction(mouse) => Ok(mouse.yaml_token().to_string()),
        },
    }
}

/// Genera il documento YAML byte-exact per ch57x-keyboard-tool
pub fn generate_yaml(profile: &Profile) -> Result<String, DomainError> {
    let mut out = String::with_capacity(2048);

    // Header obbligatorio
    out.push_str("orientation: normal\n");
    out.push_str("rows: 3\n");
    out.push_str("columns: 4\n");
    out.push_str("knobs: 2\n");
    out.push_str("layers:\n");

    let encoder_indices = if profile.device.encoder_order_swapped {
        [1usize, 0usize]
    } else {
        [0usize, 1usize]
    };

    for layer in 0..LAYER_COUNT {
        out.push_str("  - buttons:\n");
        let layer_base = layer * SLOTS_PER_LAYER;

        // 3 righe x 4 colonne
        for row in 0..ROW_COUNT {
            out.push_str("      - [");
            for col in 0..COL_COUNT {
                let slot_idx = layer_base + (row * COL_COUNT) + col;
                let token = resolve_slot_token(profile, slot_idx)?;
                if col > 0 {
                    out.push_str(", ");
                }
                out.push('"');
                out.push_str(&token);
                out.push('"');
            }
            out.push_str("]\n");
        }

        out.push_str("    knobs:\n");
        for &enc_idx in &encoder_indices {
            let enc_base = layer_base + (ROW_COUNT * COL_COUNT) + (enc_idx * 3);
            let ccw_token = resolve_slot_token(profile, enc_base)?;
            let press_token = resolve_slot_token(profile, enc_base + 1)?;
            let cw_token = resolve_slot_token(profile, enc_base + 2)?;

            out.push_str("      - ccw: \"");
            out.push_str(&ccw_token);
            out.push_str("\"\n");

            out.push_str("        press: \"");
            out.push_str(&press_token);
            out.push_str("\"\n");

            out.push_str("        cw: \"");
            out.push_str(&cw_token);
            out.push_str("\"\n");
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::allocator::allocate_triggers;
    use ch57x_domain::actions::{ActionId, HidMod, MediaToken, MouseToken};

    #[test]
    fn test_generate_yaml_default_profile() {
        let profile = Profile::default();
        let yaml = generate_yaml(&profile).expect("Default profile YAML generation must succeed");

        assert!(yaml.starts_with("orientation: normal\nrows: 3\ncolumns: 4\nknobs: 2\nlayers:\n"));
        // Tutti i tasti disabilitati devono usare f24
        assert!(yaml.contains("\"f24\""));
    }

    #[test]
    fn test_generate_yaml_with_all_action_types() {
        let mut profile = Profile::default();

        // Slot 0: KeyboardChord ctrl-shift-c
        profile.slots.get_mut(&0).unwrap().binding =
            SlotBinding::HidNative(HidAction::KeyboardChord {
                modifiers: vec![HidMod::Ctrl, HidMod::Shift],
                key: "c".into(),
            });

        // Slot 1: Media volumeup
        profile.slots.get_mut(&1).unwrap().binding =
            SlotBinding::HidNative(HidAction::ConsumerMedia(MediaToken::VolumeUp));

        // Slot 2: Mouse click
        profile.slots.get_mut(&2).unwrap().binding =
            SlotBinding::HidNative(HidAction::MouseAction(MouseToken::Click));

        // Slot 3: HostActionRef
        profile.slots.get_mut(&3).unwrap().binding = SlotBinding::HostActionRef(ActionId::new_v4());

        allocate_triggers(&mut profile).unwrap();

        let yaml = generate_yaml(&profile).expect("YAML generation must succeed");
        assert!(yaml.contains("\"ctrl-shift-c\""));
        assert!(yaml.contains("\"volumeup\""));
        assert!(yaml.contains("\"click\""));
        assert!(yaml.contains("\"ctrl-alt-shift-f1\"")); // Primo trigger allocato per slot 3
    }

    #[test]
    fn test_encoder_order_swapped() {
        let mut profile = Profile::default();
        profile.device.encoder_order_swapped = true;

        // Encoder 0 CCW (slot 12): VolumeDown
        profile.slots.get_mut(&12).unwrap().binding =
            SlotBinding::HidNative(HidAction::ConsumerMedia(MediaToken::VolumeDown));

        // Encoder 1 CCW (slot 15): VolumeUp
        profile.slots.get_mut(&15).unwrap().binding =
            SlotBinding::HidNative(HidAction::ConsumerMedia(MediaToken::VolumeUp));

        let yaml = generate_yaml(&profile).unwrap();

        // Con encoder_order_swapped = true, Encoder 1 (VolumeUp) deve apparire PRIMA di Encoder 0 (VolumeDown)
        let pos_volup = yaml.find("\"volumeup\"").unwrap();
        let pos_voldown = yaml.find("\"volumedown\"").unwrap();
        assert!(
            pos_volup < pos_voldown,
            "Encoder 1 must precede Encoder 0 when swapped"
        );
    }
}
