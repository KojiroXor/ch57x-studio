//! Contextual slot inspector widget in Apple Liquid Glass style
//!
//! Organizzato in 3 Glass Cards distinte con allineamento Pixel-Perfect a 120px:
//! - Card 1 (Info Slot): Identificatore, Etichetta (con autofocus/placeholder) e Colore pastello Apple.
//! - Card 2 (Comportamento & Tipo): Selettore tipo azione (zero emoji), editor configurazione specifica con egui::Grid.
//! - Card 3 (Test Rapido & Trigger): Anteprima trigger hardware associato (F13..F24) e pulsante di test azione.

use crate::liquid_glass::{liquid_glass_card, liquid_pill_button, liquid_segmented_control};
use crate::pad_canvas::badge_color;
use crate::state::{hex_to_color32, StatusLevel, UiState};
use crate::theme::*;
use ch57x_domain::actions::{
    ActionDefinition, ExecProcessAction, HidAction, HidMod, MacroConcurrencyPolicy,
    MacroSequenceAction, MediaToken, MouseToken, SlotBinding, WindowCoordinates, WindowPlacement,
    WindowSelector, WorkspaceTarget,
};
use ch57x_domain::hardware::{ControlId, EncoderEvent, SlotId};
use ch57x_platform::PlatformBackend;
use egui::{
    vec2, Color32, ComboBox, FontId, Margin, ProgressBar, Rounding, Sense, Stroke, TextEdit, Ui,
};

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
                Self::show_empty_placeholder(ui, state);
                return;
            }
        };

        ui.vertical(|ui| {
            // Card 1 — Info Slot
            liquid_glass_card(ui, Some("Info Slot"), |ui| {
                Self::show_slot_info_card(ui, slot, state);
            });

            ui.add_space(8.0);

            // Card 2 — Comportamento & Tipo
            liquid_glass_card(ui, Some("Comportamento & Tipo"), |ui| {
                Self::show_behavior_and_type_card(ui, slot, state, platform);
            });

            ui.add_space(8.0);

            // Card 3 — Test Rapido & Trigger
            liquid_glass_card(ui, Some("Test Rapido & Trigger"), |ui| {
                Self::show_test_and_trigger_card(ui, slot, state, platform);
            });

            ui.add_space(8.0);

            // Card 4 — Template Layer Intero
            liquid_glass_card(ui, Some("Template Intero Layer (18 Slot)"), |ui| {
                Self::show_layer_templates_section(ui, state);
            });
        });
    }

    /// Placeholder quando nessuno slot è selezionato
    fn show_empty_placeholder(ui: &mut Ui, state: &mut UiState) {
        liquid_glass_card(ui, None, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                draw_status_dot(ui, egui::pos2(ui.available_rect_before_wrap().center().x, ui.cursor().top() + 10.0), COLOR_TEXT_TERTIARY, "");
                ui.add_space(14.0);
                ui.label(
                    egui::RichText::new("Nessun Controllo Selezionato")
                        .font(FontId::proportional(15.0))
                        .color(COLOR_TEXT_PRIMARY)
                        .strong(),
                );
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(
                        "Seleziona un tasto o un encoder sul visualizzatore hardware a sinistra per configurarne il comportamento.",
                    )
                    .font(FontId::proportional(12.0))
                    .color(COLOR_TEXT_SECONDARY),
                );
                ui.add_space(20.0);
            });

            ui.separator();
            ui.add_space(10.0);

            ui.label(
                egui::RichText::new("TEMPLATE 1-CLICK PER INTERO LAYER")
                    .font(FontId::proportional(11.5))
                    .strong()
                    .color(COLOR_APPLE_BLUE),
            );
            ui.add_space(6.0);
            Self::show_layer_templates_section(ui, state);
        });
    }

    /// Sezione Template Intero Layer (con gestione conferma sovrascrittura)
    fn show_layer_templates_section(ui: &mut Ui, state: &mut UiState) {
        ui.label(
            egui::RichText::new(format!(
                "Configura tutti i 18 slot del Layer {} in un colpo solo:",
                state.editing_layer.display_number()
            ))
            .font(FontId::proportional(11.0))
            .color(COLOR_TEXT_SECONDARY),
        );
        ui.add_space(6.0);

        for template in crate::presets::LAYER_TEMPLATES {
            let is_pending = state.pending_layer_template_confirm == Some(template.id);

            egui::Frame {
                inner_margin: Margin::symmetric(10.0, 8.0),
                rounding: Rounding::same(8.0),
                fill: COLOR_SURFACE_INSET,
                stroke: Stroke::new(
                    1.0_f32,
                    if is_pending {
                        COLOR_APPLE_ORANGE
                    } else {
                        COLOR_BORDER_SUBTLE
                    },
                ),
                ..Default::default()
            }
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(template.title)
                            .strong()
                            .color(COLOR_TEXT_PRIMARY),
                    );
                });
                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new(template.description)
                        .font(FontId::proportional(10.5))
                        .color(COLOR_TEXT_TERTIARY),
                );
                ui.add_space(6.0);

                if is_pending {
                    ui.label(
                        egui::RichText::new(format!(
                            "⚠️ Il Layer {} contiene slot configurati. Vuoi sovrascrivere tutti i 18 slot con questo template?",
                            state.editing_layer.display_number()
                        ))
                        .font(FontId::proportional(11.0))
                        .color(COLOR_APPLE_ORANGE),
                    );
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let confirm_btn = egui::Button::new(
                            egui::RichText::new("Conferma Sovrascrittura")
                                .color(Color32::WHITE)
                                .font(FontId::proportional(11.0))
                                .strong(),
                        )
                        .fill(COLOR_APPLE_RED);

                        if ui.add(confirm_btn).clicked() {
                            (template.apply)(&mut state.profile, state.editing_layer);
                            let _ = ch57x_compiler::allocator::allocate_triggers(&mut state.profile);
                            state.mark_dirty();
                            state.pending_layer_template_confirm = None;
                            state.set_status(
                                format!(
                                    "Template '{}' applicato con successo al Layer {}.",
                                    template.title,
                                    state.editing_layer.display_number()
                                ),
                                StatusLevel::Success,
                            );
                        }

                        if ui
                            .button(egui::RichText::new("Annulla").font(FontId::proportional(11.0)))
                            .clicked()
                        {
                            state.pending_layer_template_confirm = None;
                        }
                    });
                } else {
                    let apply_btn = egui::Button::new(
                        egui::RichText::new(format!(
                            "Applica al Layer {}",
                            state.editing_layer.display_number()
                        ))
                        .font(FontId::proportional(11.0))
                        .color(COLOR_APPLE_BLUE),
                    )
                    .fill(COLOR_SURFACE_CARD)
                    .stroke(Stroke::new(1.0_f32, COLOR_APPLE_BLUE));

                    if ui.add(apply_btn).clicked() {
                        if state.is_layer_configured(state.editing_layer) {
                            state.pending_layer_template_confirm = Some(template.id);
                        } else {
                            (template.apply)(&mut state.profile, state.editing_layer);
                            let _ = ch57x_compiler::allocator::allocate_triggers(&mut state.profile);
                            state.mark_dirty();
                            state.set_status(
                                format!(
                                    "Template '{}' applicato con successo al Layer {}.",
                                    template.title,
                                    state.editing_layer.display_number()
                                ),
                                StatusLevel::Success,
                            );
                        }
                    }
                }
            });
            ui.add_space(6.0);
        }
    }

    /// Card 1: Info Slot (Identificatore, Etichetta, Colore con griglia fissa a 120px)
    fn show_slot_info_card(ui: &mut Ui, slot: SlotId, state: &mut UiState) {
        let (ident_title, ident_subtitle) = match slot.control {
            ControlId::Button { row, col } => (
                format!("Tasto [R{}, C{}]", row + 1, col + 1),
                format!("Layer {} - Matrice Tasti", slot.layer.display_number()),
            ),
            ControlId::Encoder { index, event } => {
                let ev_str = match event {
                    EncoderEvent::Ccw => "Rotazione Sinistra (CCW)",
                    EncoderEvent::Press => "Click Pressione (Push)",
                    EncoderEvent::Cw => "Rotazione Destra (CW)",
                };
                (
                    format!("Encoder {} - {}", index + 1, ev_str),
                    format!("Layer {} - Manopola Rotativa", slot.layer.display_number()),
                )
            }
        };

        let mut marked_dirty = false;
        let idx = slot.global_index();

        egui::Grid::new("slot_info_grid")
            .num_columns(2)
            .min_col_width(120.0)
            .spacing(vec2(12.0, 8.0))
            .show(ui, |ui| {
                // 1. Identificatore
                ui.label(egui::RichText::new("Identificatore:").color(COLOR_TEXT_SECONDARY));
                ui.horizontal(|ui| {
                    let badge_text = state.get_badge_for_slot(slot);
                    draw_pill_badge(ui, badge_text, badge_color(badge_text), Color32::WHITE);
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(ident_title)
                            .strong()
                            .color(COLOR_TEXT_PRIMARY),
                    );
                    ui.label(
                        egui::RichText::new(format!("- {}", ident_subtitle))
                            .font(FontId::proportional(11.0))
                            .color(COLOR_TEXT_TERTIARY),
                    );
                });
                ui.end_row();

                // 2. Etichetta
                ui.label(egui::RichText::new("Etichetta:").color(COLOR_TEXT_SECONDARY));
                if let Some(config) = state.profile.slots.get_mut(&idx) {
                    let text_edit = TextEdit::singleline(&mut config.label)
                        .hint_text("Nome o descrizione azione...")
                        .desired_width(190.0);
                    if ui.add(text_edit).changed() {
                        marked_dirty = true;
                    }
                } else {
                    ui.label("-");
                }
                ui.end_row();

                // 3. Colore Tasto
                ui.label(egui::RichText::new("Colore Tasto:").color(COLOR_TEXT_SECONDARY));
                if let Some(config) = state.profile.slots.get_mut(&idx) {
                    ui.horizontal_wrapped(|ui| {
                        let swatches = [
                            (None, "Default"),
                            (Some("#30D158"), "Verde Apple"),
                            (Some("#0A84FF"), "Blu Apple"),
                            (Some("#FF9F0A"), "Arancio"),
                            (Some("#FF453A"), "Rosso"),
                            (Some("#BF5AF2"), "Viola"),
                            (Some("#64D2FF"), "Azzurro"),
                            (Some("#FFD60A"), "Giallo"),
                        ];

                        for (hex_opt, name) in swatches {
                            let swatch_col = match hex_opt {
                                Some(hex) => hex_to_color32(hex).unwrap_or(Color32::GRAY),
                                None => Color32::from_rgb(45, 45, 50),
                            };

                            let is_current = config.color_hex.as_deref() == hex_opt;
                            let (swatch_rect, resp) =
                                ui.allocate_exact_size(vec2(18.0, 18.0), Sense::click());

                            let stroke = if is_current {
                                Stroke::new(2.0_f32, Color32::WHITE)
                            } else if resp.hovered() {
                                Stroke::new(1.0_f32, COLOR_BORDER_STRONG)
                            } else {
                                Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE)
                            };

                            ui.painter()
                                .rect_filled(swatch_rect, Rounding::same(5.0), swatch_col);
                            ui.painter()
                                .rect_stroke(swatch_rect, Rounding::same(5.0), stroke);

                            if resp.on_hover_text(name).clicked() {
                                config.color_hex = hex_opt.map(|s| s.to_string());
                                marked_dirty = true;
                            }
                        }
                    });
                } else {
                    ui.label("-");
                }
                ui.end_row();
            });

        if marked_dirty {
            state.mark_dirty();
        }
    }

    /// Card 2: Comportamento & Tipo (Picker tipo azione + configurazione specifica con egui::Grid)
    fn show_behavior_and_type_card(
        ui: &mut Ui,
        slot: SlotId,
        state: &mut UiState,
        platform: Option<&dyn PlatformBackend>,
    ) {
        let current_badge = state.get_badge_for_slot(slot);

        let mut category = match current_badge {
            "DIS" => 0,
            "HID" => 1,
            _ => 2,
        };

        let options = [
            (0, "Disabilitato"),
            (1, "HID Nativo"),
            (2, "Automazione Host"),
        ];

        if let ControlId::Encoder { index, .. } = slot.control {
            ui.label(
                egui::RichText::new(format!(
                    "PRESET 3-IN-1 PER ENCODER {} (CCW + PRESS + CW)",
                    index + 1
                ))
                .font(FontId::proportional(11.0))
                .strong()
                .color(COLOR_APPLE_BLUE),
            );
            ui.add_space(4.0);

            ui.horizontal_wrapped(|ui| {
                for preset in crate::presets::ENCODER_PRESETS {
                    let btn = egui::Button::new(
                        egui::RichText::new(preset.title)
                            .font(FontId::proportional(11.0))
                            .color(COLOR_TEXT_PRIMARY),
                    )
                    .fill(COLOR_SURFACE_CARD)
                    .stroke(Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE));

                    if ui.add(btn).on_hover_text(preset.description).clicked() {
                        crate::presets::apply_encoder_preset(
                            &mut state.profile,
                            slot.layer,
                            index,
                            preset,
                        );
                        let _ = ch57x_compiler::allocator::allocate_triggers(&mut state.profile);
                        state.mark_dirty();
                        state.set_status(
                            format!(
                                "Encoder {} configurato su preset '{}'.",
                                index + 1,
                                preset.title
                            ),
                            StatusLevel::Success,
                        );
                    }
                }
            });
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);
        }

        if let ControlId::Button { .. } = slot.control {
            ui.label(
                egui::RichText::new("PRESET RAPIDI PRODUTTIVITÀ")
                    .font(FontId::proportional(11.0))
                    .strong()
                    .color(COLOR_APPLE_BLUE),
            );
            ui.add_space(4.0);

            ui.horizontal_wrapped(|ui| {
                for preset in crate::presets::KEY_PRESETS {
                    let chip = egui::Button::new(
                        egui::RichText::new(preset.title)
                            .font(FontId::proportional(10.5))
                            .color(COLOR_TEXT_PRIMARY),
                    )
                    .fill(COLOR_SURFACE_CARD)
                    .stroke(Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE));

                    if ui.add(chip).on_hover_text(preset.description).clicked() {
                        (preset.apply)(&mut state.profile, slot.global_index());
                        let _ = ch57x_compiler::allocator::allocate_triggers(&mut state.profile);
                        state.mark_dirty();
                        state.set_status(
                            format!("Tasto configurato con preset '{}'.", preset.title),
                            StatusLevel::Success,
                        );
                    }
                }
            });
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);
        }

        if liquid_segmented_control(ui, "binding_category_control", &mut category, &options) {
            match category {
                0 => state.set_selected_binding_type("Disabled"),
                1 => state.set_selected_binding_type("HidNative"),
                2 if current_badge == "DIS" || current_badge == "HID" => {
                    state.set_selected_binding_type("LaunchOrFocus");
                }
                _ => {}
            }
        }

        if category == 2 {
            ui.add_space(8.0);
            egui::Grid::new("host_action_type_grid")
                .num_columns(2)
                .min_col_width(120.0)
                .spacing(vec2(12.0, 8.0))
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Azione Host:").color(COLOR_TEXT_SECONDARY));

                    let host_types = [
                        ("LaunchOrFocus", "Finestra"),
                        ("RestoreWorkspace", "Workspace"),
                        ("MacroSequence", "Macro"),
                        ("ExecProcess", "Processo"),
                        ("ContextDispatch", "Contestuale"),
                        ("AdjustBrightness", "Luminosità"),
                    ];

                    let current_type_name = match current_badge {
                        "WIN" => "LaunchOrFocus",
                        "WSP" => "RestoreWorkspace",
                        "MAC" => "MacroSequence",
                        "CMD" => "ExecProcess",
                        "CTX" => "ContextDispatch",
                        "LUM" => "AdjustBrightness",
                        _ => "LaunchOrFocus",
                    };

                    let mut sel_host = current_type_name.to_string();
                    ComboBox::from_id_salt("host_action_combo")
                        .selected_text(
                            host_types
                                .iter()
                                .find(|(id, _)| *id == current_type_name)
                                .map(|(_, desc)| *desc)
                                .unwrap_or(current_type_name),
                        )
                        .show_ui(ui, |ui| {
                            for (id, desc) in host_types {
                                if ui
                                    .selectable_value(&mut sel_host, id.to_string(), desc)
                                    .clicked()
                                {
                                    state.set_selected_binding_type(id);
                                }
                            }
                        });
                    ui.end_row();
                });
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(8.0);

        Self::show_action_specific_editor(ui, slot, state, platform);
    }

    /// Card 3: Test Rapido & Trigger (Trigger hardware e pulsante test con griglia fissa a 120px)
    fn show_test_and_trigger_card(
        ui: &mut Ui,
        slot: SlotId,
        state: &mut UiState,
        platform: Option<&dyn PlatformBackend>,
    ) {
        let mut test_requested = false;

        egui::Grid::new("test_and_trigger_grid")
            .num_columns(2)
            .min_col_width(120.0)
            .spacing(vec2(12.0, 8.0))
            .show(ui, |ui| {
                // Trigger Hardware
                ui.label(egui::RichText::new("Trigger Hardware:").color(COLOR_TEXT_SECONDARY));
                if let Some(token) = state.get_trigger_token_for_slot(slot) {
                    draw_pill_badge(ui, &token, Color32::from_rgb(18, 44, 75), COLOR_APPLE_BLUE);
                } else {
                    draw_pill_badge(
                        ui,
                        "Nessun trigger (Nativo o Disabilitato)",
                        Color32::from_rgb(34, 34, 38),
                        COLOR_TEXT_TERTIARY,
                    );
                }
                ui.end_row();

                // Verifica Azione
                ui.label(egui::RichText::new("Verifica Azione:").color(COLOR_TEXT_SECONDARY));
                let test_btn = liquid_pill_button(ui, "Testa Azione Ora", true);
                if test_btn.clicked() {
                    test_requested = true;
                }
                ui.end_row();
            });

        if test_requested {
            Self::execute_test_action(state, slot, platform);
        }
    }

    /// Configurazione specifica per tipologia di azione
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
                    egui::RichText::new(
                        "Controllo disattivato: la pressione del tasto non invierà alcun segnale.",
                    )
                    .font(FontId::proportional(11.5))
                    .color(COLOR_TEXT_SECONDARY),
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
                Self::show_launch_or_focus_editor(ui, slot, state);
            }
            "WSP" => {
                Self::show_restore_workspace_editor(ui, slot, state, platform);
            }
            "MAC" => {
                let mut marked_dirty = false;
                if let Some(ActionDefinition::MacroSequence(action)) =
                    state.get_action_for_slot_mut(slot)
                {
                    Self::show_macro_sequence_fields(ui, action, &mut marked_dirty);
                }
                if marked_dirty {
                    state.mark_dirty();
                }
            }
            "CMD" => {
                let mut marked_dirty = false;
                if let Some(ActionDefinition::ExecProcess(action)) =
                    state.get_action_for_slot_mut(slot)
                {
                    Self::show_exec_process_fields(ui, action, &mut marked_dirty);
                }
                if marked_dirty {
                    state.mark_dirty();
                }
            }
            "CTX" => {
                Self::show_context_dispatch_editor(ui, slot, state);
            }
            "LUM" => {
                let mut marked_dirty = false;
                if let Some(ActionDefinition::AdjustBrightness { delta }) =
                    state.get_action_for_slot_mut(slot)
                {
                    ui.label(
                        egui::RichText::new(
                            "Regolazione Luminosità Monitor (Surface/Laptop IOCTL & WMI):",
                        )
                        .font(FontId::proportional(11.5))
                        .color(COLOR_TEXT_PRIMARY),
                    );
                    ui.add_space(4.0);

                    egui::Grid::new("lum_action_grid")
                        .num_columns(2)
                        .min_col_width(120.0)
                        .spacing(vec2(12.0, 6.0))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new("Variazione (Delta):")
                                    .color(COLOR_TEXT_SECONDARY),
                            );
                            ui.horizontal(|ui| {
                                let mut val = *delta;
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut val)
                                            .speed(1)
                                            .range(-50..=50)
                                            .suffix("%"),
                                    )
                                    .changed()
                                {
                                    *delta = val;
                                    marked_dirty = true;
                                }
                                if ui.button("+6%").clicked() {
                                    *delta = 6;
                                    marked_dirty = true;
                                }
                                if ui.button("-6%").clicked() {
                                    *delta = -6;
                                    marked_dirty = true;
                                }
                            });
                            ui.end_row();
                        });

                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(
                            "Modifica la luminosità dello schermo ad ogni scatto dell'encoder o pressione del tasto.",
                        )
                        .font(FontId::proportional(10.5))
                        .color(COLOR_TEXT_TERTIARY),
                    );
                }
                if marked_dirty {
                    state.mark_dirty();
                }
            }
            _ => {
                ui.label("Azione personalizzata o non gestita.");
            }
        }
    }

    /// Campi per HID Nativo con egui::Grid
    fn show_hid_native_fields(ui: &mut Ui, hid_action: &mut HidAction, marked_dirty: &mut bool) {
        let mut mode = match hid_action {
            HidAction::KeyboardChord { .. } => 0,
            HidAction::ConsumerMedia(_) => 1,
            HidAction::MouseAction(_) => 2,
        };

        let hid_modes = [
            (0, "Tastiera / Scorciatoia"),
            (1, "Media & Volume"),
            (2, "Mouse & Rotellina"),
        ];

        if liquid_segmented_control(ui, "hid_mode_control", &mut mode, &hid_modes) {
            match mode {
                0 => {
                    *hid_action = HidAction::KeyboardChord {
                        modifiers: Vec::new(),
                        key: "f13".into(),
                    };
                    *marked_dirty = true;
                }
                1 => {
                    *hid_action = HidAction::ConsumerMedia(MediaToken::PlayPause);
                    *marked_dirty = true;
                }
                2 => {
                    *hid_action = HidAction::MouseAction(MouseToken::Click);
                    *marked_dirty = true;
                }
                _ => {}
            }
        }

        ui.add_space(8.0);

        egui::Grid::new("hid_native_grid")
            .num_columns(2)
            .min_col_width(120.0)
            .spacing(vec2(12.0, 6.0))
            .show(ui, |ui| match hid_action {
                HidAction::KeyboardChord { modifiers, key } => {
                    ui.label(egui::RichText::new("Modificatori:").color(COLOR_TEXT_SECONDARY));
                    ui.horizontal(|ui| {
                        for (mod_item, label) in [
                            (HidMod::Ctrl, "Control"),
                            (HidMod::Shift, "Shift"),
                            (HidMod::Alt, "Alt"),
                            (HidMod::Win, "Cmd/Win"),
                        ] {
                            let mut has_mod = modifiers.contains(&mod_item);
                            if ui.checkbox(&mut has_mod, label).changed() {
                                if has_mod {
                                    modifiers.push(mod_item);
                                } else {
                                    modifiers.retain(|m| *m != mod_item);
                                }
                                *marked_dirty = true;
                            }
                        }
                    });
                    ui.end_row();

                    ui.label(egui::RichText::new("Tasto Base:").color(COLOR_TEXT_SECONDARY));
                    ui.horizontal(|ui| {
                        if ui
                            .add(TextEdit::singleline(key).desired_width(120.0))
                            .changed()
                        {
                            *marked_dirty = true;
                        }
                        ui.label(
                            egui::RichText::new("(es. a..z, 1..9, f1..f24, space, enter)")
                                .font(FontId::proportional(11.0))
                                .color(COLOR_TEXT_TERTIARY),
                        );
                    });
                    ui.end_row();
                }
                HidAction::ConsumerMedia(media_tok) => {
                    let media_options = [
                        (MediaToken::VolumeUp, "Alza Volume (Volume Up)"),
                        (MediaToken::VolumeDown, "Abbassa Volume (Volume Down)"),
                        (MediaToken::Mute, "Mute Audio"),
                        (MediaToken::PlayPause, "Play / Pausa"),
                        (MediaToken::NextSong, "Traccia Successiva"),
                        (MediaToken::PrevSong, "Traccia Precedente"),
                        (MediaToken::Stop, "Stop Riproduzione"),
                    ];

                    ui.label(egui::RichText::new("Comando Media:").color(COLOR_TEXT_SECONDARY));
                    ComboBox::from_id_salt("media_combo")
                        .selected_text(media_tok.yaml_token())
                        .show_ui(ui, |ui| {
                            for (tok, label) in media_options {
                                if ui.selectable_value(media_tok, tok, label).clicked() {
                                    *marked_dirty = true;
                                }
                            }
                        });
                    ui.end_row();
                }
                HidAction::MouseAction(mouse_tok) => {
                    let mouse_options = [
                        (MouseToken::Click, "Click Sinistro"),
                        (MouseToken::RightClick, "Click Destro"),
                        (MouseToken::MiddleClick, "Click Centrale"),
                        (MouseToken::WheelUp, "Rotella Su (Scroll Up)"),
                        (MouseToken::WheelDown, "Rotella Giù (Scroll Down)"),
                    ];

                    ui.label(egui::RichText::new("Azione Mouse:").color(COLOR_TEXT_SECONDARY));
                    ComboBox::from_id_salt("mouse_combo")
                        .selected_text(mouse_tok.yaml_token())
                        .show_ui(ui, |ui| {
                            for (tok, label) in mouse_options {
                                if ui.selectable_value(mouse_tok, tok, label).clicked() {
                                    *marked_dirty = true;
                                }
                            }
                        });
                    ui.end_row();
                }
            });
    }

    /// Editor per Gestione Finestra (LaunchOrFocus) con egui::Grid a 120px
    fn show_launch_or_focus_editor(ui: &mut Ui, slot: SlotId, state: &mut UiState) {
        let capture_info = state
            .window_capture
            .as_ref()
            .map(|p| (p.remaining_seconds, p.total_seconds));
        if let Some((remaining, total)) = capture_info {
            let pct = (total - remaining) / total;
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    draw_status_dot(
                        ui,
                        egui::pos2(ui.cursor().min.x + 6.0, ui.cursor().min.y + 10.0),
                        COLOR_APPLE_ORANGE,
                        "",
                    );
                    ui.add_space(14.0);
                    ui.label(
                        egui::RichText::new(format!(
                            "Porta in primo piano l'applicazione desiderata... ({:.1}s)",
                            remaining.max(0.0)
                        ))
                        .color(COLOR_APPLE_ORANGE)
                        .strong(),
                    );
                });
                ui.add_space(4.0);
                ui.add(
                    ProgressBar::new(pct)
                        .desired_width(ui.available_width())
                        .fill(COLOR_APPLE_BLUE),
                );
                ui.add_space(4.0);
                if ui.button("Annulla Cattura").clicked() {
                    state.cancel_window_capture();
                }
            });
            ui.add_space(8.0);
        } else {
            let capture_btn = egui::Button::new(
                egui::RichText::new("Cattura Finestra Attiva (3s)")
                    .color(COLOR_APPLE_BLUE)
                    .strong(),
            )
            .fill(COLOR_SURFACE_CARD)
            .stroke(Stroke::new(1.0_f32, COLOR_APPLE_BLUE));

            if ui.add(capture_btn).on_hover_text("Avvia un timer di 3 secondi per catturare automaticamente la finestra che porterai in primo piano.").clicked() {
                state.start_window_capture(slot, 3.0);
            }
            ui.add_space(8.0);
        }

        let mut marked_dirty = false;

        if let Some(ActionDefinition::LaunchOrFocus(action)) = state.get_action_for_slot_mut(slot) {
            egui::Grid::new("launch_or_focus_grid")
                .num_columns(2)
                .min_col_width(120.0)
                .spacing(vec2(12.0, 6.0))
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Eseguibile:").color(COLOR_TEXT_SECONDARY));
                    let mut exe = action.selector.exe_name.clone().unwrap_or_default();
                    if ui
                        .add(
                            TextEdit::singleline(&mut exe)
                                .hint_text("es. code.exe, chrome.exe")
                                .desired_width(180.0),
                        )
                        .changed()
                    {
                        action.selector.exe_name = if exe.trim().is_empty() {
                            None
                        } else {
                            Some(exe)
                        };
                        marked_dirty = true;
                    }
                    ui.end_row();

                    ui.label(egui::RichText::new("Titolo Finestra:").color(COLOR_TEXT_SECONDARY));
                    let mut title = action.selector.title_contains.clone().unwrap_or_default();
                    if ui
                        .add(
                            TextEdit::singleline(&mut title)
                                .hint_text("es. GitHub, Discord")
                                .desired_width(180.0),
                        )
                        .changed()
                    {
                        action.selector.title_contains = if title.trim().is_empty() {
                            None
                        } else {
                            Some(title)
                        };
                        marked_dirty = true;
                    }
                    ui.end_row();
                });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(8.0);

            // Quick Snap Buttons
            ui.label(
                egui::RichText::new("POSIZIONAMENTO RAPIDO (QUICK SNAP)")
                    .font(FontId::proportional(11.0))
                    .strong()
                    .color(COLOR_TEXT_SECONDARY),
            );
            ui.add_space(4.0);

            ui.horizontal_wrapped(|ui| {
                if ui
                    .button("Metà Sinistra")
                    .on_hover_text("Posiziona sulla metà sinistra dello schermo")
                    .clicked()
                {
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

                if ui
                    .button("Metà Destra")
                    .on_hover_text("Posiziona sulla metà destra dello schermo")
                    .clicked()
                {
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

                if ui
                    .button("Schermo Intero")
                    .on_hover_text("Copre l'intero monitor")
                    .clicked()
                {
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

                if ui
                    .button("Alto Sinistra")
                    .on_hover_text("Quadrante 50% x 50% in alto a sinistra")
                    .clicked()
                {
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

                if ui
                    .button("Basso Destra")
                    .on_hover_text("Quadrante 50% x 50% in basso a destra")
                    .clicked()
                {
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
        }

        if marked_dirty {
            state.mark_dirty();
        }
    }

    /// Esegue il test reale dell'azione
    fn execute_test_action(
        state: &mut UiState,
        slot: SlotId,
        platform: Option<&dyn PlatformBackend>,
    ) {
        let action = state.get_action_for_slot(slot).cloned();

        match action {
            Some(ActionDefinition::LaunchOrFocus(a)) => {
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
                    let placement_ref = a.placement.as_ref().unwrap_or(&default_placement);

                    let launch_tuple = a.launch.as_ref().map(|l| {
                        (
                            l.program.as_path(),
                            l.args.as_slice(),
                            l.working_dir.as_deref(),
                        )
                    });

                    match p.windows().launch_or_place_window(
                        &a.selector,
                        placement_ref,
                        launch_tuple,
                        a.timeout_ms,
                    ) {
                        Ok(result) => {
                            let msg = format!(
                                "Test riuscito: Posizionamento={:?}, Focus={:?}",
                                result.placement, result.focus
                            );
                            state.set_status(msg, StatusLevel::Success);
                        }
                        Err(e) => {
                            state.set_status(
                                format!("Errore esecuzione test: {}", e),
                                StatusLevel::Error,
                            );
                        }
                    }
                } else {
                    state.set_status(
                        "Piattaforma non inizializzata per il test.",
                        StatusLevel::Warning,
                    );
                }
            }
            Some(ActionDefinition::ExecProcess(a)) => {
                let mut cmd = std::process::Command::new(&a.program);
                cmd.args(&a.args);
                if let Some(ref dir) = a.working_dir {
                    cmd.current_dir(dir);
                }
                #[cfg(target_os = "windows")]
                {
                    use std::os::windows::process::CommandExt;
                    cmd.creation_flags(0x08000000);
                }
                match cmd.spawn() {
                    Ok(child) => {
                        state.set_status(
                            format!("Processo avviato con PID {}", child.id()),
                            StatusLevel::Success,
                        );
                    }
                    Err(e) => {
                        state.set_status(
                            format!("Errore avvio processo: {}", e),
                            StatusLevel::Error,
                        );
                    }
                }
            }
            Some(ActionDefinition::RestoreWorkspace(a)) => {
                state.set_status(
                    format!("Test Workspace: {} finestre configurate.", a.targets.len()),
                    StatusLevel::Info,
                );
            }
            Some(ActionDefinition::MacroSequence(a)) => {
                state.set_status(
                    format!(
                        "Test Macro: sequenza con {} step configurati.",
                        a.steps.len()
                    ),
                    StatusLevel::Info,
                );
            }
            Some(ActionDefinition::ContextDispatch(a)) => {
                state.set_status(
                    format!("Test Contestuale: {} regole definite.", a.rules.len()),
                    StatusLevel::Info,
                );
            }
            Some(ActionDefinition::ShowHud { duration_ms }) => {
                state.set_status(
                    format!("Test OSD: visualizzazione per {} ms.", duration_ms),
                    StatusLevel::Info,
                );
            }
            Some(ActionDefinition::AdjustBrightness { delta }) => {
                if let Some(p) = platform {
                    match p.adjust_brightness(delta) {
                        Ok(new_val) => {
                            state.set_status(
                                format!("Luminosità impostata al {}%", new_val),
                                StatusLevel::Success,
                            );
                        }
                        Err(e) => {
                            state.set_status(
                                format!("Errore regolazione luminosità: {}", e),
                                StatusLevel::Error,
                            );
                        }
                    }
                } else {
                    state.set_status(
                        "Piattaforma non inizializzata per il test.",
                        StatusLevel::Warning,
                    );
                }
            }
            None => {
                let badge = state.get_badge_for_slot(slot);
                if badge == "HID" {
                    state.set_status("Azione HID nativa: inviata direttamente dal microcontrollore all'host via USB.", StatusLevel::Info);
                } else {
                    state.set_status(
                        "Nessuna azione configurata su questo slot.",
                        StatusLevel::Warning,
                    );
                }
            }
        }
    }

    /// Cattura automatica della finestra attiva
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
                        format!(
                            "Catturata finestra: {} (\"{}\")",
                            fg.exe_name, fg.window_title
                        ),
                        StatusLevel::Success,
                    );
                }
                Err(e) => {
                    state.set_status(
                        format!("Impossibile catturare finestra: {}", e),
                        StatusLevel::Error,
                    );
                }
            }
        }
    }

    /// Editor per RestoreWorkspace con egui::Grid
    fn show_restore_workspace_editor(
        ui: &mut Ui,
        slot: SlotId,
        state: &mut UiState,
        platform: Option<&dyn PlatformBackend>,
    ) {
        let mut marked_dirty = false;
        let mut capture_clicked = false;
        let mut remove_idx = None;

        if let Some(ActionDefinition::RestoreWorkspace(action)) =
            state.get_action_for_slot_mut(slot)
        {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!(
                        "Finestre nel Workspace: {}",
                        action.targets.len()
                    ))
                    .color(COLOR_TEXT_PRIMARY),
                );
                if ui.button("+ Cattura Finestra Corrente").clicked() {
                    capture_clicked = true;
                }
            });

            ui.add_space(6.0);

            for (i, target) in action.targets.iter().enumerate() {
                ui.horizontal(|ui| {
                    let desc = target
                        .selector
                        .exe_name
                        .as_deref()
                        .unwrap_or("Applicazione");
                    ui.label(format!("{}. {}", i + 1, desc));
                    if ui.button("Rimuovi").clicked() {
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
                    if let Some(ActionDefinition::RestoreWorkspace(action)) =
                        state.get_action_for_slot_mut(slot)
                    {
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

    /// Campi per MacroSequence con egui::Grid
    fn show_macro_sequence_fields(
        ui: &mut Ui,
        action: &mut MacroSequenceAction,
        marked_dirty: &mut bool,
    ) {
        egui::Grid::new("macro_seq_grid")
            .num_columns(2)
            .min_col_width(120.0)
            .spacing(vec2(12.0, 6.0))
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Concorrenza:").color(COLOR_TEXT_SECONDARY));
                let mut pol = match action.concurrency {
                    MacroConcurrencyPolicy::IgnoreIfRunning => 0,
                    MacroConcurrencyPolicy::RestartCurrent => 1,
                    MacroConcurrencyPolicy::QueueBounded(_) => 2,
                };
                let opts = [(0, "Ignora"), (1, "Riavvia")];
                if liquid_segmented_control(ui, "macro_pol_control", &mut pol, &opts) {
                    action.concurrency = match pol {
                        0 => MacroConcurrencyPolicy::IgnoreIfRunning,
                        _ => MacroConcurrencyPolicy::RestartCurrent,
                    };
                    *marked_dirty = true;
                }
                ui.end_row();

                ui.label(egui::RichText::new("Passaggi (Steps):").color(COLOR_TEXT_SECONDARY));
                ui.label(
                    egui::RichText::new(format!("{}", action.steps.len()))
                        .strong()
                        .color(COLOR_TEXT_PRIMARY),
                );
                ui.end_row();
            });
    }

    /// Campi per ExecProcess con egui::Grid
    fn show_exec_process_fields(
        ui: &mut Ui,
        action: &mut ExecProcessAction,
        marked_dirty: &mut bool,
    ) {
        ui.horizontal_wrapped(|ui| {
            let preset_cmd_btn = egui::Button::new(
                egui::RichText::new("Preset: agy (cmd.exe)")
                    .color(COLOR_APPLE_BLUE)
                    .font(FontId::proportional(11.0)),
            )
            .fill(COLOR_SURFACE_CARD)
            .stroke(Stroke::new(1.0_f32, COLOR_APPLE_BLUE));

            if ui
                .add(preset_cmd_btn)
                .on_hover_text("Configura comando inline salvato nel profilo: apre agy in prompt classico (cmd.exe /k agy).")
                .clicked()
            {
                action.program = std::path::PathBuf::from("cmd.exe");
                action.args = vec![
                    "/c".into(),
                    "start".into(),
                    "cmd.exe".into(),
                    "/k".into(),
                    "agy".into(),
                ];
                *marked_dirty = true;
            }

            let preset_wt_btn = egui::Button::new(
                egui::RichText::new("Preset: agy (wt.exe)")
                    .color(COLOR_APPLE_BLUE)
                    .font(FontId::proportional(11.0)),
            )
            .fill(COLOR_SURFACE_CARD)
            .stroke(Stroke::new(1.0_f32, COLOR_APPLE_BLUE));

            if ui
                .add(preset_wt_btn)
                .on_hover_text("Configura comando inline salvato nel profilo: apre agy in Windows Terminal moderno (wt.exe -d . agy).")
                .clicked()
            {
                action.program = std::path::PathBuf::from("wt.exe");
                action.args = vec!["-d".into(), ".".into(), "agy".into()];
                *marked_dirty = true;
            }
        });

        ui.add_space(6.0);

        egui::Grid::new("exec_proc_grid")
            .num_columns(2)
            .min_col_width(120.0)
            .spacing(vec2(12.0, 6.0))
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Programma:").color(COLOR_TEXT_SECONDARY));
                let mut prog = action.program.to_string_lossy().to_string();
                if ui
                    .add(TextEdit::singleline(&mut prog).desired_width(190.0))
                    .changed()
                {
                    action.program = std::path::PathBuf::from(prog);
                    *marked_dirty = true;
                }
                ui.end_row();

                ui.label(egui::RichText::new("Argomenti:").color(COLOR_TEXT_SECONDARY));
                let mut args_str = action.args.join(" ");
                if ui
                    .add(TextEdit::singleline(&mut args_str).desired_width(190.0))
                    .changed()
                {
                    action.args = if args_str.trim().is_empty() {
                        Vec::new()
                    } else if args_str.contains("-EncodedCommand ") {
                        let parts: Vec<&str> = args_str.splitn(2, "-EncodedCommand ").collect();
                        let mut v: Vec<String> =
                            parts[0].split_whitespace().map(String::from).collect();
                        v.push("-EncodedCommand".to_string());
                        v.push(parts[1].trim().to_string());
                        v
                    } else {
                        args_str.split_whitespace().map(String::from).collect()
                    };
                    *marked_dirty = true;
                }
                ui.end_row();
            });
    }

    /// Editor per ContextDispatch
    fn show_context_dispatch_editor(ui: &mut Ui, _slot: SlotId, _state: &mut UiState) {
        ui.label(
            egui::RichText::new("Regole attive per l'applicazione in primo piano.")
                .font(FontId::proportional(11.5))
                .color(COLOR_TEXT_SECONDARY),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspector_instantiation() {
        let mut inspector = InspectorPanel::new();
        let _ = &mut inspector;
    }
}
