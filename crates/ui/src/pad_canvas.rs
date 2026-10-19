//! Pad canvas photorealistic Cupertino Pro Dark widget faithful to Macropad VortexDeck
//!
//! Visualizza:
//! - Lastra in acrilico fumè scuro con 4 viti angolari satinate
//! - Griglia 4 colonne x 3 righe = 12 keycaps scolpiti stile Apple Magic Keyboard / PBT
//! - 3 capsule LED verticali con alone glow radiale
//! - 2 encoder rotativi in alluminio spazzolato con tacche radiali e segmented control a 3 zone
//! - Zero emoji: simboli geometrici vettoriali puri e feedback luminoso

use crate::state::{hex_to_color32, UiState};
use crate::theme::*;
use ch57x_domain::hardware::{
    ControlId, EncoderEvent, LayerIndex, SlotId, COL_COUNT, ROW_COUNT,
};
use egui::{
    pos2, vec2, Align2, Color32, FontId, Painter, Pos2, Rect, Response, Rounding, Sense, Stroke, Ui,
    Vec2,
};
use std::f32::consts::PI;

const CHASSIS_ACRYLIC: Color32 = Color32::from_rgb(34, 34, 37); // #222225
const KEYCAP_BASE: Color32 = Color32::from_rgb(44, 44, 48); // #2C2C30
const KEYCAP_DISH: Color32 = Color32::from_rgb(38, 38, 42); // #26262A
const KEYCAP_HOVER: Color32 = Color32::from_rgb(52, 52, 58);
const KEYCAP_SPECULAR: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 26);
const SCREW_METAL: Color32 = Color32::from_rgb(174, 174, 178); // #AEAEB2
const SCREW_SLOT: Color32 = Color32::from_rgb(99, 99, 102); // #636366

/// Restituisce il colore corrispondente a un badge semantico Apple
pub fn badge_color(badge: &str) -> Color32 {
    match badge {
        "HID" => COLOR_APPLE_BLUE,
        "WIN" => COLOR_APPLE_GREEN,
        "WSP" => COLOR_APPLE_PURPLE,
        "MAC" => COLOR_APPLE_ORANGE,
        "CMD" => Color32::from_rgb(255, 214, 10), // Apple Yellow
        "CTX" => COLOR_APPLE_TEAL,
        _ => COLOR_TEXT_TERTIARY,
    }
}

pub struct PadCanvas {
    pub desired_size: Vec2,
}

impl Default for PadCanvas {
    fn default() -> Self {
        Self {
            desired_size: vec2(600.0, 370.0),
        }
    }
}

impl PadCanvas {
    pub fn new() -> Self {
        Self::default()
    }

    /// Disegna il canvas hardware fotorealistico in stile Cupertino Pro Dark
    pub fn show(&mut self, ui: &mut Ui, state: &mut UiState) -> Response {
        let (rect, response) = ui.allocate_exact_size(self.desired_size, Sense::hover());
        let painter = ui.painter_at(rect);

        // 1. Lastra Acrilico Fumè con bordo speculare e ombreggiatura
        painter.rect_filled(rect, Rounding::same(20.0), CHASSIS_ACRYLIC);
        painter.rect_stroke(
            rect,
            Rounding::same(20.0),
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(255, 255, 255, 34)),
        );

        let origin = rect.min;

        // 2. 4 Viti Angolari in Alluminio Satinato con intaglio a croce
        let screw_offset = 14.0;
        let screw_positions = [
            pos2(origin.x + screw_offset, origin.y + screw_offset),
            pos2(origin.x + rect.width() - screw_offset, origin.y + screw_offset),
            pos2(origin.x + screw_offset, origin.y + rect.height() - screw_offset),
            pos2(origin.x + rect.width() - screw_offset, origin.y + rect.height() - screw_offset),
        ];
        for sp in screw_positions {
            Self::draw_satin_screw(&painter, sp);
        }

        // Header superiore discreto
        painter.text(
            pos2(origin.x + 28.0, origin.y + 14.0),
            Align2::LEFT_TOP,
            "VORTEXDECK HARDWARE",
            FontId::proportional(10.5),
            COLOR_TEXT_TERTIARY,
        );

        let layer_str = format!("LAYER {}", state.editing_layer.display_number());
        painter.text(
            pos2(origin.x + 310.0, origin.y + 14.0),
            Align2::CENTER_TOP,
            layer_str,
            FontId::proportional(11.0),
            COLOR_APPLE_BLUE,
        );

        // -------------------------------------------------------------
        // SEZIONE 1: Matrice 4×3 Tasti (Sinistra)
        // -------------------------------------------------------------
        let key_area_origin = pos2(origin.x + 22.0, origin.y + 36.0);
        let key_size = 68.0;
        let key_gap = 10.0;

        for r in 0..ROW_COUNT as u8 {
            for c in 0..COL_COUNT as u8 {
                let slot = SlotId {
                    layer: state.editing_layer,
                    control: ControlId::Button { row: r, col: c },
                };

                let k_x = key_area_origin.x + (c as f32) * (key_size + key_gap);
                let k_y = key_area_origin.y + (r as f32) * (key_size + key_gap);
                let key_rect = Rect::from_min_size(pos2(k_x, k_y), vec2(key_size, key_size));

                let key_id = ui.make_persistent_id(format!("key_{}_{}_{}", state.editing_layer.0, r, c));
                let key_resp = ui.interact(key_rect, key_id, Sense::click());

                if key_resp.clicked() {
                    state.select_slot(slot);
                }

                Self::draw_sculpted_keycap(&painter, key_rect, &key_resp, slot, r, c, state);
            }
        }

        // -------------------------------------------------------------
        // SEZIONE 2: 3 LED Verticali (Centro)
        // -------------------------------------------------------------
        let led_center_x = origin.x + 348.0;
        let led_start_y = origin.y + 70.0;
        let led_spacing_y = 82.0;

        for l in 0..3u8 {
            let led_pos = pos2(led_center_x, led_start_y + (l as f32) * led_spacing_y);
            let is_active = state.editing_layer.0 == l;

            let led_id = ui.make_persistent_id(format!("led_layer_{}", l));
            let led_rect = Rect::from_center_size(led_pos, vec2(28.0, 36.0));
            let led_resp = ui.interact(led_rect, led_id, Sense::click());

            if led_resp.clicked() {
                state.set_editing_layer(LayerIndex(l));
            }

            Self::draw_capsule_led(&painter, led_pos, l + 1, is_active);
        }

        // -------------------------------------------------------------
        // SEZIONE 3: 2 Grandi Encoder Rotativi (Destra)
        // -------------------------------------------------------------
        let enc_center_x = origin.x + 478.0;
        let enc_radius = 42.0;
        let enc_y_positions = [origin.y + 90.0, origin.y + 248.0];

        for enc_idx in 0..2u8 {
            let enc_center = pos2(enc_center_x, enc_y_positions[enc_idx as usize]);
            Self::draw_brushed_knob(&painter, enc_center, enc_radius, enc_idx);

            // Segmented Control a 3 zone (↺ CCW, ● PRESS, ↻ CW)
            let ctrl_origin = pos2(enc_center.x - 72.0, enc_center.y + 50.0);
            let zone_w = 48.0;
            let zone_h = 24.0;

            let events = [
                (EncoderEvent::Ccw, "CCW", 0.0),
                (EncoderEvent::Press, "PUSH", 1.0),
                (EncoderEvent::Cw, "CW", 2.0),
            ];

            for (ev, label, offset_idx) in events {
                let slot = SlotId {
                    layer: state.editing_layer,
                    control: ControlId::Encoder {
                        index: enc_idx,
                        event: ev,
                    },
                };
                let zone_rect = Rect::from_min_size(
                    pos2(ctrl_origin.x + offset_idx * zone_w, ctrl_origin.y),
                    vec2(zone_w, zone_h),
                );

                let zone_id = ui.make_persistent_id(format!("enc_{}_{:?}", enc_idx, ev));
                let zone_resp = ui.interact(zone_rect, zone_id, Sense::click());

                if zone_resp.clicked() {
                    state.select_slot(slot);
                }

                let is_sel = state.selected_slot == Some(slot);
                let bg_color = if is_sel {
                    COLOR_APPLE_BLUE
                } else if zone_resp.hovered() {
                    COLOR_CONTROL_HOVER
                } else {
                    COLOR_CONTROL_INACTIVE
                };

                let rounding = if offset_idx == 0.0 {
                    Rounding { nw: 6.0, ne: 0.0, sw: 6.0, se: 0.0 }
                } else if offset_idx == 2.0 {
                    Rounding { nw: 0.0, ne: 6.0, sw: 0.0, se: 6.0 }
                } else {
                    Rounding::ZERO
                };

                painter.rect_filled(zone_rect, rounding, bg_color);
                painter.rect_stroke(zone_rect, rounding, Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE));

                let text_color = if is_sel { Color32::WHITE } else { COLOR_TEXT_PRIMARY };
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

    /// Disegna una vite angolare satinata in alluminio
    fn draw_satin_screw(painter: &Painter, center: Pos2) {
        painter.circle_filled(center, 4.0, SCREW_METAL);
        painter.circle_stroke(center, 4.0, Stroke::new(0.8_f32, Color32::from_rgb(140, 140, 145)));
        // Intaglio a croce
        painter.line_segment(
            [pos2(center.x - 2.2, center.y), pos2(center.x + 2.2, center.y)],
            Stroke::new(0.8_f32, SCREW_SLOT),
        );
        painter.line_segment(
            [pos2(center.x, center.y - 2.2), pos2(center.x, center.y + 2.2)],
            Stroke::new(0.8_f32, SCREW_SLOT),
        );
    }

    /// Disegna un keycap scolpito stile Magic Keyboard / PBT
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

        // Colore personalizzato se presente
        let custom_color = config
            .and_then(|c| c.color_hex.as_ref())
            .and_then(|h| hex_to_color32(h));

        // 1. Base Esterna
        let base_bg = if is_selected {
            Color32::from_rgb(32, 45, 68)
        } else if resp.hovered() {
            KEYCAP_HOVER
        } else {
            custom_color.unwrap_or(KEYCAP_BASE)
        };
        painter.rect_filled(rect, Rounding::same(12.0), base_bg);

        // 2. Piatto interno concavo leggermente incassato
        let dish_rect = rect.shrink(3.5);
        let dish_bg = if is_selected {
            Color32::from_rgb(26, 38, 58)
        } else {
            KEYCAP_DISH
        };
        painter.rect_filled(dish_rect, Rounding::same(9.0), dish_bg);

        // Riflesso speculare superiore
        let top_highlight = [
            pos2(dish_rect.min.x + 8.0, dish_rect.min.y + 1.5),
            pos2(dish_rect.max.x - 8.0, dish_rect.min.y + 1.5),
        ];
        painter.line_segment(top_highlight, Stroke::new(1.0_f32, KEYCAP_SPECULAR));

        // 3. Anello luminoso blu Apple per lo slot selezionato
        if is_selected {
            // Alone esterno glow
            painter.rect_stroke(
                rect.expand(1.5),
                Rounding::same(13.5),
                Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(10, 132, 255, 80)),
            );
            // Bordo primario
            painter.rect_stroke(rect, Rounding::same(12.0), Stroke::new(2.0_f32, COLOR_APPLE_BLUE));
        } else {
            painter.rect_stroke(rect, Rounding::same(12.0), Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE));
        }

        // 4. Indice Discreto in alto a sinistra (es. R1·C1)
        let idx_str = format!("R{}·C{}", row + 1, col + 1);
        painter.text(
            pos2(rect.min.x + 7.0, rect.min.y + 6.0),
            Align2::LEFT_TOP,
            idx_str,
            FontId::proportional(8.5),
            COLOR_TEXT_TERTIARY,
        );

        // 5. Etichetta Principale in Grassetto al centro
        let label_text = config.map(|c| c.label.as_str()).unwrap_or("—");
        let display_lbl = if label_text.len() > 8 {
            format!("{}..", &label_text[..7])
        } else {
            label_text.to_string()
        };
        painter.text(
            pos2(rect.center().x, rect.center().y - 1.0),
            Align2::CENTER_CENTER,
            display_lbl,
            FontId::proportional(12.5),
            COLOR_TEXT_PRIMARY,
        );

        // 6. Pill Badge tipologia in basso (es. HID o F13)
        let badge_text = state.get_badge_for_slot(slot);
        let b_color = badge_color(badge_text);

        let pill_w = 34.0;
        let pill_h = 13.0;
        let pill_rect = Rect::from_center_size(
            pos2(rect.center().x, rect.max.y - 10.0),
            vec2(pill_w, pill_h),
        );
        painter.rect_filled(pill_rect, Rounding::same(4.0), Color32::from_rgba_premultiplied(b_color.r() / 4, b_color.g() / 4, b_color.b() / 4, 180));
        painter.rect_stroke(pill_rect, Rounding::same(4.0), Stroke::new(0.8_f32, b_color));

        painter.text(
            pill_rect.center(),
            Align2::CENTER_CENTER,
            badge_text,
            FontId::proportional(8.5),
            b_color,
        );
    }

    /// Disegna una capsula LED con glow radiale
    fn draw_capsule_led(painter: &Painter, center: Pos2, layer_num: u8, is_active: bool) {
        let capsule_rect = Rect::from_center_size(center, vec2(10.0, 20.0));
        let (color, glow_color) = if is_active {
            (
                COLOR_APPLE_BLUE,
                Color32::from_rgba_premultiplied(10, 132, 255, 90),
            )
        } else {
            (
                Color32::from_rgb(58, 58, 62),
                Color32::TRANSPARENT,
            )
        };

        if is_active {
            painter.rect_filled(capsule_rect.expand(5.0), Rounding::same(10.0), glow_color);
        }

        painter.rect_filled(capsule_rect, Rounding::same(5.0), color);
        painter.rect_stroke(capsule_rect, Rounding::same(5.0), Stroke::new(1.0_f32, Color32::from_rgb(80, 80, 85)));

        painter.text(
            pos2(center.x, center.y + 16.0),
            Align2::CENTER_TOP,
            format!("L{}", layer_num),
            FontId::proportional(9.0),
            if is_active { COLOR_TEXT_PRIMARY } else { COLOR_TEXT_TERTIARY },
        );
    }

    /// Disegna una manopola rotativa circolare in alluminio spazzolato
    fn draw_brushed_knob(painter: &Painter, center: Pos2, radius: f32, enc_idx: u8) {
        // Anello metallico esterno
        painter.circle_filled(center, radius, Color32::from_rgb(52, 52, 56));
        painter.circle_stroke(center, radius, Stroke::new(1.5_f32, Color32::from_rgb(142, 142, 147)));

        // Tacche radiali esterne (12 tacche stile ghiera orologio)
        for i in 0..12 {
            let angle = (i as f32) * (2.0 * PI / 12.0);
            let p1 = pos2(center.x + angle.cos() * (radius - 5.0), center.y + angle.sin() * (radius - 5.0));
            let p2 = pos2(center.x + angle.cos() * (radius - 1.0), center.y + angle.sin() * (radius - 1.0));
            painter.line_segment([p1, p2], Stroke::new(1.0_f32, Color32::from_rgb(174, 174, 178)));
        }

        // Disco interno in alluminio concentrico
        painter.circle_filled(center, radius - 7.0, Color32::from_rgb(40, 40, 44));
        painter.circle_stroke(center, radius - 7.0, Stroke::new(1.0_f32, Color32::from_rgb(70, 70, 75)));

        // Linea indicatrice fisica (tacchetta ore 12)
        let indicator_p1 = pos2(center.x, center.y - 12.0);
        let indicator_p2 = pos2(center.x, center.y - (radius - 9.0));
        painter.line_segment([indicator_p1, indicator_p2], Stroke::new(2.0_f32, COLOR_APPLE_BLUE));

        // Testo encoder al centro
        let enc_title = if enc_idx == 0 { "ENC 1" } else { "ENC 2" };
        painter.text(
            center,
            Align2::CENTER_CENTER,
            enc_title,
            FontId::proportional(10.0),
            COLOR_TEXT_SECONDARY,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pad_canvas_default() {
        let canvas = PadCanvas::new();
        assert_eq!(canvas.desired_size.x, 600.0);
        assert_eq!(canvas.desired_size.y, 370.0);
    }

    #[test]
    fn test_badge_colors() {
        assert_eq!(badge_color("HID"), COLOR_APPLE_BLUE);
        assert_eq!(badge_color("WIN"), COLOR_APPLE_GREEN);
    }
}
