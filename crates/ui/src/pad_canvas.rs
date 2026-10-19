//! Widget PadCanvas in stile Apple Liquid Glass & Alluminio CNC
//! Zero `from_rgba_premultiplied` errati, zero simboli Unicode mancanti (`□`).

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
            desired_size: vec2(580.0, 355.0),
        }
    }

    pub fn show(&mut self, ui: &mut Ui, state: &mut UiState) -> Response {
        let (rect, response) = ui.allocate_exact_size(self.desired_size, Sense::hover());
        let painter = ui.painter_at(rect);

        // 1. Ombra esterna morbida della scocca hardware
        painter.rect_filled(
            rect.translate(vec2(0.0, 8.0)).expand(2.0),
            Rounding::same(24.0),
            Color32::from_black_alpha(110),
        );

        // 2. Lastra Acrilico Fumè scuro traslucido + Bordo Vetro Speculare
        painter.rect_filled(
            rect,
            Rounding::same(22.0),
            Color32::from_rgba_unmultiplied(20, 23, 32, 240),
        );
        painter.rect_stroke(
            rect.shrink(1.0),
            Rounding::same(21.0),
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 18)),
        );
        painter.rect_stroke(
            rect,
            Rounding::same(22.0),
            Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 45)),
        );

        let origin = rect.min;

        // 3. 4 Viti Angolari in Alluminio Satinato
        let screw_offset = 16.0;
        let screw_centers = [
            pos2(origin.x + screw_offset, origin.y + screw_offset),
            pos2(
                origin.x + self.desired_size.x - screw_offset,
                origin.y + screw_offset,
            ),
            pos2(
                origin.x + screw_offset,
                origin.y + self.desired_size.y - screw_offset,
            ),
            pos2(
                origin.x + self.desired_size.x - screw_offset,
                origin.y + self.desired_size.y - screw_offset,
            ),
        ];
        for sc in screw_centers {
            Self::draw_satin_screw(&painter, sc);
        }

        // Intestazione serigrafata sul PCB
        painter.text(
            pos2(origin.x + 34.0, origin.y + 18.0),
            Align2::LEFT_CENTER,
            "VORTEXDECK HARDWARE",
            FontId::proportional(9.5),
            COLOR_TEXT_TERTIARY,
        );
        painter.text(
            pos2(origin.x + 324.0, origin.y + 18.0),
            Align2::RIGHT_CENTER,
            format!("LAYER {}", state.editing_layer.0 + 1),
            FontId::proportional(9.5),
            COLOR_APPLE_BLUE,
        );

        // -------------------------------------------------------------
        // SEZIONE 1: Matrice 4 Colonne x 3 Righe Tasti PBT (Sinistra)
        // -------------------------------------------------------------
        let grid_start_x = origin.x + 28.0;
        let grid_start_y = origin.y + 42.0;
        let key_w = 66.0;
        let key_h = 66.0;
        let gap_x = 12.0;
        let gap_y = 12.0;

        for r in 0..3u8 {
            for c in 0..4u8 {
                let slot = SlotId {
                    layer: state.editing_layer,
                    control: ControlId::Button { row: r, col: c },
                };

                let k_x = grid_start_x + (c as f32) * (key_w + gap_x);
                let k_y = grid_start_y + (r as f32) * (key_h + gap_y);
                let key_rect = Rect::from_min_size(pos2(k_x, k_y), vec2(key_w, key_h));

                let key_resp = ui.interact(key_rect, ui.id().with(("key", r, c)), Sense::click());
                if key_resp.clicked() {
                    state.select_slot(slot);
                }

                Self::draw_sculpted_keycap(&painter, key_rect, &key_resp, slot, r, c, state);
            }
        }

        // -------------------------------------------------------------
        // SEZIONE 2: 3 Indicatori LED Verticali (Centro)
        // -------------------------------------------------------------
        let led_x = grid_start_x + 4.0 * (key_w + gap_x) + 14.0;
        let led_y_center = origin.y + self.desired_size.y / 2.0 + 4.0;
        let led_spacing = 42.0;

        for i in 0..3u8 {
            let ly = led_y_center + (i as f32 - 1.0) * led_spacing;
            let is_active = state.editing_layer.0 == i;
            Self::draw_capsule_led(&painter, pos2(led_x, ly), i + 1, is_active);
        }

        // -------------------------------------------------------------
        // SEZIONE 3: 2 Encoders Rotativi in Alluminio CNC (Destra)
        // -------------------------------------------------------------
        let knob_center_x = origin.x + 478.0;
        let knob_1_y = origin.y + 98.0;
        let knob_2_y = origin.y + 246.0;
        let knob_radius = 42.0;

        for (enc_idx, cy) in [(0u8, knob_1_y), (1u8, knob_2_y)] {
            let center = pos2(knob_center_x, cy);
            Self::draw_brushed_knob(&painter, center, knob_radius, enc_idx);

            // Segmented Control a 3 zone [ - CCW | PUSH | + CW ] (senza glifi Unicode mancanti)
            let bar_y = cy + knob_radius + 16.0;
            let bar_w = 132.0;
            let bar_h = 24.0;
            let bar_rect = Rect::from_center_size(pos2(knob_center_x, bar_y), vec2(bar_w, bar_h));

            painter.rect_filled(
                bar_rect,
                Rounding::same(7.0),
                Color32::from_rgba_unmultiplied(12, 15, 22, 230),
            );
            painter.rect_stroke(
                bar_rect,
                Rounding::same(7.0),
                Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE),
            );

            let zone_w = bar_w / 3.0;
            let events = [
                (EncoderEvent::Ccw, "- CCW", 0),
                (EncoderEvent::Press, "PUSH", 1),
                (EncoderEvent::Cw, "+ CW", 2),
            ];

            for (ev, label, z_idx) in events {
                let slot = SlotId {
                    layer: state.editing_layer,
                    control: ControlId::Encoder {
                        index: enc_idx,
                        event: ev,
                    },
                };
                let is_sel = state.selected_slot == Some(slot);

                let z_min_x = bar_rect.min.x + (z_idx as f32) * zone_w;
                let zone_rect =
                    Rect::from_min_size(pos2(z_min_x, bar_rect.min.y), vec2(zone_w, bar_h))
                        .shrink(1.5);

                let zone_resp = ui.interact(
                    zone_rect,
                    ui.id().with(("enc_zone", enc_idx, z_idx)),
                    Sense::click(),
                );
                if zone_resp.clicked() {
                    state.select_slot(slot);
                }

                let bg_color = if is_sel {
                    COLOR_APPLE_BLUE
                } else if zone_resp.hovered() {
                    COLOR_CONTROL_HOVER
                } else {
                    Color32::TRANSPARENT
                };

                painter.rect_filled(zone_rect, Rounding::same(5.5), bg_color);

                let text_color = if is_sel {
                    Color32::WHITE
                } else {
                    COLOR_TEXT_SECONDARY
                };
                painter.text(
                    zone_rect.center(),
                    Align2::CENTER_CENTER,
                    label,
                    FontId::proportional(10.0),
                    text_color,
                );
            }
        }

        response
    }

    fn draw_satin_screw(painter: &Painter, center: Pos2) {
        painter.circle_filled(center, 4.5, Color32::from_rgb(168, 174, 186));
        painter.circle_stroke(
            center,
            4.5,
            Stroke::new(1.0_f32, Color32::from_rgb(90, 96, 108)),
        );
        painter.line_segment(
            [
                pos2(center.x - 2.3, center.y),
                pos2(center.x + 2.3, center.y),
            ],
            Stroke::new(1.0_f32, Color32::from_rgb(70, 74, 84)),
        );
        painter.line_segment(
            [
                pos2(center.x, center.y - 2.3),
                pos2(center.x, center.y + 2.3),
            ],
            Stroke::new(1.0_f32, Color32::from_rgb(70, 74, 84)),
        );
    }

    fn draw_sculpted_keycap(
        painter: &Painter,
        rect: Rect,
        resp: &Response,
        slot: SlotId,
        row: u8,
        col: u8,
        state: &UiState,
    ) {
        let is_selected = state.selected_slot == Some(slot);
        let config = state.get_slot_config(slot);

        let custom_color = config
            .and_then(|c| c.color_hex.as_ref())
            .and_then(|h| hex_to_color32(h));

        // 1. Base Esterna 3D del Keycap
        let base_bg = if is_selected {
            Color32::from_rgba_unmultiplied(24, 58, 110, 240)
        } else if resp.hovered() {
            Color32::from_rgba_unmultiplied(54, 62, 82, 240)
        } else {
            custom_color.unwrap_or(Color32::from_rgba_unmultiplied(38, 44, 58, 240))
        };
        painter.rect_filled(rect, Rounding::same(12.0), base_bg);

        // 2. Incavo Superiore Concavo (Dish)
        let dish_rect = rect.shrink(4.0);
        let dish_bg = if is_selected {
            Color32::from_rgba_unmultiplied(18, 42, 82, 250)
        } else if resp.hovered() {
            Color32::from_rgba_unmultiplied(44, 50, 68, 250)
        } else {
            Color32::from_rgba_unmultiplied(28, 33, 45, 250)
        };
        painter.rect_filled(dish_rect, Rounding::same(9.0), dish_bg);

        // Riflesso speculare superiore sul keycap
        painter.line_segment(
            [
                pos2(dish_rect.min.x + 7.0, dish_rect.min.y + 1.0),
                pos2(dish_rect.max.x - 7.0, dish_rect.min.y + 1.0),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 38)),
        );

        // 3. Bordo Selezione o Sottile
        if is_selected {
            painter.rect_stroke(
                rect.expand(2.0),
                Rounding::same(14.0),
                Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(10, 132, 255, 95)),
            );
            painter.rect_stroke(
                rect,
                Rounding::same(12.0),
                Stroke::new(2.0_f32, COLOR_APPLE_BLUE),
            );
        } else {
            painter.rect_stroke(
                rect,
                Rounding::same(12.0),
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 26)),
            );
        }

        // 4. Coordinate Matrice (es. R1-C1)
        let idx_str = format!("R{}-C{}", row + 1, col + 1);
        painter.text(
            pos2(rect.min.x + 7.0, rect.min.y + 6.0),
            Align2::LEFT_TOP,
            idx_str,
            FontId::proportional(8.5),
            COLOR_TEXT_TERTIARY,
        );

        // 5. Etichetta Centrale (UTF-8 safe, senza simboli Unicode mancanti)
        let label_text = config.map(|c| c.label.as_str()).unwrap_or("-");
        let display_lbl = if label_text.chars().count() > 8 {
            format!("{}..", label_text.chars().take(7).collect::<String>())
        } else {
            label_text.to_string()
        };
        painter.text(
            pos2(rect.center().x, rect.center().y - 1.0),
            Align2::CENTER_CENTER,
            display_lbl,
            FontId::proportional(12.0),
            COLOR_TEXT_PRIMARY,
        );

        // 6. Badge Inferiore
        let badge_text = state.get_badge_for_slot(slot);
        let b_color = badge_color(badge_text);
        let pill_rect =
            Rect::from_center_size(pos2(rect.center().x, rect.max.y - 11.0), vec2(38.0, 13.0));

        painter.rect_filled(
            pill_rect,
            Rounding::same(4.5),
            Color32::from_rgba_unmultiplied(b_color.r(), b_color.g(), b_color.b(), 32),
        );
        painter.rect_stroke(
            pill_rect,
            Rounding::same(4.5),
            Stroke::new(
                0.8_f32,
                Color32::from_rgba_unmultiplied(b_color.r(), b_color.g(), b_color.b(), 140),
            ),
        );
        painter.text(
            pill_rect.center(),
            Align2::CENTER_CENTER,
            badge_text,
            FontId::proportional(8.5),
            b_color,
        );
    }

    fn draw_capsule_led(painter: &Painter, center: Pos2, layer_num: u8, is_active: bool) {
        let capsule_rect = Rect::from_center_size(center, vec2(10.0, 22.0));
        if is_active {
            painter.rect_filled(
                capsule_rect.expand(4.0),
                Rounding::same(9.0),
                Color32::from_rgba_unmultiplied(10, 132, 255, 65),
            );
            painter.rect_filled(capsule_rect, Rounding::same(5.0), COLOR_APPLE_BLUE);
        } else {
            painter.rect_filled(
                capsule_rect,
                Rounding::same(5.0),
                Color32::from_rgb(42, 46, 58),
            );
        }
        painter.rect_stroke(
            capsule_rect,
            Rounding::same(5.0),
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 35)),
        );

        painter.text(
            pos2(center.x, center.y + 17.0),
            Align2::CENTER_TOP,
            format!("L{}", layer_num),
            FontId::proportional(9.0),
            if is_active {
                COLOR_TEXT_PRIMARY
            } else {
                COLOR_TEXT_TERTIARY
            },
        );
    }

    fn draw_brushed_knob(painter: &Painter, center: Pos2, radius: f32, enc_idx: u8) {
        // Ombra proiettata dalla manopola
        painter.circle_filled(
            center + vec2(0.0, 4.0),
            radius + 1.0,
            Color32::from_black_alpha(100),
        );

        // Ghiera esterna zigrinata in alluminio
        painter.circle_filled(center, radius, Color32::from_rgb(56, 62, 76));
        painter.circle_stroke(
            center,
            radius,
            Stroke::new(1.5_f32, Color32::from_rgb(130, 140, 160)),
        );

        for i in 0..24 {
            let angle = (i as f32) * (2.0 * PI / 24.0);
            let p1 = pos2(
                center.x + angle.cos() * (radius - 4.5),
                center.y + angle.sin() * (radius - 4.5),
            );
            let p2 = pos2(
                center.x + angle.cos() * (radius - 1.0),
                center.y + angle.sin() * (radius - 1.0),
            );
            painter.line_segment(
                [p1, p2],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 45)),
            );
        }

        // Corpo centrale tornito
        painter.circle_filled(center, radius - 6.5, Color32::from_rgb(32, 36, 46));
        painter.circle_stroke(
            center,
            radius - 6.5,
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 35)),
        );

        // Tacca indicatrice luminosa a ore 12
        painter.line_segment(
            [
                pos2(center.x, center.y - 14.0),
                pos2(center.x, center.y - (radius - 8.5)),
            ],
            Stroke::new(2.2_f32, COLOR_APPLE_BLUE),
        );

        let enc_title = if enc_idx == 0 { "ENC 1" } else { "ENC 2" };
        painter.text(
            pos2(center.x, center.y + 4.0),
            Align2::CENTER_CENTER,
            enc_title,
            FontId::proportional(10.0),
            COLOR_TEXT_SECONDARY,
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
        _ => COLOR_TEXT_TERTIARY,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pad_canvas_default() {
        let canvas = PadCanvas::new();
        assert_eq!(canvas.desired_size.x, 580.0);
        assert_eq!(canvas.desired_size.y, 355.0);
    }

    #[test]
    fn test_badge_colors() {
        assert_eq!(badge_color("HID"), COLOR_APPLE_BLUE);
        assert_eq!(badge_color("WIN"), COLOR_APPLE_GREEN);
    }
}
