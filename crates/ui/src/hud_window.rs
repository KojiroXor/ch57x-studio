//! HUD OSD compact layout window in Cupertino Pro Dark style
//!
//! Visualizza la mappa completa dei 12 tasti e dei 2 encoder a schermo
//! con uno stile visivo traslucido da On-Screen Display.

use crate::pad_canvas::badge_color;
use crate::state::UiState;
use crate::theme::*;
use ch57x_domain::hardware::{ControlId, EncoderEvent, LayerIndex, SlotId, COL_COUNT, ROW_COUNT};
use egui::{
    pos2, vec2, Align2, Color32, Context, FontId, Rounding, Stroke, Ui, Vec2, Window,
};

pub struct HudWindow {
    pub is_visible: bool,
    pub compact_mode: bool,
}

impl Default for HudWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl HudWindow {
    pub fn new() -> Self {
        Self {
            is_visible: false,
            compact_mode: false,
        }
    }

    /// Disegna la finestra HUD flottante OSD se visibile
    pub fn show(&mut self, ctx: &Context, state: &mut UiState) {
        if !self.is_visible {
            return;
        }

        let mut open = self.is_visible;
        Window::new("VortexDeck HUD OSD")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .title_bar(false)
            .anchor(Align2::CENTER_TOP, vec2(0.0, 40.0))
            .frame(egui::Frame {
                inner_margin: egui::Margin::same(14.0),
                rounding: Rounding::same(16.0),
                fill: Color32::from_rgba_premultiplied(28, 28, 30, 235),
                stroke: Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE),
                ..Default::default()
            })
            .show(ctx, |ui| {
                self.show_content(ui, state);
            });

        self.is_visible = open;
    }

    /// Disegna il contenuto interno dell'HUD OSD (usabile anche embedded)
    pub fn show_content(&self, ui: &mut Ui, state: &mut UiState) {
        ui.vertical(|ui| {
            // Header OSD stile Cupertino
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("VORTEXDECK OSD")
                        .font(FontId::proportional(12.0))
                        .color(COLOR_TEXT_SECONDARY)
                        .strong(),
                );

                ui.add_space(10.0);

                // Layer indicator pills (L1, L2, L3)
                for l in 0..3u8 {
                    let is_active = state.editing_layer.0 == l;
                    let text = format!("Layer {}", l + 1);
                    let col = if is_active {
                        Color32::WHITE
                    } else {
                        COLOR_TEXT_SECONDARY
                    };

                    let btn = egui::Button::new(egui::RichText::new(text).color(col).strong())
                        .fill(if is_active {
                            COLOR_APPLE_BLUE
                        } else {
                            COLOR_CONTROL_INACTIVE
                        })
                        .rounding(Rounding::same(6.0));

                    if ui.add(btn).clicked() {
                        state.set_editing_layer(LayerIndex(l));
                    }
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Corpo principale: Tasti a sinistra, Encoders a destra
            ui.horizontal(|ui| {
                // Griglia 12 Tasti 4x3
                ui.vertical(|ui| {
                    let key_size = vec2(64.0, 48.0);

                    for r in 0..ROW_COUNT as u8 {
                        ui.horizontal(|ui| {
                            for c in 0..COL_COUNT as u8 {
                                let slot = SlotId {
                                    layer: state.editing_layer,
                                    control: ControlId::Button { row: r, col: c },
                                };
                                Self::draw_hud_key(ui, key_size, slot, state);
                            }
                        });
                    }
                });

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(12.0);

                // Sezione Encoders
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("ENCODERS")
                            .font(FontId::proportional(10.5))
                            .color(COLOR_TEXT_SECONDARY)
                            .strong(),
                    );
                    ui.add_space(4.0);

                    for enc_idx in 0..2u8 {
                        let enc_title = if enc_idx == 0 { "Encoder 1 (Top)" } else { "Encoder 2 (Bot)" };
                        ui.label(egui::RichText::new(enc_title).font(FontId::proportional(10.0)).color(COLOR_TEXT_TERTIARY));

                        let events = [
                            (EncoderEvent::Ccw, "CCW"),
                            (EncoderEvent::Press, "PUSH"),
                            (EncoderEvent::Cw, "CW"),
                        ];

                        ui.horizontal(|ui| {
                            for (ev, symbol) in events {
                                let slot = SlotId {
                                    layer: state.editing_layer,
                                    control: ControlId::Encoder {
                                        index: enc_idx,
                                        event: ev,
                                    },
                                };
                                let badge = state.get_badge_for_slot(slot);
                                let config = state.get_slot_config(slot);
                                let label = config.map(|c| c.label.as_str()).unwrap_or(symbol);
                                let is_sel = state.selected_slot == Some(slot);

                                let (pill_rect, _) = ui.allocate_exact_size(vec2(52.0, 34.0), egui::Sense::hover());
                                let bg = if is_sel {
                                    Color32::from_rgb(30, 45, 65)
                                } else {
                                    COLOR_SURFACE_CARD
                                };
                                ui.painter().rect_filled(pill_rect, Rounding::same(6.0), bg);
                                ui.painter().rect_stroke(
                                    pill_rect,
                                    Rounding::same(6.0),
                                    Stroke::new(1.0_f32, if is_sel { COLOR_APPLE_BLUE } else { COLOR_BORDER_SUBTLE }),
                                );

                                // Symbol in alto
                                ui.painter().text(
                                    pos2(pill_rect.min.x + 5.0, pill_rect.min.y + 4.0),
                                    Align2::LEFT_TOP,
                                    symbol,
                                    FontId::proportional(8.5),
                                    COLOR_TEXT_SECONDARY,
                                );

                                // Badge in alto a destra
                                ui.painter().text(
                                    pos2(pill_rect.max.x - 4.0, pill_rect.min.y + 4.0),
                                    Align2::RIGHT_TOP,
                                    badge,
                                    FontId::proportional(8.0),
                                    badge_color(badge),
                                );

                                // Label al centro
                                let display_lbl = if label.len() > 6 {
                                    format!("{}..", &label[..5])
                                } else {
                                    label.to_string()
                                };
                                ui.painter().text(
                                    pos2(pill_rect.center().x, pill_rect.center().y + 4.0),
                                    Align2::CENTER_CENTER,
                                    display_lbl,
                                    FontId::proportional(10.0),
                                    COLOR_TEXT_PRIMARY,
                                );
                            }
                        });
                        ui.add_space(4.0);
                    }
                });
            });
        });
    }

    /// Disegna un singolo tasto compatto HUD
    fn draw_hud_key(ui: &mut Ui, size: Vec2, slot: SlotId, state: &UiState) {
        let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
        let is_selected = state.selected_slot == Some(slot);
        let config = state.get_slot_config(slot);

        let bg = if is_selected {
            Color32::from_rgb(30, 45, 65)
        } else {
            COLOR_SURFACE_CARD
        };
        let stroke = if is_selected {
            Stroke::new(1.5_f32, COLOR_APPLE_BLUE)
        } else {
            Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE)
        };

        ui.painter().rect_filled(rect, Rounding::same(6.0), bg);
        ui.painter().rect_stroke(rect, Rounding::same(6.0), stroke);

        // Badge pill in alto a sinistra
        let badge_text = state.get_badge_for_slot(slot);
        let badge_bg = badge_color(badge_text);
        ui.painter().text(
            pos2(rect.min.x + 5.0, rect.min.y + 4.0),
            Align2::LEFT_TOP,
            badge_text,
            FontId::proportional(8.0),
            badge_bg,
        );

        // Label tasto
        let label_text = config.map(|c| c.label.as_str()).unwrap_or("—");
        let display_lbl = if label_text.len() > 7 {
            format!("{}..", &label_text[..6])
        } else {
            label_text.to_string()
        };

        ui.painter().text(
            pos2(rect.center().x, rect.center().y + 1.0),
            Align2::CENTER_CENTER,
            display_lbl,
            FontId::proportional(11.0),
            COLOR_TEXT_PRIMARY,
        );

        // Trigger token in basso
        if let Some(token) = state.get_trigger_token_for_slot(slot) {
            ui.painter().text(
                pos2(rect.center().x, rect.max.y - 3.0),
                Align2::CENTER_BOTTOM,
                token,
                FontId::proportional(8.0),
                COLOR_TEXT_TERTIARY,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hud_window_instantiation() {
        let mut hud = HudWindow::new();
        assert!(!hud.is_visible);
        hud.is_visible = true;
        assert!(hud.is_visible);
    }
}
