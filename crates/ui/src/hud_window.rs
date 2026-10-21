//! On-Screen Display (HUD OSD) Cheat-Sheet Flottante in stile Apple Liquid Glass

use crate::pad_canvas::badge_color;
use crate::state::UiState;
use crate::theme::*;
use ch57x_domain::hardware::{ControlId, EncoderEvent, LayerIndex, SlotId};
use egui::{pos2, vec2, Align2, Context, FontId, Margin, Rounding, Sense, Stroke, Ui};

pub struct HudWindow {
    pub is_visible: bool,
}

impl Default for HudWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl HudWindow {
    pub fn new() -> Self {
        Self { is_visible: false }
    }

    pub fn show(&mut self, ctx: &Context, state: &mut UiState) {
        if !self.is_visible {
            return;
        }
        egui::Window::new("VortexDeck OSD")
            .anchor(Align2::CENTER_BOTTOM, vec2(0.0, -48.0))
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .frame(egui::Frame {
                inner_margin: Margin::same(18.0),
                rounding: RADIUS_CARD,
                fill: rgba(16, 20, 30, 238),
                stroke: Stroke::new(1.0_f32, COLOR_BORDER_STRONG),
                ..Default::default()
            })
            .show(ctx, |ui| {
                self.show_content(ui, state);
            });
    }

    pub fn show_content(&mut self, ui: &mut Ui, state: &mut UiState) {
        ui.vertical(|ui| {
            // Header OSD
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("VORTEXDECK OSD")
                        .font(FontId::proportional(12.0))
                        .strong()
                        .color(COLOR_TEXT_SECONDARY),
                );
                ui.add_space(12.0);
                let mut cur = state.editing_layer.0;
                let opts = [(0u8, "Layer 1"), (1u8, "Layer 2"), (2u8, "Layer 3")];
                if segmented_control(ui, &mut cur, &opts) {
                    state.set_editing_layer(LayerIndex(cur));
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(10.0);

            ui.horizontal_top(|ui| {
                // Griglia 4x3 Tasti
                ui.vertical(|ui| {
                    for r in 0..3u8 {
                        ui.horizontal(|ui| {
                            for c in 0..4u8 {
                                let slot = SlotId {
                                    layer: state.editing_layer,
                                    control: ControlId::Button { row: r, col: c },
                                };
                                Self::draw_osd_cell(ui, slot, vec2(74.0, 50.0), None, state);
                            }
                        });
                    }
                });

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(16.0);

                // 2 Encoder Rotativi
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("ENCODERS")
                            .font(FontId::proportional(10.5))
                            .strong()
                            .color(COLOR_TEXT_TERTIARY),
                    );
                    for (idx, title) in [
                        (0u8, "Encoder 1 (Superiore)"),
                        (1u8, "Encoder 2 (Inferiore)"),
                    ] {
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(title)
                                .font(FontId::proportional(11.0))
                                .color(COLOR_TEXT_SECONDARY),
                        );
                        ui.horizontal(|ui| {
                            for (ev, tag) in [
                                (EncoderEvent::Ccw, "CCW"),
                                (EncoderEvent::Press, "PUSH"),
                                (EncoderEvent::Cw, "CW"),
                            ] {
                                let slot = SlotId {
                                    layer: state.editing_layer,
                                    control: ControlId::Encoder {
                                        index: idx,
                                        event: ev,
                                    },
                                };
                                Self::draw_osd_cell(ui, slot, vec2(74.0, 42.0), Some(tag), state);
                            }
                        });
                    }
                });
            });
        });
    }

    fn draw_osd_cell(
        ui: &mut Ui,
        slot: SlotId,
        size: egui::Vec2,
        prefix: Option<&str>,
        state: &UiState,
    ) {
        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
        let p = ui.painter();

        p.rect_filled(rect, Rounding::same(8.0), COLOR_SURFACE_INSET);
        p.rect_stroke(
            rect,
            Rounding::same(8.0),
            Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE),
        );

        let badge = state.get_badge_for_slot(slot);
        let b_col = badge_color(badge);

        if let Some(pre) = prefix {
            p.text(
                pos2(rect.min.x + 6.0, rect.min.y + 5.0),
                Align2::LEFT_TOP,
                pre,
                FontId::proportional(8.5),
                COLOR_TEXT_TERTIARY,
            );
        }

        p.text(
            pos2(rect.max.x - 6.0, rect.min.y + 5.0),
            Align2::RIGHT_TOP,
            badge,
            FontId::proportional(8.5),
            b_col,
        );

        let raw_lbl = state
            .get_slot_config(slot)
            .map(|c| c.label.as_str())
            .unwrap_or("—");
        let lbl = if raw_lbl.chars().count() > 9 {
            format!("{}..", raw_lbl.chars().take(8).collect::<String>())
        } else {
            raw_lbl.to_string()
        };

        p.text(
            pos2(rect.center().x, rect.center().y + 6.0),
            Align2::CENTER_CENTER,
            lbl,
            FontId::proportional(11.0),
            COLOR_TEXT_PRIMARY,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hud_window_instantiation() {
        let h = HudWindow::new();
        assert!(!h.is_visible);
    }
}
