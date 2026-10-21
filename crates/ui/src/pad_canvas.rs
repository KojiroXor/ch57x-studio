//! Rappresentazione Hardware fedele del Macropad 4x3 + 3 LED + 2 Encoder Rotativi CNC
//! Layout 570x350 proporzionato per entrare comodamente nella colonna sinistra senza mai tagliare i tasti.

use crate::state::{hex_to_color32, UiState};
use crate::theme::*;
use ch57x_domain::hardware::{ControlId, EncoderEvent, SlotId};
use egui::{
    pos2, vec2, Align2, Color32, FontId, Painter, Pos2, Rect, Response, Rounding, Sense, Stroke,
    Ui, Vec2,
};
use std::f32::consts::PI;

pub struct PadCanvas {
    pub desired_size: Vec2,
}

impl Default for PadCanvas {
    fn default() -> Self {
        Self::new()
    }
}

impl PadCanvas {
    pub fn new() -> Self {
        Self {
            desired_size: vec2(560.0, 340.0),
        }
    }

    pub fn show(&mut self, ui: &mut Ui, state: &mut UiState) -> Response {
        let (rect, response) = ui.allocate_exact_size(self.desired_size, Sense::hover());
        let painter = ui.painter_at(rect);

        // 1. Ombra profonda sotto la lastra in acrilico
        painter.rect_filled(
            rect.translate(vec2(0.0, 6.0)),
            Rounding::same(22.0),
            Color32::from_black_alpha(120),
        );

        // 2. Scocca Acrilico Fumè + Bordo Vetro
        painter.rect_filled(rect, Rounding::same(20.0), Color32::from_rgb(20, 23, 31));
        painter.rect_stroke(
            rect,
            Rounding::same(20.0),
            Stroke::new(1.5_f32, rgba(255, 255, 255, 42)),
        );
        painter.rect_stroke(
            rect.shrink(2.0),
            Rounding::same(18.0),
            Stroke::new(1.0_f32, rgba(255, 255, 255, 14)),
        );

        let origin = rect.min;

        // 3. 4 Viti Angolari in Alluminio
        let so = 14.0;
        for sc in [
            pos2(origin.x + so, origin.y + so),
            pos2(origin.x + self.desired_size.x - so, origin.y + so),
            pos2(origin.x + so, origin.y + self.desired_size.y - so),
            pos2(
                origin.x + self.desired_size.x - so,
                origin.y + self.desired_size.y - so,
            ),
        ] {
            Self::draw_satin_screw(&painter, sc);
        }

        // Serigrafia superiore
        painter.text(
            pos2(origin.x + 28.0, origin.y + 16.0),
            Align2::LEFT_CENTER,
            "VORTEXDECK HARDWARE",
            FontId::proportional(9.0),
            COLOR_TEXT_TERTIARY,
        );
        painter.text(
            pos2(origin.x + 318.0, origin.y + 16.0),
            Align2::RIGHT_CENTER,
            format!("LAYER {}", state.editing_layer.0 + 1),
            FontId::proportional(9.0),
            COLOR_APPLE_BLUE,
        );

        // -------------------------------------------------------------
        // SEZIONE 1: Matrice 4 Colonne x 3 Righe Tasti (Sinistra)
        // -------------------------------------------------------------
        let grid_x = origin.x + 26.0;
        let grid_y = origin.y + 34.0;
        let kw = 66.0;
        let kh = 86.0;
        let gx = 10.0;
        let gy = 12.0;

        let pointer_pos = ui.input(|i| i.pointer.hover_pos());
        let mut hovered_button_slot: Option<SlotId> = None;

        for r in 0..3u8 {
            for c in 0..4u8 {
                let slot = SlotId {
                    layer: state.editing_layer,
                    control: ControlId::Button { row: r, col: c },
                };
                let kx = grid_x + (c as f32) * (kw + gx);
                let ky = grid_y + (r as f32) * (kh + gy);
                let k_rect = Rect::from_min_size(pos2(kx, ky), vec2(kw, kh));

                if let Some(pos) = pointer_pos {
                    if k_rect.contains(pos) {
                        hovered_button_slot = Some(slot);
                    }
                }

                let k_resp =
                    ui.interact(k_rect, ui.id().with(("key", r, c)), Sense::click_and_drag());

                if k_resp.drag_started() {
                    state.dragged_slot = Some(slot);
                    state.select_slot(slot);
                }

                if k_resp.clicked() && state.dragged_slot.is_none() {
                    state.select_slot(slot);
                }

                Self::draw_keycap(&painter, k_rect, &k_resp, slot, r, c, state);
            }
        }

        // Aggiorna target slot e posizione durante il drag
        if state.dragged_slot.is_some() {
            state.drag_target_slot = hovered_button_slot;
            state.drag_pointer_pos = pointer_pos;
            ui.ctx().request_repaint();
        }

        // Gestione fine trascinamento (drop o rilascio mouse)
        if ui.input(|i| i.pointer.any_released()) {
            if let Some(source_slot) = state.dragged_slot.take() {
                if let Some(target_slot) = hovered_button_slot {
                    if source_slot != target_slot {
                        state.swap_slots(source_slot, target_slot);
                        let src_idx = (source_slot.global_index() % 18) + 1;
                        let dst_idx = (target_slot.global_index() % 18) + 1;
                        state.set_status(
                            format!(
                                "Configurazione scambiata con successo tra Tasto {} e Tasto {}.",
                                src_idx, dst_idx
                            ),
                            crate::state::StatusLevel::Success,
                        );
                    }
                }
                state.drag_target_slot = None;
                state.drag_pointer_pos = None;
            }
        }

        // Disegna l'anteprima fluttuante del tasto trascinato
        if let Some(dragged) = state.dragged_slot {
            if let Some(pos) = pointer_pos {
                Self::draw_drag_ghost(&painter, pos, dragged, state);
            }
        }

        // -------------------------------------------------------------
        // SEZIONE 2: 3 Indicatori LED Verticali (Centro)
        // -------------------------------------------------------------
        let led_x = grid_x + 4.0 * (kw + gx) + 12.0;
        let led_cy = origin.y + self.desired_size.y * 0.5 + 6.0;
        for i in 0..3u8 {
            let ly = led_cy + (i as f32 - 1.0) * 44.0;
            Self::draw_led(&painter, pos2(led_x, ly), i + 1, state.editing_layer.0 == i);
        }

        // -------------------------------------------------------------
        // SEZIONE 3: 2 Encoder Rotativi CNC + Selettore 3 Zone (Destra)
        // -------------------------------------------------------------
        let knob_cx = origin.x + 462.0;
        let knob_r = 40.0;

        for (enc_idx, cy) in [(0u8, origin.y + 92.0), (1u8, origin.y + 236.0)] {
            let center = pos2(knob_cx, cy);
            Self::draw_knob(&painter, center, knob_r, enc_idx);

            // Barra a 3 zone sotto la manopola: [ CCW | PUSH | CW ]
            let bar_rect =
                Rect::from_center_size(pos2(knob_cx, cy + knob_r + 16.0), vec2(126.0, 24.0));
            painter.rect_filled(bar_rect, Rounding::same(6.5), COLOR_SURFACE_INSET);
            painter.rect_stroke(
                bar_rect,
                Rounding::same(6.5),
                Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE),
            );

            let zw = bar_rect.width() / 3.0;
            for (ev, lbl, z_i) in [
                (EncoderEvent::Ccw, "CCW", 0),
                (EncoderEvent::Press, "PUSH", 1),
                (EncoderEvent::Cw, "CW", 2),
            ] {
                let slot = SlotId {
                    layer: state.editing_layer,
                    control: ControlId::Encoder {
                        index: enc_idx,
                        event: ev,
                    },
                };
                let is_sel = state.selected_slot == Some(slot);
                let z_rect = Rect::from_min_size(
                    pos2(bar_rect.min.x + (z_i as f32) * zw, bar_rect.min.y),
                    vec2(zw, bar_rect.height()),
                )
                .shrink(1.5);

                let z_resp = ui.interact(
                    z_rect,
                    ui.id().with(("enc_z", enc_idx, z_i)),
                    Sense::click(),
                );
                if z_resp.clicked() {
                    state.select_slot(slot);
                }

                let bg = if is_sel {
                    COLOR_APPLE_BLUE
                } else if z_resp.hovered() {
                    COLOR_CONTROL_HOVER
                } else {
                    Color32::TRANSPARENT
                };
                painter.rect_filled(z_rect, Rounding::same(5.0), bg);
                painter.text(
                    z_rect.center(),
                    Align2::CENTER_CENTER,
                    lbl,
                    FontId::proportional(9.5),
                    if is_sel {
                        Color32::WHITE
                    } else {
                        COLOR_TEXT_SECONDARY
                    },
                );
            }
        }

        response
    }

    fn draw_satin_screw(painter: &Painter, c: Pos2) {
        painter.circle_filled(c, 4.2, Color32::from_rgb(160, 166, 178));
        painter.circle_stroke(c, 4.2, Stroke::new(0.8_f32, Color32::from_rgb(80, 86, 98)));
        painter.line_segment(
            [pos2(c.x - 2.0, c.y), pos2(c.x + 2.0, c.y)],
            Stroke::new(0.9_f32, Color32::from_rgb(65, 70, 80)),
        );
        painter.line_segment(
            [pos2(c.x, c.y - 2.0), pos2(c.x, c.y + 2.0)],
            Stroke::new(0.9_f32, Color32::from_rgb(65, 70, 80)),
        );
    }

    fn draw_keycap(
        painter: &Painter,
        rect: Rect,
        resp: &Response,
        slot: SlotId,
        row: u8,
        col: u8,
        state: &UiState,
    ) {
        let is_sel = state.selected_slot == Some(slot);
        let is_being_dragged = state.dragged_slot == Some(slot);
        let is_drag_target = state.dragged_slot.is_some()
            && state.drag_target_slot == Some(slot)
            && !is_being_dragged;

        let cfg = state.get_slot_config(slot);
        let custom_col = cfg
            .and_then(|c| c.color_hex.as_deref())
            .and_then(hex_to_color32);

        let base_col = if is_drag_target {
            Color32::from_rgb(20, 65, 45)
        } else if is_being_dragged {
            Color32::from_rgb(20, 24, 32)
        } else if is_sel {
            Color32::from_rgb(28, 56, 104)
        } else if resp.hovered() {
            Color32::from_rgb(48, 55, 72)
        } else {
            custom_col.unwrap_or(Color32::from_rgb(35, 40, 52))
        };
        painter.rect_filled(rect, Rounding::same(11.0), base_col);

        let dish = rect.shrink2(vec2(3.5, 4.0));
        let dish_col = if is_drag_target {
            Color32::from_rgb(18, 52, 36)
        } else if is_being_dragged {
            Color32::from_rgb(18, 20, 26)
        } else if is_sel {
            Color32::from_rgb(22, 44, 82)
        } else {
            Color32::from_rgb(27, 31, 42)
        };
        painter.rect_filled(dish, Rounding::same(8.0), dish_col);

        let border = if is_drag_target {
            Stroke::new(2.5_f32, COLOR_APPLE_GREEN)
        } else if is_being_dragged {
            Stroke::new(1.0_f32, rgba(255, 255, 255, 16))
        } else if is_sel {
            Stroke::new(2.0_f32, COLOR_APPLE_BLUE)
        } else {
            Stroke::new(1.0_f32, rgba(255, 255, 255, 28))
        };
        painter.rect_stroke(rect, Rounding::same(11.0), border);

        // Coordinate R1-C1
        painter.text(
            pos2(dish.min.x + 5.0, dish.min.y + 4.0),
            Align2::LEFT_TOP,
            format!("R{}-C{}", row + 1, col + 1),
            FontId::proportional(8.0),
            COLOR_TEXT_TERTIARY,
        );

        // Label UTF-8 safe
        let raw_lbl = cfg.map(|c| c.label.as_str()).unwrap_or("");
        let lbl = if raw_lbl.is_empty() {
            format!("Slot {}", row * 4 + col + 1)
        } else if raw_lbl.chars().count() > 8 {
            format!("{}..", raw_lbl.chars().take(7).collect::<String>())
        } else {
            raw_lbl.to_string()
        };

        painter.text(
            pos2(dish.center().x, dish.center().y - 1.0),
            Align2::CENTER_CENTER,
            lbl,
            FontId::proportional(11.5),
            COLOR_TEXT_PRIMARY,
        );

        // Badge inferiore
        let badge = state.get_badge_for_slot(slot);
        let b_col = badge_color(badge);
        let pill =
            Rect::from_center_size(pos2(dish.center().x, dish.max.y - 9.5), vec2(36.0, 13.0));
        painter.rect_filled(
            pill,
            Rounding::same(4.0),
            rgba(b_col.r(), b_col.g(), b_col.b(), 36),
        );
        painter.rect_stroke(
            pill,
            Rounding::same(4.0),
            Stroke::new(0.8_f32, rgba(b_col.r(), b_col.g(), b_col.b(), 130)),
        );
        painter.text(
            pill.center(),
            Align2::CENTER_CENTER,
            badge,
            FontId::proportional(8.0),
            b_col,
        );
    }

    fn draw_led(painter: &Painter, center: Pos2, num: u8, active: bool) {
        let r = Rect::from_center_size(center, vec2(9.0, 20.0));
        if active {
            painter.rect_filled(r.expand(3.5), Rounding::same(8.0), rgba(10, 132, 255, 65));
            painter.rect_filled(r, Rounding::same(4.5), COLOR_APPLE_BLUE);
        } else {
            painter.rect_filled(r, Rounding::same(4.5), Color32::from_rgb(40, 44, 56));
        }
        painter.rect_stroke(
            r,
            Rounding::same(4.5),
            Stroke::new(1.0_f32, rgba(255, 255, 255, 32)),
        );
        painter.text(
            pos2(center.x, center.y + 15.0),
            Align2::CENTER_TOP,
            format!("L{}", num),
            FontId::proportional(8.5),
            if active {
                COLOR_TEXT_PRIMARY
            } else {
                COLOR_TEXT_TERTIARY
            },
        );
    }

    fn draw_knob(painter: &Painter, center: Pos2, radius: f32, idx: u8) {
        painter.circle_filled(
            center + vec2(0.0, 3.0),
            radius + 1.0,
            Color32::from_black_alpha(100),
        );
        painter.circle_filled(center, radius, Color32::from_rgb(54, 60, 74));
        painter.circle_stroke(
            center,
            radius,
            Stroke::new(1.5_f32, Color32::from_rgb(125, 135, 155)),
        );

        for i in 0..20 {
            let a = (i as f32) * (2.0 * PI / 20.0);
            let p1 = pos2(
                center.x + a.cos() * (radius - 4.0),
                center.y + a.sin() * (radius - 4.0),
            );
            let p2 = pos2(
                center.x + a.cos() * (radius - 1.0),
                center.y + a.sin() * (radius - 1.0),
            );
            painter.line_segment([p1, p2], Stroke::new(1.0_f32, rgba(255, 255, 255, 42)));
        }

        painter.circle_filled(center, radius - 6.0, Color32::from_rgb(30, 34, 44));
        painter.circle_stroke(
            center,
            radius - 6.0,
            Stroke::new(1.0_f32, rgba(255, 255, 255, 35)),
        );
        painter.line_segment(
            [
                pos2(center.x, center.y - 12.0),
                pos2(center.x, center.y - (radius - 8.0)),
            ],
            Stroke::new(2.0_f32, COLOR_APPLE_BLUE),
        );
        painter.text(
            pos2(center.x, center.y + 4.0),
            Align2::CENTER_CENTER,
            format!("ENC {}", idx + 1),
            FontId::proportional(9.5),
            COLOR_TEXT_SECONDARY,
        );
    }

    fn draw_drag_ghost(painter: &Painter, pos: Pos2, slot: SlotId, state: &UiState) {
        let ghost_size = vec2(66.0, 84.0);
        let rect = Rect::from_center_size(pos, ghost_size);

        // Ombra profonda fluttuante
        painter.rect_filled(
            rect.translate(vec2(0.0, 6.0)),
            Rounding::same(12.0),
            Color32::from_black_alpha(160),
        );

        // Corpo fluttuante con bordo luminoso blu Apple
        painter.rect_filled(rect, Rounding::same(12.0), rgba(22, 35, 60, 240));
        painter.rect_stroke(
            rect,
            Rounding::same(12.0),
            Stroke::new(2.0_f32, COLOR_APPLE_BLUE),
        );

        let dish = rect.shrink2(vec2(3.5, 4.0));
        painter.rect_filled(dish, Rounding::same(8.0), rgba(28, 48, 85, 220));

        let cfg = state.get_slot_config(slot);
        let raw_lbl = cfg.map(|c| c.label.as_str()).unwrap_or("");
        let lbl = if raw_lbl.is_empty() {
            "Sposta".to_string()
        } else if raw_lbl.chars().count() > 8 {
            format!("{}..", raw_lbl.chars().take(7).collect::<String>())
        } else {
            raw_lbl.to_string()
        };

        painter.text(
            pos2(dish.center().x, dish.center().y - 2.0),
            Align2::CENTER_CENTER,
            lbl,
            FontId::proportional(11.5),
            Color32::WHITE,
        );

        let badge = state.get_badge_for_slot(slot);
        let b_col = badge_color(badge);
        let pill =
            Rect::from_center_size(pos2(dish.center().x, dish.max.y - 9.5), vec2(36.0, 13.0));
        painter.rect_filled(
            pill,
            Rounding::same(4.0),
            rgba(b_col.r(), b_col.g(), b_col.b(), 60),
        );
        painter.rect_stroke(pill, Rounding::same(4.0), Stroke::new(1.0_f32, b_col));
        painter.text(
            pill.center(),
            Align2::CENTER_CENTER,
            badge,
            FontId::proportional(8.0),
            b_col,
        );
    }
}

pub fn badge_color(badge: &str) -> Color32 {
    match badge {
        "HID" => COLOR_APPLE_BLUE,
        "WIN" => COLOR_APPLE_GREEN,
        "WSP" => COLOR_APPLE_PURPLE,
        "MAC" => COLOR_APPLE_ORANGE,
        "CMD" => COLOR_APPLE_RED,
        "CTX" => COLOR_APPLE_CYAN,
        "LUM" => COLOR_APPLE_ORANGE,
        _ => COLOR_TEXT_TERTIARY,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pad_canvas_default() {
        let c = PadCanvas::default();
        assert!(c.desired_size.x > 0.0);
    }

    #[test]
    fn test_badge_colors() {
        assert_eq!(badge_color("HID"), COLOR_APPLE_BLUE);
        assert_eq!(badge_color("WIN"), COLOR_APPLE_GREEN);
    }
}
