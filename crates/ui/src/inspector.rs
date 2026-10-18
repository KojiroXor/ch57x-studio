//! Contextual slot inspector widget
//!
//! Permette di configurare:
//! - Etichetta testuale e selettore colore.
//! - Tipo di binding: `Disabled`, `HidNative` (Tasti speciali, Media, Mouse), `LaunchOrFocus`, `RestoreWorkspace`, `MacroSequence`, `ExecProcess`, `ContextDispatch`.
//! - Per `LaunchOrFocus`:
//!   - Countdown non-bloccante di 3 secondi per cattura automatica della finestra attiva ("🎯 Cattura Finestra").
//!   - Quick Snap buttons: Metà SX, Metà DX, Schermo Intero, Quadrante Alto-SX, Quadrante Basso-DX.
//!   - Pulsante "▶️ Testa Azione Ora".

use crate::pad_canvas::badge_color;
use crate::state::{hex_to_color32, StatusLevel, UiState};
use ch57x_domain::actions::{
    ActionDefinition, ActionId, ContextRule, ExecProcessAction, HidAction,
    HidMod, MacroConcurrencyPolicy, MacroSequenceAction, MacroStep,
    MediaToken, MouseToken, SlotBinding, WindowCoordinates,
    WindowPlacement, WindowSelector, WorkspaceTarget,
};
use ch57x_domain::hardware::{ControlId, EncoderEvent, SlotId};
use ch57x_platform::PlatformBackend;
use egui::{
    vec2, Align2, Color32, ComboBox, FontId, ProgressBar, Rounding, Sense, Stroke, TextEdit, Ui,
};
use std::path::PathBuf;

pub struct InspectorPanel;

impl Default for InspectorPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl InspectorPanel {
    pub fn new() -> Self {
        Self
    }

    /// Disegna il pannello di ispezione e configurazione dello slot selezionato
    pub fn show(
        &mut self,
        ui: &mut Ui,
        state: &mut UiState,
        platform: Option<&dyn PlatformBackend>,
    ) {
        // Gestione avanzamento countdown non-bloccante di cattura finestra
        let dt = ui.input(|i| i.stable_dt);
        if state.window_capture.is_some() {
            ui.ctx().request_repaint();
            if let Some(target_slot) = state.tick_window_capture(dt) {
                Self::perform_window_capture(state, target_slot, platform);
            }
        }

        let slot = match state.selected_slot {
            Some(s) => s,
            None => {
                Self::show_empty_placeholder(ui);
                return;
            }
        };

        ui.vertical(|ui| {
            // Intestazione dello Slot
            Self::show_slot_header(ui, slot, state);

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Proprietà Generali: Etichetta e Selettore Colore
            Self::show_general_properties(ui, slot, state);

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Selettore del Tipo di Binding
            Self::show_binding_selector(ui, slot, state);

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Configurazione Specifica dell'Azione selezionata
            Self::show_action_specific_editor(ui, slot, state, platform);
        });
    }

    /// Placeholder quando nessuno slot è selezionato
    fn show_empty_placeholder(ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(
                egui::RichText::new("👈 Nessuno Slot Selezionato")
                    .font(FontId::proportional(16.0))
                    .color(Color32::from_rgb(161, 161, 170)),
            );
            ui.add_space(10.0);
            ui.label(
                egui::RichText::new(
                    "Clicca su un tasto della griglia 4x3 o su un settore degli encoder per configurarne l'azione.",
                )
                .color(Color32::from_rgb(113, 113, 122)),
            );
        });
    }

    /// Intestazione informativa per lo slot attivo
    fn show_slot_header(ui: &mut Ui, slot: SlotId, state: &UiState) {
        let (title, subtitle) = match slot.control {
            ControlId::Button { row, col } => (
                format!("Tasto [R{}, C{}]", row + 1, col + 1),
                format!("Layer {} • Posizione Griglia", slot.layer.display_number()),
            ),
            ControlId::Encoder { index, event } => {
                let ev_str = match event {
                    EncoderEvent::Ccw => "Rotazione Antioraria ↺ CCW",
                    EncoderEvent::Press => "Click Pressione ● PRESS",
                    EncoderEvent::Cw => "Rotazione Oraria ↻ CW",
                };
                (
                    format!("Encoder {} ({})", index + 1, ev_str),
                    format!("Layer {} • Controllo Manopola", slot.layer.display_number()),
                )
            }
        };

        ui.horizontal(|ui| {
            let badge_text = state.get_badge_for_slot(slot);
            let badge_bg = badge_color(badge_text);

            let (pill_rect, _) = ui.allocate_exact_size(vec2(32.0, 18.0), Sense::hover());
            ui.painter().rect_filled(pill_rect, Rounding::same(4.0), badge_bg);
            ui.painter().text(
                pill_rect.center(),
                Align2::CENTER_CENTER,
                badge_text,
                FontId::proportional(10.0),
                Color32::WHITE,
            );

            ui.vertical(|ui| {
                ui.label(egui::RichText::new(title).font(FontId::proportional(14.0)).strong());
                ui.label(
                    egui::RichText::new(subtitle)
                        .font(FontId::proportional(11.0))
                        .color(Color32::from_rgb(161, 161, 170)),
                );
            });
        });

        // Trigger Phantom assegnato
        if let Some(token) = state.get_trigger_token_for_slot(slot) {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Trigger OS:")
                        .font(FontId::proportional(11.0))
                        .color(Color32::from_rgb(161, 161, 170)),
                );
                ui.label(
                    egui::RichText::new(token)
                        .font(FontId::monospace(11.0))
                        .color(Color32::from_rgb(56, 189, 248)),
                );
            });
        }
    }

    /// Sezione Etichetta e Selettore Colore
    fn show_general_properties(ui: &mut Ui, slot: SlotId, state: &mut UiState) {
        ui.label(egui::RichText::new("PROPRIETÀ GENERALI").font(FontId::proportional(11.0)).strong());
        ui.add_space(4.0);

        let mut marked_dirty = false;
        let idx = slot.global_index();
        if let Some(config) = state.profile.slots.get_mut(&idx) {
            // Etichetta testuale
            ui.horizontal(|ui| {
                ui.label("Etichetta:");
                let text_edit = TextEdit::singleline(&mut config.label).desired_width(180.0);
                if ui.add(text_edit).changed() {
                    marked_dirty = true;
                }
            });

            ui.add_space(4.0);

            // Selettore Colore
            ui.horizontal(|ui| {
                ui.label("Colore:");

                // Palette preset swatches
                let swatches = [
                    (None, "Reset"),
                    (Some("#EF4444"), "Rosso"),
                    (Some("#10B981"), "Verde"),
                    (Some("#3B82F6"), "Blu"),
                    (Some("#F59E0B"), "Ambra"),
                    (Some("#8B5CF6"), "Viola"),
                    (Some("#06B6D4"), "Ciano"),
                    (Some("#F97316"), "Arancio"),
                ];

                for (hex_opt, name) in swatches {
                    let swatch_col = match hex_opt {
                        Some(hex) => hex_to_color32(hex).unwrap_or(Color32::GRAY),
                        None => Color32::from_rgb(45, 45, 50),
                    };

                    let is_current = config.color_hex.as_deref() == hex_opt;
                    let (swatch_rect, resp) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::click());

                    let stroke = if is_current {
                        Stroke::new(2.0_f32, Color32::WHITE)
                    } else if resp.hovered() {
                        Stroke::new(1.0_f32, Color32::LIGHT_GRAY)
                    } else {
                        Stroke::new(1.0_f32, Color32::DARK_GRAY)
                    };

                    ui.painter().rect_filled(swatch_rect, Rounding::same(3.0), swatch_col);
                    ui.painter().rect_stroke(swatch_rect, Rounding::same(3.0), stroke);

                    let resp = resp.on_hover_text(name);
                    if resp.clicked() {
                        config.color_hex = hex_opt.map(|s| s.to_string());
                        marked_dirty = true;
                    }
                }
            });

            // Campo HEX manuale
            let mut hex_val = config.color_hex.clone().unwrap_or_default();
            ui.horizontal(|ui| {
                ui.label("HEX (#RRGGBB):");
                let edit = TextEdit::singleline(&mut hex_val).desired_width(80.0);
                if ui.add(edit).changed() {
                    if hex_val.is_empty() {
                        config.color_hex = None;
                        marked_dirty = true;
                    } else if hex_to_color32(&hex_val).is_some() {
                        config.color_hex = Some(hex_val);
                        marked_dirty = true;
                    }
                }
            });
        }

        if marked_dirty {
            state.mark_dirty();
        }
    }

    /// Selettore Tipo di Binding
    fn show_binding_selector(ui: &mut Ui, slot: SlotId, state: &mut UiState) {
        ui.label(egui::RichText::new("TIPO DI BINDING").font(FontId::proportional(11.0)).strong());
        ui.add_space(4.0);

        let current_type_name = state.get_badge_for_slot(slot);
        let current_desc = match current_type_name {
            "DIS" => "Disabled",
            "HID" => "HidNative",
            "WIN" => "LaunchOrFocus",
            "WSP" => "RestoreWorkspace",
            "MAC" => "MacroSequence",
            "CMD" => "ExecProcess",
            "CTX" => "ContextDispatch",
            _ => "Disabled",
        };

        let binding_types = [
            ("Disabled", "Disabilitato"),
            ("HidNative", "HID Nativo (Tasti / Media / Mouse)"),
            ("LaunchOrFocus", "Finestra (Launch or Focus + QuickSnap)"),
            ("RestoreWorkspace", "Workspace Multi-Finestra"),
            ("MacroSequence", "Sequenza Macro"),
            ("ExecProcess", "Esegui Processo Esterno"),
            ("ContextDispatch", "Dispatch Contestuale"),
        ];

        let mut selected_type = current_desc.to_string();

        ComboBox::from_id_salt("binding_type_combo")
            .selected_text(
                binding_types
                    .iter()
                    .find(|(id, _)| *id == current_desc)
                    .map(|(_, desc)| *desc)
                    .unwrap_or(current_desc),
            )
            .show_ui(ui, |ui| {
                for (id, desc) in binding_types {
                    if ui.selectable_value(&mut selected_type, id.to_string(), desc).clicked() {
                        state.set_selected_binding_type(id);
                    }
                }
            });
    }

    /// Sezione editor specifica in base all'azione impostata
    fn show_action_specific_editor(
        ui: &mut Ui,
        slot: SlotId,
        state: &mut UiState,
        platform: Option<&dyn PlatformBackend>,
    ) {
        let badge = state.get_badge_for_slot(slot);
        match badge {
            "DIS" => {
                ui.label(
                    egui::RichText::new("Nessuna azione assegnata a questo slot.")
                        .color(Color32::from_rgb(161, 161, 170)),
                );
            }
            "HID" => {
                let mut marked_dirty = false;
                let idx = slot.global_index();
                if let Some(config) = state.profile.slots.get_mut(&idx) {
                    if let SlotBinding::HidNative(hid) = &mut config.binding {
                        Self::show_hid_native_fields(ui, hid, &mut marked_dirty);
                    }
                }
                if marked_dirty {
                    state.mark_dirty();
                }
            }
            "WIN" => {
                Self::show_launch_or_focus_editor(ui, slot, state, platform);
            }
            "WSP" => {
                Self::show_restore_workspace_editor(ui, slot, state, platform);
            }
            "MAC" => {
                let mut marked_dirty = false;
                if let Some(ActionDefinition::MacroSequence(action)) = state.get_action_for_slot_mut(slot) {
                    Self::show_macro_sequence_fields(ui, action, &mut marked_dirty);
                }
                if marked_dirty {
                    state.mark_dirty();
                }
            }
            "CMD" => {
                let mut marked_dirty = false;
                if let Some(ActionDefinition::ExecProcess(action)) = state.get_action_for_slot_mut(slot) {
                    Self::show_exec_process_fields(ui, action, &mut marked_dirty);
                }
                if marked_dirty {
                    state.mark_dirty();
                }
            }
            "CTX" => {
                Self::show_context_dispatch_editor(ui, slot, state);
            }
            _ => {
                ui.label("Azione personalizzata o non gestita.");
            }
        }
    }

    /// Campi per HID Nativo
    fn show_hid_native_fields(ui: &mut Ui, hid_action: &mut HidAction, marked_dirty: &mut bool) {
        ui.label(egui::RichText::new("CONFIGURAZIONE HID NATIVO").font(FontId::proportional(11.0)).strong());
        ui.add_space(4.0);

        let mut mode = match hid_action {
            HidAction::KeyboardChord { .. } => 0,
            HidAction::ConsumerMedia(_) => 1,
            HidAction::MouseAction(_) => 2,
        };

        ui.horizontal(|ui| {
            if ui.radio_value(&mut mode, 0, "Tastiera").clicked() {
                *hid_action = HidAction::KeyboardChord {
                    modifiers: Vec::new(),
                    key: "f13".into(),
                };
                *marked_dirty = true;
            }
            if ui.radio_value(&mut mode, 1, "Media").clicked() {
                *hid_action = HidAction::ConsumerMedia(MediaToken::PlayPause);
                *marked_dirty = true;
            }
            if ui.radio_value(&mut mode, 2, "Mouse").clicked() {
                *hid_action = HidAction::MouseAction(MouseToken::Click);
                *marked_dirty = true;
            }
        });

        ui.add_space(6.0);

        match hid_action {
            HidAction::KeyboardChord { modifiers, key } => {
                ui.label("Modificatori:");
                ui.horizontal(|ui| {
                    let mut has_ctrl = modifiers.contains(&HidMod::Ctrl);
                    if ui.checkbox(&mut has_ctrl, "Ctrl").changed() {
                        if has_ctrl {
                            modifiers.push(HidMod::Ctrl);
                        } else {
                            modifiers.retain(|m| *m != HidMod::Ctrl);
                        }
                        *marked_dirty = true;
                    }

                    let mut has_shift = modifiers.contains(&HidMod::Shift);
                    if ui.checkbox(&mut has_shift, "Shift").changed() {
                        if has_shift {
                            modifiers.push(HidMod::Shift);
                        } else {
                            modifiers.retain(|m| *m != HidMod::Shift);
                        }
                        *marked_dirty = true;
                    }

                    let mut has_alt = modifiers.contains(&HidMod::Alt);
                    if ui.checkbox(&mut has_alt, "Alt").changed() {
                        if has_alt {
                            modifiers.push(HidMod::Alt);
                        } else {
                            modifiers.retain(|m| *m != HidMod::Alt);
                        }
                        *marked_dirty = true;
                    }

                    let mut has_win = modifiers.contains(&HidMod::Win);
                    if ui.checkbox(&mut has_win, "Win").changed() {
                        if has_win {
                            modifiers.push(HidMod::Win);
                        } else {
                            modifiers.retain(|m| *m != HidMod::Win);
                        }
                        *marked_dirty = true;
                    }
                });

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label("Tasto Base:");
                    if ui.add(TextEdit::singleline(key).desired_width(100.0)).changed() {
                        *marked_dirty = true;
                    }
                });
            }
            HidAction::ConsumerMedia(media_tok) => {
                let media_options = [
                    (MediaToken::VolumeUp, "Alza Volume (VolumeUp)"),
                    (MediaToken::VolumeDown, "Abbassa Volume (VolumeDown)"),
                    (MediaToken::Mute, "Mute"),
                    (MediaToken::PlayPause, "Play / Pause"),
                    (MediaToken::NextSong, "Traccia Successiva (Next)"),
                    (MediaToken::PrevSong, "Traccia Precedente (Prev)"),
                    (MediaToken::Stop, "Stop"),
                ];

                ComboBox::from_id_salt("media_combo")
                    .selected_text(media_tok.yaml_token())
                    .show_ui(ui, |ui| {
                        for (tok, label) in media_options {
                            let is_sel = *media_tok == tok;
                            if ui.selectable_label(is_sel, label).clicked() {
                                *media_tok = tok;
                                *marked_dirty = true;
                            }
                        }
                    });
            }
            HidAction::MouseAction(mouse_tok) => {
                let mouse_options = [
                    (MouseToken::Click, "Click Sinistro"),
                    (MouseToken::RightClick, "Click Destro"),
                    (MouseToken::MiddleClick, "Click Centrale"),
                    (MouseToken::WheelUp, "Rotella Su (WheelUp)"),
                    (MouseToken::WheelDown, "Rotella Giù (WheelDown)"),
                ];

                ComboBox::from_id_salt("mouse_combo")
                    .selected_text(mouse_tok.yaml_token())
                    .show_ui(ui, |ui| {
                        for (tok, label) in mouse_options {
                            let is_sel = *mouse_tok == tok;
                            if ui.selectable_label(is_sel, label).clicked() {
                                *mouse_tok = tok;
                                *marked_dirty = true;
                            }
                        }
                    });
            }
        }
    }

    /// Editor completo per LaunchOrFocus
    fn show_launch_or_focus_editor(
        ui: &mut Ui,
        slot: SlotId,
        state: &mut UiState,
        platform: Option<&dyn PlatformBackend>,
    ) {
        ui.label(egui::RichText::new("GESTIONE FINESTRA (LAUNCH OR FOCUS)").font(FontId::proportional(11.0)).strong());
        ui.add_space(4.0);

        // Sezione Countdown Cattura Finestra
        if let Some(progress) = &state.window_capture {
            let pct = (progress.total_seconds - progress.remaining_seconds) / progress.total_seconds;
            ui.label(
                egui::RichText::new(format!(
                    "⏳ Porta in primo piano la finestra desiderata... ({:.1}s)",
                    progress.remaining_seconds.max(0.0)
                ))
                .color(Color32::from_rgb(250, 204, 21)),
            );
            ui.add(ProgressBar::new(pct).desired_width(ui.available_width()));
            if ui.button("⏹ Annulla Cattura").clicked() {
                state.cancel_window_capture();
            }
            ui.add_space(6.0);
        } else if ui.button("🎯 Cattura Finestra (3s)").on_hover_text("Avvia un countdown di 3 secondi per catturare automaticamente la finestra che porterai in primo piano.").clicked() {
            state.start_window_capture(slot, 3.0);
        }

        ui.add_space(6.0);

        let mut marked_dirty = false;
        let mut test_requested = false;

        if let Some(ActionDefinition::LaunchOrFocus(action)) = state.get_action_for_slot_mut(slot) {
            // Campi Selettore Finestra
            ui.group(|ui| {
                ui.label(egui::RichText::new("Criteri di Selezione").font(FontId::proportional(11.0)).strong());

                ui.horizontal(|ui| {
                    ui.label("Nome Eseguibile (.exe):");
                    let mut exe = action.selector.exe_name.clone().unwrap_or_default();
                    if ui.add(TextEdit::singleline(&mut exe).desired_width(140.0)).changed() {
                        action.selector.exe_name = if exe.trim().is_empty() { None } else { Some(exe) };
                        marked_dirty = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Titolo Finestra (contiene):");
                    let mut title = action.selector.title_contains.clone().unwrap_or_default();
                    if ui.add(TextEdit::singleline(&mut title).desired_width(140.0)).changed() {
                        action.selector.title_contains = if title.trim().is_empty() { None } else { Some(title) };
                        marked_dirty = true;
                    }
                });
            });

            ui.add_space(6.0);

            // Sezione Quick Snap Buttons
            ui.group(|ui| {
                ui.label(egui::RichText::new("Quick Snap Layout").font(FontId::proportional(11.0)).strong());
                ui.horizontal_wrapped(|ui| {
                    if ui.button("◧ Metà SX").on_hover_text("Posiziona sulla metà sinistra dello schermo").clicked() {
                        action.placement = Some(WindowPlacement {
                            monitor_index: None,
                            coordinates: WindowCoordinates::Normalized {
                                x: 0.0,
                                y: 0.0,
                                width: 0.5,
                                height: 1.0,
                            },
                            bring_to_foreground: true,
                        });
                        marked_dirty = true;
                    }

                    if ui.button("◨ Metà DX").on_hover_text("Posiziona sulla metà destra dello schermo").clicked() {
                        action.placement = Some(WindowPlacement {
                            monitor_index: None,
                            coordinates: WindowCoordinates::Normalized {
                                x: 0.5,
                                y: 0.0,
                                width: 0.5,
                                height: 1.0,
                            },
                            bring_to_foreground: true,
                        });
                        marked_dirty = true;
                    }

                    if ui.button("🗖 Schermo Intero").on_hover_text("Massimizza o copre l'intero schermo").clicked() {
                        action.placement = Some(WindowPlacement {
                            monitor_index: None,
                            coordinates: WindowCoordinates::Normalized {
                                x: 0.0,
                                y: 0.0,
                                width: 1.0,
                                height: 1.0,
                            },
                            bring_to_foreground: true,
                        });
                        marked_dirty = true;
                    }

                    if ui.button("◸ Quadrante Alto-SX").on_hover_text("Quadrante in alto a sinistra (50% x 50%)").clicked() {
                        action.placement = Some(WindowPlacement {
                            monitor_index: None,
                            coordinates: WindowCoordinates::Normalized {
                                x: 0.0,
                                y: 0.0,
                                width: 0.5,
                                height: 0.5,
                            },
                            bring_to_foreground: true,
                        });
                        marked_dirty = true;
                    }

                    if ui.button("◿ Quadrante Basso-DX").on_hover_text("Quadrante in basso a destra (50% x 50%)").clicked() {
                        action.placement = Some(WindowPlacement {
                            monitor_index: None,
                            coordinates: WindowCoordinates::Normalized {
                                x: 0.5,
                                y: 0.5,
                                width: 0.5,
                                height: 0.5,
                            },
                            bring_to_foreground: true,
                        });
                        marked_dirty = true;
                    }
                });

                // Riepilogo coordinate attuali
                if let Some(p) = &action.placement {
                    match &p.coordinates {
                        WindowCoordinates::Normalized { x, y, width, height } => {
                            ui.label(
                                egui::RichText::new(format!(
                                    "Snap: Pos({:.0}%, {:.0}%) Dim({:.0}% x {:.0}%)",
                                    x * 100.0,
                                    y * 100.0,
                                    width * 100.0,
                                    height * 100.0
                                ))
                                .font(FontId::proportional(11.0))
                                .color(Color32::from_rgb(161, 161, 170)),
                            );
                        }
                        WindowCoordinates::Absolute { x, y, width, height } => {
                            ui.label(
                                egui::RichText::new(format!("Pixel: ({}, {}) {}x{}px", x, y, width, height))
                                .font(FontId::proportional(11.0))
                                .color(Color32::from_rgb(161, 161, 170)),
                            );
                        }
                    }
                } else {
                    ui.label(
                        egui::RichText::new("Nessun riposizionamento impostato (solo Focus)")
                            .font(FontId::proportional(11.0))
                            .color(Color32::from_rgb(161, 161, 170)),
                    );
                }
            });

            ui.add_space(8.0);

            // Pulsante di Test Azione
            let test_btn = ui.button(
                egui::RichText::new("▶️ Testa Azione Ora")
                    .font(FontId::proportional(13.0))
                    .color(Color32::from_rgb(34, 197, 94)),
            );

            if test_btn.clicked() {
                test_requested = true;
            }
        }

        if marked_dirty {
            state.mark_dirty();
        }

        if test_requested {
            Self::execute_test_action(state, slot, platform);
        }
    }

    /// Esegue il test reale dell'azione `launch_or_place_window`
    fn execute_test_action(state: &mut UiState, slot: SlotId, platform: Option<&dyn PlatformBackend>) {
        let action = match state.get_action_for_slot(slot) {
            Some(ActionDefinition::LaunchOrFocus(a)) => a.clone(),
            _ => return,
        };

        if let Some(p) = platform {
            let default_placement = WindowPlacement {
                monitor_index: None,
                coordinates: WindowCoordinates::Normalized {
                    x: 0.0,
                    y: 0.0,
                    width: 0.5,
                    height: 1.0,
                },
                bring_to_foreground: true,
            };
            let placement_ref = action.placement.as_ref().unwrap_or(&default_placement);

            let launch_tuple = action.launch.as_ref().map(|l| {
                (
                    l.program.as_path(),
                    l.args.as_slice(),
                    l.working_dir.as_deref(),
                )
            });

            match p.windows().launch_or_place_window(
                &action.selector,
                placement_ref,
                launch_tuple,
                action.timeout_ms,
            ) {
                Ok(result) => {
                    let msg = format!(
                        "Esito Test: Posizionamento={:?}, Focus={:?}",
                        result.placement, result.focus
                    );
                    state.set_status(msg, StatusLevel::Success);
                }
                Err(e) => {
                    state.set_status(format!("Errore esecuzione test: {}", e), StatusLevel::Error);
                }
            }
        } else {
            state.set_status(
                "Piattaforma non inizializzata per il test in anteprima.",
                StatusLevel::Warning,
            );
        }
    }

    /// Cattura automatica della finestra in primo piano al termine del countdown
    fn perform_window_capture(
        state: &mut UiState,
        slot: SlotId,
        platform: Option<&dyn PlatformBackend>,
    ) {
        if let Some(p) = platform {
            let current_pid = std::process::id();
            match p.windows().capture_foreground_window(current_pid) {
                Ok(fg) => {
                    if let Some(ActionDefinition::LaunchOrFocus(action)) =
                        state.get_action_for_slot_mut(slot)
                    {
                        action.selector.exe_name = Some(fg.exe_name.clone());
                        action.selector.title_contains = Some(fg.window_title.clone());
                        action.placement = Some(WindowPlacement {
                            monitor_index: None,
                            coordinates: WindowCoordinates::Absolute {
                                x: fg.x,
                                y: fg.y,
                                width: fg.width,
                                height: fg.height,
                            },
                            bring_to_foreground: true,
                        });
                        state.is_dirty = true;
                    }
                    state.set_status(
                        format!("🎯 Catturata finestra: {} - \"{}\"", fg.exe_name, fg.window_title),
                        StatusLevel::Success,
                    );
                }
                Err(e) => {
                    state.set_status(format!("Impossibile catturare finestra: {}", e), StatusLevel::Error);
                }
            }
        } else {
            state.set_status(
                "Piattaforma non disponibile per la cattura finestra.",
                StatusLevel::Warning,
            );
        }
    }

    /// Editor per RestoreWorkspace
    fn show_restore_workspace_editor(
        ui: &mut Ui,
        slot: SlotId,
        state: &mut UiState,
        platform: Option<&dyn PlatformBackend>,
    ) {
        ui.label(egui::RichText::new("WORKSPACE MULTI-FINESTRA").font(FontId::proportional(11.0)).strong());
        ui.add_space(4.0);

        let mut marked_dirty = false;
        let mut capture_clicked = false;
        let mut remove_idx = None;

        if let Some(ActionDefinition::RestoreWorkspace(action)) = state.get_action_for_slot_mut(slot) {
            ui.horizontal(|ui| {
                ui.label(format!("Finestre nel Workspace: {}", action.targets.len()));
                if ui.button("+ Cattura Finestra Corrente").clicked() {
                    capture_clicked = true;
                }
            });

            // Elenco target correnti
            for (i, target) in action.targets.iter().enumerate() {
                ui.horizontal(|ui| {
                    let desc = target
                        .selector
                        .exe_name
                        .as_deref()
                        .unwrap_or("App sconosciuta");
                    ui.label(format!("{}. {}", i + 1, desc));
                    if ui.button("❌").clicked() {
                        remove_idx = Some(i);
                    }
                });
            }

            if let Some(idx) = remove_idx {
                action.targets.remove(idx);
                marked_dirty = true;
            }
        }

        if marked_dirty {
            state.mark_dirty();
        }

        if capture_clicked {
            if let Some(p) = platform {
                if let Ok(fg) = p.windows().capture_foreground_window(std::process::id()) {
                    if let Some(ActionDefinition::RestoreWorkspace(action)) = state.get_action_for_slot_mut(slot) {
                        action.targets.push(WorkspaceTarget {
                            selector: WindowSelector {
                                exe_name: Some(fg.exe_name),
                                title_contains: Some(fg.window_title),
                                class_name: None,
                            },
                            placement: WindowPlacement {
                                monitor_index: None,
                                coordinates: WindowCoordinates::Absolute {
                                    x: fg.x,
                                    y: fg.y,
                                    width: fg.width,
                                    height: fg.height,
                                },
                                bring_to_foreground: true,
                            },
                            launch: None,
                        });
                        state.mark_dirty();
                    }
                }
            }
        }
    }

    /// Campi per MacroSequence
    fn show_macro_sequence_fields(ui: &mut Ui, action: &mut MacroSequenceAction, marked_dirty: &mut bool) {
        ui.label(egui::RichText::new("SEQUENZA MACRO").font(FontId::proportional(11.0)).strong());
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label("Concorrenza:");
            let mut pol = match action.concurrency {
                MacroConcurrencyPolicy::IgnoreIfRunning => 0,
                MacroConcurrencyPolicy::RestartCurrent => 1,
                MacroConcurrencyPolicy::QueueBounded(_) => 2,
            };
            if ui.radio_value(&mut pol, 0, "Ignora").clicked() {
                action.concurrency = MacroConcurrencyPolicy::IgnoreIfRunning;
                *marked_dirty = true;
            }
            if ui.radio_value(&mut pol, 1, "Riavvia").clicked() {
                action.concurrency = MacroConcurrencyPolicy::RestartCurrent;
                *marked_dirty = true;
            }
        });

        ui.add_space(4.0);
        ui.label(format!("Passaggi (Steps): {}", action.steps.len()));

        ui.horizontal(|ui| {
            if ui.button("+ KeyTap").clicked() {
                action.steps.push(MacroStep::KeyTap { key: "enter".into() });
                *marked_dirty = true;
            }
            if ui.button("+ Delay 100ms").clicked() {
                action.steps.push(MacroStep::DelayMs(100));
                *marked_dirty = true;
            }
            if ui.button("+ Digita Testo").clicked() {
                action.steps.push(MacroStep::TypeText { text: "Hello".into() });
                *marked_dirty = true;
            }
        });

        let mut remove_idx = None;
        for (i, step) in action.steps.iter().enumerate() {
            ui.horizontal(|ui| {
                let desc = match step {
                    MacroStep::KeyTap { key } => format!("Tap '{}'", key),
                    MacroStep::KeyChord { modifiers, key } => {
                        format!("Chord {:?}+{}", modifiers, key)
                    }
                    MacroStep::TypeText { text } => format!("Type \"{}\"", text),
                    MacroStep::ClipboardPaste { text, .. } => format!("Paste \"{}\"", text),
                    MacroStep::DelayMs(ms) => format!("Delay {}ms", ms),
                    MacroStep::WaitForWindow { .. } => "Wait For Window".into(),
                };
                ui.label(format!("{}. {}", i + 1, desc));
                if ui.button("❌").clicked() {
                    remove_idx = Some(i);
                }
            });
        }

        if let Some(idx) = remove_idx {
            action.steps.remove(idx);
            *marked_dirty = true;
        }
    }

    /// Campi per ExecProcess
    fn show_exec_process_fields(ui: &mut Ui, action: &mut ExecProcessAction, marked_dirty: &mut bool) {
        ui.label(egui::RichText::new("ESEGUI PROCESSO ESTERNO").font(FontId::proportional(11.0)).strong());
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label("Eseguibile:");
            let mut prog_str = action.program.to_string_lossy().to_string();
            if ui.add(TextEdit::singleline(&mut prog_str).desired_width(180.0)).changed() {
                action.program = PathBuf::from(prog_str);
                *marked_dirty = true;
            }
        });

        ui.horizontal(|ui| {
            if ui.checkbox(&mut action.hide_console_window, "Nascondi Finestra Console").changed() {
                *marked_dirty = true;
            }
        });
    }

    /// Editor per ContextDispatch
    fn show_context_dispatch_editor(ui: &mut Ui, slot: SlotId, state: &mut UiState) {
        ui.label(egui::RichText::new("DISPATCH CONTESTUALE").font(FontId::proportional(11.0)).strong());
        ui.add_space(4.0);

        let mut marked_dirty = false;
        let mut add_rule = false;
        let mut remove_idx = None;

        if let Some(ActionDefinition::ContextDispatch(action)) = state.get_action_for_slot_mut(slot) {
            ui.label(format!("Regole definite: {}", action.rules.len()));
            if ui.button("+ Aggiungi Regola per App").clicked() {
                add_rule = true;
            }

            for (i, rule) in action.rules.iter_mut().enumerate() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("Regola {}:", i + 1));
                        let mut exe = rule.match_exe_name.clone().unwrap_or_default();
                        if ui.add(TextEdit::singleline(&mut exe).desired_width(120.0)).changed() {
                            rule.match_exe_name = if exe.is_empty() { None } else { Some(exe) };
                            marked_dirty = true;
                        }
                        if ui.button("❌").clicked() {
                            remove_idx = Some(i);
                        }
                    });
                });
            }

            if let Some(idx) = remove_idx {
                action.rules.remove(idx);
                marked_dirty = true;
            }
        }

        if add_rule {
            let target_action = ActionId::new_v4();
            if let Some(ActionDefinition::ContextDispatch(action)) = state.get_action_for_slot_mut(slot) {
                action.rules.push(ContextRule {
                    match_exe_name: Some("chrome.exe".into()),
                    match_title_contains: None,
                    target_action_id: target_action,
                });
                marked_dirty = true;
            }
        }

        if marked_dirty {
            state.mark_dirty();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspector_instantiation() {
        let inspector = InspectorPanel::new();
        let _ = inspector;
    }
}
