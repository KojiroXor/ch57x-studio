//! Preset System for VortexDeck (CH57x Studio)
//!
//! Provides 1-click presets for:
//! 1. Rotary Encoders (3-in-1: CCW, Press, CW)
//! 2. Single Keys (Productivity quick chips)
//! 3. Full Layer Templates (18-slot batch configurations)

use ch57x_domain::actions::{
    ActionDefinition, ActionId, ExecProcessAction, HidAction, HidMod, MediaToken, MouseToken,
    SlotBinding,
};
use ch57x_domain::hardware::{ControlId, EncoderEvent, LayerIndex, SlotId, BUTTONS_PER_LAYER};
use ch57x_domain::profile::Profile;
use std::path::PathBuf;

/// Descrizione di un preset per Encoder (3 eventi in un colpo solo)
#[derive(Clone, Copy)]
pub struct EncoderPresetDef {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub ccw_label: &'static str,
    pub press_label: &'static str,
    pub cw_label: &'static str,
    pub make_bindings: fn() -> (SlotBinding, SlotBinding, SlotBinding),
}

/// Descrizione di un preset per Tasto singolo (Chip rapido)
#[derive(Clone, Copy)]
pub struct KeyPresetDef {
    pub id: &'static str,
    pub title: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub apply: fn(profile: &mut Profile, slot_idx: usize),
}

/// Descrizione di un template per intero layer
#[derive(Clone, Copy)]
pub struct LayerTemplateDef {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub apply: fn(profile: &mut Profile, layer: LayerIndex),
}

// ============================================================================
// 1. PRESET ENCODER 3-IN-1
// ============================================================================

pub const ENCODER_PRESETS: &[EncoderPresetDef] = &[
    EncoderPresetDef {
        id: "vol_mute",
        title: "Volume + Mute",
        description: "SX = Abbassa Volume | Click = Mute Audio | DX = Alza Volume",
        ccw_label: "Vol -",
        press_label: "Mute",
        cw_label: "Vol +",
        make_bindings: || {
            (
                SlotBinding::HidNative(HidAction::ConsumerMedia(MediaToken::VolumeDown)),
                SlotBinding::HidNative(HidAction::ConsumerMedia(MediaToken::Mute)),
                SlotBinding::HidNative(HidAction::ConsumerMedia(MediaToken::VolumeUp)),
            )
        },
    },
    EncoderPresetDef {
        id: "lum_play",
        title: "Luminosità + Play",
        description: "SX = Riduci Lum | Click = Play/Pausa | DX = Aumenta Lum",
        ccw_label: "Lum -",
        press_label: "Play/Pausa",
        cw_label: "Lum +",
        make_bindings: || {
            (
                SlotBinding::HidNative(HidAction::ConsumerMedia(MediaToken::BrightnessDown)),
                SlotBinding::HidNative(HidAction::ConsumerMedia(MediaToken::PlayPause)),
                SlotBinding::HidNative(HidAction::ConsumerMedia(MediaToken::BrightnessUp)),
            )
        },
    },
    EncoderPresetDef {
        id: "zoom_reset",
        title: "Zoom + Reset 100%",
        description: "SX = Zoom In | Click = Reset 100% (Ctrl+0) | DX = Zoom Out",
        ccw_label: "Zoom -",
        press_label: "100%",
        cw_label: "Zoom +",
        make_bindings: || {
            (
                SlotBinding::HidNative(HidAction::KeyboardChord {
                    modifiers: vec![HidMod::Ctrl],
                    key: "minus".into(),
                }),
                SlotBinding::HidNative(HidAction::KeyboardChord {
                    modifiers: vec![HidMod::Ctrl],
                    key: "0".into(),
                }),
                SlotBinding::HidNative(HidAction::KeyboardChord {
                    modifiers: vec![HidMod::Ctrl],
                    key: "equal".into(),
                }),
            )
        },
    },
    EncoderPresetDef {
        id: "tabs",
        title: "Scorri Tab (IDE/Browser)",
        description:
            "SX = Tab Prec (Ctrl+Shift+Tab) | Click = Chiudi (Ctrl+W) | DX = Tab Succ (Ctrl+Tab)",
        ccw_label: "Tab SX",
        press_label: "Chiudi Tab",
        cw_label: "Tab DX",
        make_bindings: || {
            (
                SlotBinding::HidNative(HidAction::KeyboardChord {
                    modifiers: vec![HidMod::Ctrl, HidMod::Shift],
                    key: "tab".into(),
                }),
                SlotBinding::HidNative(HidAction::KeyboardChord {
                    modifiers: vec![HidMod::Ctrl],
                    key: "w".into(),
                }),
                SlotBinding::HidNative(HidAction::KeyboardChord {
                    modifiers: vec![HidMod::Ctrl],
                    key: "tab".into(),
                }),
            )
        },
    },
    EncoderPresetDef {
        id: "undo_redo_save",
        title: "Undo / Redo / Salva",
        description: "SX = Annulla (Ctrl+Z) | Click = Salva (Ctrl+S) | DX = Ripristina (Ctrl+Y)",
        ccw_label: "Undo",
        press_label: "Salva",
        cw_label: "Redo",
        make_bindings: || {
            (
                SlotBinding::HidNative(HidAction::KeyboardChord {
                    modifiers: vec![HidMod::Ctrl],
                    key: "z".into(),
                }),
                SlotBinding::HidNative(HidAction::KeyboardChord {
                    modifiers: vec![HidMod::Ctrl],
                    key: "s".into(),
                }),
                SlotBinding::HidNative(HidAction::KeyboardChord {
                    modifiers: vec![HidMod::Ctrl],
                    key: "y".into(),
                }),
            )
        },
    },
    EncoderPresetDef {
        id: "desktops",
        title: "Desktop Virtuali Win11",
        description:
            "SX = Desk SX (Win+Ctrl+←) | Click = Task View (Win+Tab) | DX = Desk DX (Win+Ctrl+→)",
        ccw_label: "Desk SX",
        press_label: "TaskView",
        cw_label: "Desk DX",
        make_bindings: || {
            (
                SlotBinding::HidNative(HidAction::KeyboardChord {
                    modifiers: vec![HidMod::Win, HidMod::Ctrl],
                    key: "left".into(),
                }),
                SlotBinding::HidNative(HidAction::KeyboardChord {
                    modifiers: vec![HidMod::Win],
                    key: "tab".into(),
                }),
                SlotBinding::HidNative(HidAction::KeyboardChord {
                    modifiers: vec![HidMod::Win, HidMod::Ctrl],
                    key: "right".into(),
                }),
            )
        },
    },
    EncoderPresetDef {
        id: "mouse_scroll",
        title: "Scroll Mouse + Play",
        description: "SX = Rotella Su | Click = Play/Pausa | DX = Rotella Giù",
        ccw_label: "Scroll SU",
        press_label: "Play/Pausa",
        cw_label: "Scroll GIU",
        make_bindings: || {
            (
                SlotBinding::HidNative(HidAction::MouseAction(MouseToken::WheelUp)),
                SlotBinding::HidNative(HidAction::ConsumerMedia(MediaToken::PlayPause)),
                SlotBinding::HidNative(HidAction::MouseAction(MouseToken::WheelDown)),
            )
        },
    },
];

pub fn apply_encoder_preset(
    profile: &mut Profile,
    layer: LayerIndex,
    encoder_index: u8,
    preset: &EncoderPresetDef,
) {
    let ccw_slot = SlotId {
        layer,
        control: ControlId::Encoder {
            index: encoder_index,
            event: EncoderEvent::Ccw,
        },
    };
    let press_slot = SlotId {
        layer,
        control: ControlId::Encoder {
            index: encoder_index,
            event: EncoderEvent::Press,
        },
    };
    let cw_slot = SlotId {
        layer,
        control: ControlId::Encoder {
            index: encoder_index,
            event: EncoderEvent::Cw,
        },
    };

    let (ccw_b, press_b, cw_b) = if preset.id == "lum_play" {
        let action_ccw = ActionId::new_v4();
        let action_cw = ActionId::new_v4();
        profile
            .actions
            .insert(action_ccw, ActionDefinition::AdjustBrightness { delta: -5 });
        profile
            .actions
            .insert(action_cw, ActionDefinition::AdjustBrightness { delta: 5 });
        (
            SlotBinding::HostActionRef(action_ccw),
            SlotBinding::HidNative(HidAction::ConsumerMedia(MediaToken::PlayPause)),
            SlotBinding::HostActionRef(action_cw),
        )
    } else {
        (preset.make_bindings)()
    };

    if let Some(cfg) = profile.slots.get_mut(&ccw_slot.global_index()) {
        cfg.binding = ccw_b;
        cfg.label = preset.ccw_label.to_string();
    }
    if let Some(cfg) = profile.slots.get_mut(&press_slot.global_index()) {
        cfg.binding = press_b;
        cfg.label = preset.press_label.to_string();
    }
    if let Some(cfg) = profile.slots.get_mut(&cw_slot.global_index()) {
        cfg.binding = cw_b;
        cfg.label = preset.cw_label.to_string();
    }
}

// ============================================================================
// 2. PRESET TASTI SINGOLI (PRODUTTIVITÀ)
// ============================================================================

pub fn set_exec_process_slot(
    profile: &mut Profile,
    slot_idx: usize,
    program: &str,
    args: &[&str],
    label: &str,
) {
    let action_id = ActionId::new_v4();
    let action = ExecProcessAction {
        program: PathBuf::from(program),
        args: args.iter().map(|s| s.to_string()).collect(),
        working_dir: None,
        timeout_ms: 5000,
        hide_console_window: false,
    };
    profile
        .actions
        .insert(action_id, ActionDefinition::ExecProcess(action));
    if let Some(cfg) = profile.slots.get_mut(&slot_idx) {
        cfg.binding = SlotBinding::HostActionRef(action_id);
        cfg.label = label.to_string();
    }
}

pub fn set_hid_chord_slot(
    profile: &mut Profile,
    slot_idx: usize,
    modifiers: Vec<HidMod>,
    key: &str,
    label: &str,
) {
    if let Some(cfg) = profile.slots.get_mut(&slot_idx) {
        cfg.binding = SlotBinding::HidNative(HidAction::KeyboardChord {
            modifiers,
            key: key.to_string(),
        });
        cfg.label = label.to_string();
    }
}

pub const KEY_PRESETS: &[KeyPresetDef] = &[
    KeyPresetDef {
        id: "agy_cmd",
        title: "Apri agy (Prompt)",
        label: "agy",
        description: "Apre agy nel prompt comandi standard (cmd.exe /k agy)",
        category: "CLI & AI",
        apply: |p, idx| {
            set_exec_process_slot(
                p,
                idx,
                "cmd.exe",
                &["/c", "start", "cmd.exe", "/k", "agy"],
                "agy",
            );
        },
    },
    KeyPresetDef {
        id: "agy_wt",
        title: "Apri agy (Windows Terminal)",
        label: "agy (WT)",
        description: "Apre agy in Windows Terminal moderno (wt.exe -d . agy)",
        category: "CLI & AI",
        apply: |p, idx| {
            set_exec_process_slot(p, idx, "wt.exe", &["-d", ".", "agy"], "agy (WT)");
        },
    },
    KeyPresetDef {
        id: "wt",
        title: "Windows Terminal",
        label: "Terminal",
        description: "Avvia Windows Terminal nella cartella corrente (wt.exe)",
        category: "CLI & AI",
        apply: |p, idx| {
            set_exec_process_slot(p, idx, "wt.exe", &[], "Terminal");
        },
    },
    KeyPresetDef {
        id: "devmgmt",
        title: "Gestione Dispositivi",
        label: "DeviceMgr",
        description: "Apre Gestione Dispositivi Windows (mmc devmgmt.msc per debug USB/COM)",
        category: "Sistema",
        apply: |p, idx| {
            set_exec_process_slot(
                p,
                idx,
                "cmd.exe",
                &["/c", "start", "devmgmt.msc"],
                "DeviceMgr",
            );
        },
    },
    KeyPresetDef {
        id: "screenshot",
        title: "Screenshot Area",
        label: "Screen",
        description: "Cattura schermata selettiva Windows (Win+Shift+S)",
        category: "Utilità",
        apply: |p, idx| {
            set_hid_chord_slot(p, idx, vec![HidMod::Win, HidMod::Shift], "s", "Screen");
        },
    },
    KeyPresetDef {
        id: "clipboard",
        title: "Cronologia Appunti",
        label: "Appunti",
        description: "Apre lo storico appunti multilivello di Windows (Win+V)",
        category: "Utilità",
        apply: |p, idx| {
            set_hid_chord_slot(p, idx, vec![HidMod::Win], "v", "Appunti");
        },
    },
    KeyPresetDef {
        id: "mic_mute",
        title: "Mute Microfono Win11",
        label: "Mute Mic",
        description: "Mute microfono globale nativo Windows 11 (Win+Alt+K)",
        category: "Utilità",
        apply: |p, idx| {
            set_hid_chord_slot(p, idx, vec![HidMod::Win, HidMod::Alt], "k", "Mute Mic");
        },
    },
    KeyPresetDef {
        id: "taskmgr",
        title: "Task Manager",
        label: "TaskMgr",
        description: "Apre Gestione Attività Windows (Ctrl+Shift+Esc)",
        category: "Sistema",
        apply: |p, idx| {
            set_hid_chord_slot(
                p,
                idx,
                vec![HidMod::Ctrl, HidMod::Shift],
                "escape",
                "TaskMgr",
            );
        },
    },
    KeyPresetDef {
        id: "move_monitor",
        title: "Sposta Altro Monitor",
        label: "Move Disp",
        description: "Sposta finestra attiva al monitor successivo (Win+Shift+→)",
        category: "Finestre",
        apply: |p, idx| {
            set_hid_chord_slot(
                p,
                idx,
                vec![HidMod::Win, HidMod::Shift],
                "right",
                "Move Disp",
            );
        },
    },
    KeyPresetDef {
        id: "copy",
        title: "Copia",
        label: "Copia",
        description: "Copia negli appunti (Ctrl+C)",
        category: "Editing",
        apply: |p, idx| {
            set_hid_chord_slot(p, idx, vec![HidMod::Ctrl], "c", "Copia");
        },
    },
    KeyPresetDef {
        id: "paste",
        title: "Incolla",
        label: "Incolla",
        description: "Incolla dagli appunti (Ctrl+V)",
        category: "Editing",
        apply: |p, idx| {
            set_hid_chord_slot(p, idx, vec![HidMod::Ctrl], "v", "Incolla");
        },
    },
    KeyPresetDef {
        id: "cut",
        title: "Taglia",
        label: "Taglia",
        description: "Taglia selezione (Ctrl+X)",
        category: "Editing",
        apply: |p, idx| {
            set_hid_chord_slot(p, idx, vec![HidMod::Ctrl], "x", "Taglia");
        },
    },
    KeyPresetDef {
        id: "undo",
        title: "Undo (Annulla)",
        label: "Undo",
        description: "Annulla ultima modifica (Ctrl+Z)",
        category: "Editing",
        apply: |p, idx| {
            set_hid_chord_slot(p, idx, vec![HidMod::Ctrl], "z", "Undo");
        },
    },
    KeyPresetDef {
        id: "redo",
        title: "Redo (Ripristina)",
        label: "Redo",
        description: "Ripristina modifica annullata (Ctrl+Y)",
        category: "Editing",
        apply: |p, idx| {
            set_hid_chord_slot(p, idx, vec![HidMod::Ctrl], "y", "Redo");
        },
    },
    KeyPresetDef {
        id: "save",
        title: "Salva",
        label: "Salva",
        description: "Salva file attivo (Ctrl+S)",
        category: "Editing",
        apply: |p, idx| {
            set_hid_chord_slot(p, idx, vec![HidMod::Ctrl], "s", "Salva");
        },
    },
    KeyPresetDef {
        id: "search",
        title: "Cerca",
        label: "Cerca",
        description: "Trova nel documento o nell'app (Ctrl+F)",
        category: "Editing",
        apply: |p, idx| {
            set_hid_chord_slot(p, idx, vec![HidMod::Ctrl], "f", "Cerca");
        },
    },
];

// ============================================================================
// 3. TEMPLATE INTERO LAYER
// ============================================================================

struct KeyConfigEntry {
    _id: &'static str,
    apply_fn: fn(&mut Profile, usize),
}

struct WinKeyConfigEntry {
    modifiers: &'static [HidMod],
    key: &'static str,
    label: &'static str,
}

pub const LAYER_TEMPLATES: &[LayerTemplateDef] = &[
    LayerTemplateDef {
        id: "coding_ai",
        title: "Coding & Produttività AI",
        description: "Encoder: Volume+Mute e Zoom+Reset | Tasti: agy, Terminal, Screen, Appunti, TaskMgr, Taglia, Copia, Incolla, Undo, Redo, Salva.",
        apply: |profile, layer| {
            // Encoder 0 (Superiore): Volume + Mute
            apply_encoder_preset(profile, layer, 0, &ENCODER_PRESETS[0]);
            // Encoder 1 (Inferiore): Zoom + Reset
            apply_encoder_preset(profile, layer, 1, &ENCODER_PRESETS[2]);

            // 12 Tasti Matrice 4x3
            let keys_configs: &[KeyConfigEntry] = &[
                // Riga 0
                KeyConfigEntry {
                    _id: "agy_cmd",
                    apply_fn: |p, i| set_exec_process_slot(p, i, "cmd.exe", &["/c", "start", "cmd.exe", "/k", "agy"], "agy"),
                },
                KeyConfigEntry {
                    _id: "wt",
                    apply_fn: |p, i| set_exec_process_slot(p, i, "wt.exe", &[], "Terminal"),
                },
                KeyConfigEntry {
                    _id: "screenshot",
                    apply_fn: |p, i| set_hid_chord_slot(p, i, vec![HidMod::Win, HidMod::Shift], "s", "Screen"),
                },
                KeyConfigEntry {
                    _id: "clipboard",
                    apply_fn: |p, i| set_hid_chord_slot(p, i, vec![HidMod::Win], "v", "Appunti"),
                },
                // Riga 1
                KeyConfigEntry {
                    _id: "devmgmt",
                    apply_fn: |p, i| set_exec_process_slot(p, i, "cmd.exe", &["/c", "start", "devmgmt.msc"], "DeviceMgr"),
                },
                KeyConfigEntry {
                    _id: "taskmgr",
                    apply_fn: |p, i| set_hid_chord_slot(p, i, vec![HidMod::Ctrl, HidMod::Shift], "escape", "TaskMgr"),
                },
                KeyConfigEntry {
                    _id: "cut",
                    apply_fn: |p, i| set_hid_chord_slot(p, i, vec![HidMod::Ctrl], "x", "Taglia"),
                },
                KeyConfigEntry {
                    _id: "copy",
                    apply_fn: |p, i| set_hid_chord_slot(p, i, vec![HidMod::Ctrl], "c", "Copia"),
                },
                // Riga 2
                KeyConfigEntry {
                    _id: "paste",
                    apply_fn: |p, i| set_hid_chord_slot(p, i, vec![HidMod::Ctrl], "v", "Incolla"),
                },
                KeyConfigEntry {
                    _id: "undo",
                    apply_fn: |p, i| set_hid_chord_slot(p, i, vec![HidMod::Ctrl], "z", "Undo"),
                },
                KeyConfigEntry {
                    _id: "redo",
                    apply_fn: |p, i| set_hid_chord_slot(p, i, vec![HidMod::Ctrl], "y", "Redo"),
                },
                KeyConfigEntry {
                    _id: "save",
                    apply_fn: |p, i| set_hid_chord_slot(p, i, vec![HidMod::Ctrl], "s", "Salva"),
                },
            ];

            for (idx, entry) in keys_configs.iter().enumerate() {
                if idx < BUTTONS_PER_LAYER {
                    let slot = SlotId {
                        layer,
                        control: ControlId::Button {
                            row: (idx / 4) as u8,
                            col: (idx % 4) as u8,
                        },
                    };
                    (entry.apply_fn)(profile, slot.global_index());
                }
            }
        },
    },
    LayerTemplateDef {
        id: "window_tiling",
        title: "Window & Multi-Monitor Tiling",
        description: "Encoder: Scorri Tab e Desktop Virtuali | Tasti: Snap Metà SX/DX, Massimizza, Minimizza, Sposta Monitor, Task View, Mostra Desktop.",
        apply: |profile, layer| {
            // Encoder 0: Scorri Tab
            apply_encoder_preset(profile, layer, 0, &ENCODER_PRESETS[3]);
            // Encoder 1: Desktop Virtuali
            apply_encoder_preset(profile, layer, 1, &ENCODER_PRESETS[5]);

            // 12 Tasti Finestre Windows
            let win_keys: &[WinKeyConfigEntry] = &[
                // Riga 0
                WinKeyConfigEntry { modifiers: &[HidMod::Win], key: "left", label: "Metà SX" },
                WinKeyConfigEntry { modifiers: &[HidMod::Win], key: "up", label: "Massimizza" },
                WinKeyConfigEntry { modifiers: &[HidMod::Win], key: "right", label: "Metà DX" },
                WinKeyConfigEntry { modifiers: &[HidMod::Win], key: "down", label: "Minimizza" },
                // Riga 1
                WinKeyConfigEntry { modifiers: &[HidMod::Win, HidMod::Shift], key: "right", label: "Monitor DX" },
                WinKeyConfigEntry { modifiers: &[HidMod::Win, HidMod::Shift], key: "left", label: "Monitor SX" },
                WinKeyConfigEntry { modifiers: &[HidMod::Win], key: "tab", label: "TaskView" },
                WinKeyConfigEntry { modifiers: &[HidMod::Win], key: "d", label: "Desktop" },
                // Riga 2
                WinKeyConfigEntry { modifiers: &[HidMod::Alt], key: "space", label: "Menu Finestra" },
                WinKeyConfigEntry { modifiers: &[HidMod::Win, HidMod::Ctrl], key: "d", label: "Nuovo Desk" },
                WinKeyConfigEntry { modifiers: &[HidMod::Win, HidMod::Ctrl], key: "f4", label: "Chiudi Desk" },
                WinKeyConfigEntry { modifiers: &[HidMod::Win], key: "e", label: "Esplora File" },
            ];

            for (idx, entry) in win_keys.iter().enumerate() {
                if idx < BUTTONS_PER_LAYER {
                    let slot = SlotId {
                        layer,
                        control: ControlId::Button {
                            row: (idx / 4) as u8,
                            col: (idx % 4) as u8,
                        },
                    };
                    set_hid_chord_slot(profile, slot.global_index(), entry.modifiers.to_vec(), entry.key, entry.label);
                }
            }
        },
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_encoder_presets() {
        let mut profile = Profile::default();
        let layer = LayerIndex(0);

        for preset in ENCODER_PRESETS {
            apply_encoder_preset(&mut profile, layer, 0, preset);
            let ccw_slot = SlotId {
                layer,
                control: ControlId::Encoder {
                    index: 0,
                    event: EncoderEvent::Ccw,
                },
            };
            let cfg = profile.slots.get(&ccw_slot.global_index()).unwrap();
            assert_eq!(cfg.label, preset.ccw_label);
        }
    }

    #[test]
    fn test_apply_layer_templates() {
        let mut profile = Profile::default();
        let layer = LayerIndex(0);

        for template in LAYER_TEMPLATES {
            (template.apply)(&mut profile, layer);
            // Verify buttons are non-empty
            for b in 0..12 {
                let slot = SlotId {
                    layer,
                    control: ControlId::Button {
                        row: (b / 4) as u8,
                        col: (b % 4) as u8,
                    },
                };
                let cfg = profile.slots.get(&slot.global_index()).unwrap();
                assert_ne!(cfg.label, format!("Slot {}", slot.global_index() + 1));
            }
        }
    }
}
