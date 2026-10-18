//! Pad canvas vector drawing widget faithful to Macropad VortexDeck
//!
//! Visualizza:
//! - Griglia 4 colonne x 3 righe = 12 tasti quadrati a sinistra con etichetta, badge colorato e token trigger.
//! - 3 LED verticali al centro per il monitoraggio e commutazione dei Layer (1..3).
//! - 2 grandi Encoder circolari a destra suddivisi in 3 aree cliccabili:
//!   - ↺ CCW (rotazione antioraria)
//!   - ● PRESS (click manopola)
//!   - ↻ CW (rotazione oraria)
//! - Evidenziazione dello slot selezionato e animazione/feedback visivo al click.

use crate::state::{hex_to_color32, UiState};
use ch57x_domain::hardware::{
    ControlId, EncoderEvent, LayerIndex, SlotId, COL_COUNT, ROW_COUNT,
};
use egui::{
    pos2, vec2, Align2, Color32, FontId, Pos2, Rect, Response, Rounding, Sense, Shape, Stroke, Ui,
    Vec2,
};
use std::f32::consts::{FRAC_PI_2, PI};

/// Colore di sfondo del canvas e dello chassis
const CHASSIS_BG: Color32 = Color32::from_rgb(24, 24, 27); // Zinc 900
const CHASSIS_STROKE: Color32 = Color32::from_rgb(63, 63, 70); // Zinc 700
const KEY_DEFAULT_BG: Color32 = Color32::from_rgb(39, 39, 42); // Zinc 800
const KEY_HOVER_BG: Color32 = Color32::from_rgb(50, 50, 56);
const KEY_BORDER_DEFAULT: Color32 = Color32::from_rgb(82, 82, 91); // Zinc 600
const SELECTED_GLOW: Color32 = Color32::from_rgb(56, 189, 248); // Sky 400
const CLICK_FLASH: Color32 = Color32::from_rgb(254, 240, 138); // Yellow 200

/// Restituisce il colore corrispondente a un badge
pub fn badge_color(badge: &str) -> Color32 {
    match badge {
        "HID" => Color32::from_rgb(37, 99, 235), // Blue 600
        "WIN" => Color32::from_rgb(22, 163, 74), // Green 600
        "WSP" => Color32::from_rgb(147, 51, 234), // Purple 600
        "MAC" => Color32::from_rgb(234, 88, 12), // Orange 600
        "CMD" => Color32::from_rgb(217, 119, 6), // Amber 600
        "CTX" => Color32::from_rgb(13, 148, 136), // Teal 600
        _ => Color32::from_rgb(82, 82, 91),      // Gray
    }
}

pub struct PadCanvas {
    pub desired_size: Vec2,
}

impl Default for PadCanvas {
    fn default() -> Self {
        Self {
            desired_size: vec2(580.0, 360.0),
        }
    }
}

impl PadCanvas {
    pub fn new() -> Self {
        Self::default()
    }

    /// Disegna il widget PadCanvas completo integrandosi nel loop UI di egui
    pub fn show(&mut self, ui: &mut Ui, state: &mut UiState) -> Response {
        let (rect, response) = ui.allocate_exact_size(self.desired_size, Sense::hover());
        let painter = ui.painter_at(rect);

        // 1. Chassis Background (VortexDeck Hardware Pad Case)
        painter.rect_filled(rect, Rounding::same(16.0), CHASSIS_BG);
        painter.rect_stroke(rect, Rounding::same(16.0), Stroke::new(1.5_f32, CHASSIS_STROKE));

        // Dimensionamento dinamico basato sul rettangolo allocato
        let origin = rect.min;

        // Disegna branding sottile in alto
        let brand_pos = pos2(origin.x + 20.0, origin.y + 14.0);
        painter.text(
            brand_pos,
            Align2::LEFT_TOP,
            "VORTEXDECK CH57x",
            FontId::proportional(11.0),
            Color32::from_rgb(161, 161, 170),
        );

        let layer_label = format!("ACTIVE: LAYER {}", state.editing_layer.display_number());
        painter.text(
            pos2(origin.x + rect.width() - 20.0, origin.y + 14.0),
            Align2::RIGHT_TOP,
            layer_label,
            FontId::proportional(11.0),
            Color32::from_rgb(56, 189, 248),
        );

        // -------------------------------------------------------------
        // SEZIONE 1: 12 Tasti quadrati a sinistra (4 colonne x 3 righe)
        // -------------------------------------------------------------
        let key_area_origin = pos2(origin.x + 20.0, origin.y + 36.0);
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

                // Renderizza tasto singolo
                self.draw_button_key(&painter, key_rect, &key_resp, slot, state);
            }
        }

        // -------------------------------------------------------------
        // SEZIONE 2: 3 LED Verticali al centro (Indicatori Layer)
        // -------------------------------------------------------------
        let led_center_x = origin.x + 348.0;
        let led_start_y = origin.y + 70.0;
        let led_spacing_y = 80.0;

        for l in 0..3u8 {
            let led_pos = pos2(led_center_x, led_start_y + (l as f32) * led_spacing_y);
            let is_active = state.editing_layer.0 == l;

            let led_id = ui.make_persistent_id(format!("led_layer_{}", l));
            let led_rect = Rect::from_center_size(led_pos, vec2(24.0, 24.0));
            let led_resp = ui.interact(led_rect, led_id, Sense::click());

            if led_resp.clicked() {
                state.set_editing_layer(LayerIndex(l));
            }

            // Glow esterno se attivo
            if is_active {
                painter.circle_filled(led_pos, 10.0, Color32::from_rgba_unmultiplied(34, 197, 94, 60));
                painter.circle_filled(led_pos, 6.0, Color32::from_rgb(34, 197, 94));
                painter.circle_stroke(led_pos, 6.0, Stroke::new(1.5_f32, Color32::WHITE));
            } else {
                let dim_col = if led_resp.hovered() {
                    Color32::from_rgb(60, 60, 65)
                } else {
                    Color32::from_rgb(39, 39, 42)
                };
                painter.circle_filled(led_pos, 5.0, dim_col);
                painter.circle_stroke(led_pos, 5.0, Stroke::new(1.0_f32, Color32::from_rgb(82, 82, 91)));
            }

            // Etichetta LED
            painter.text(
                pos2(led_pos.x, led_pos.y + 14.0),
                Align2::CENTER_TOP,
                format!("L{}", l + 1),
                FontId::proportional(10.0),
                if is_active {
                    Color32::from_rgb(244, 244, 245)
                } else {
                    Color32::from_rgb(113, 113, 122)
                },
            );
        }

        // -------------------------------------------------------------
        // SEZIONE 3: 2 Grandi Encoder circolari a destra
        // -------------------------------------------------------------
        let knob_center_x = origin.x + 465.0;
        let knob_centers = [
            pos2(knob_center_x, origin.y + 105.0), // Knob 0 (Top)
            pos2(knob_center_x, origin.y + 245.0), // Knob 1 (Bottom)
        ];

        for (enc_idx, &center) in knob_centers.iter().enumerate() {
            self.draw_circular_encoder(ui, &painter, center, enc_idx as u8, state);
        }

        response
    }

    /// Disegna un singolo tasto quadrato con badge, etichetta e token trigger
    fn draw_button_key(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        resp: &Response,
        slot: SlotId,
        state: &UiState,
    ) {
        let is_selected = state.selected_slot == Some(slot);
        let is_recent_click = state.is_slot_recently_clicked(slot);
        let config = state.get_slot_config(slot);

        // Colore di riempimento di base (supporta custom color_hex)
        let mut bg_color = if let Some(cfg) = config {
            if let Some(hex) = &cfg.color_hex {
                hex_to_color32(hex).unwrap_or(KEY_DEFAULT_BG)
            } else {
                KEY_DEFAULT_BG
            }
        } else {
            KEY_DEFAULT_BG
        };

        if resp.hovered() && !is_selected {
            bg_color = KEY_HOVER_BG;
        }

        if is_recent_click {
            bg_color = CLICK_FLASH;
        }

        // Bordo / Stroke
        let stroke = if is_selected {
            Stroke::new(2.5_f32, SELECTED_GLOW)
        } else if resp.hovered() {
            Stroke::new(1.5_f32, Color32::from_rgb(161, 161, 170))
        } else {
            Stroke::new(1.0_f32, KEY_BORDER_DEFAULT)
        };

        // Ombreggiatura o glow per lo slot selezionato
        if is_selected {
            let glow_rect = rect.expand(2.5);
            painter.rect_stroke(
                glow_rect,
                Rounding::same(10.0),
                Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(56, 189, 248, 120)),
            );
        }

        painter.rect_filled(rect, Rounding::same(8.0), bg_color);
        painter.rect_stroke(rect, Rounding::same(8.0), stroke);

        // Badge pill in alto a sinistra
        let badge_text = state.get_badge_for_slot(slot);
        let badge_bg = badge_color(badge_text);
        let pill_rect = Rect::from_min_size(
            pos2(rect.min.x + 5.0, rect.min.y + 5.0),
            vec2(26.0, 14.0),
        );
        painter.rect_filled(pill_rect, Rounding::same(3.0), badge_bg);
        painter.text(
            pill_rect.center(),
            Align2::CENTER_CENTER,
            badge_text,
            FontId::proportional(9.0),
            Color32::WHITE,
        );

        // Etichetta del tasto al centro
        let label_text = config
            .map(|c| c.label.as_str())
            .unwrap_or_else(|| "Key");
        let display_label = if label_text.len() > 8 {
            format!("{}...", &label_text[..7])
        } else {
            label_text.to_string()
        };

        let text_color = if is_recent_click {
            Color32::BLACK
        } else {
            Color32::from_rgb(244, 244, 245)
        };

        painter.text(
            pos2(rect.center().x, rect.center().y + 2.0),
            Align2::CENTER_CENTER,
            display_label,
            FontId::proportional(12.0),
            text_color,
        );

        // Token trigger in basso (es. "f13", "ctrl-f14")
        if let Some(token) = state.get_trigger_token_for_slot(slot) {
            painter.text(
                pos2(rect.center().x, rect.max.y - 4.0),
                Align2::CENTER_BOTTOM,
                token,
                FontId::proportional(9.0),
                Color32::from_rgb(161, 161, 170),
            );
        }
    }

    /// Disegna un encoder circolare suddiviso in 3 aree cliccabili:
    /// - CCW (semi-corona circolare sinistra)
    /// - PRESS (cerchio centrale)
    /// - CW (semi-corona circolare destra)
    fn draw_circular_encoder(
        &self,
        ui: &mut Ui,
        painter: &egui::Painter,
        center: Pos2,
        enc_idx: u8,
        state: &mut UiState,
    ) {
        let radius_outer = 52.0;
        let radius_inner = 23.0;

        let slot_ccw = SlotId {
            layer: state.editing_layer,
            control: ControlId::Encoder {
                index: enc_idx,
                event: EncoderEvent::Ccw,
            },
        };
        let slot_press = SlotId {
            layer: state.editing_layer,
            control: ControlId::Encoder {
                index: enc_idx,
                event: EncoderEvent::Press,
            },
        };
        let slot_cw = SlotId {
            layer: state.editing_layer,
            control: ControlId::Encoder {
                index: enc_idx,
                event: EncoderEvent::Cw,
            },
        };

        let is_selected_ccw = state.selected_slot == Some(slot_ccw);
        let is_selected_press = state.selected_slot == Some(slot_press);
        let is_selected_cw = state.selected_slot == Some(slot_cw);

        // Bounding box per l'interazione intero encoder
        let bounds = Rect::from_center_size(center, vec2(radius_outer * 2.0 + 8.0, radius_outer * 2.0 + 8.0));
        let enc_id = ui.make_persistent_id(format!("encoder_{}_{}", state.editing_layer.0, enc_idx));
        let enc_resp = ui.interact(bounds, enc_id, Sense::click());

        // Hit-testing circolare per determinare quale delle 3 zone è stata cliccata o puntata
        let mut hovered_event: Option<EncoderEvent> = None;
        if let Some(hover_pos) = enc_resp.hover_pos() {
            let offset = hover_pos - center;
            let dist = offset.length();
            if dist <= radius_inner {
                hovered_event = Some(EncoderEvent::Press);
            } else if dist <= radius_outer {
                if offset.x < 0.0 {
                    hovered_event = Some(EncoderEvent::Ccw);
                } else {
                    hovered_event = Some(EncoderEvent::Cw);
                }
            }
        }

        if enc_resp.clicked() {
            if let Some(event) = hovered_event {
                let clicked_slot = match event {
                    EncoderEvent::Ccw => slot_ccw,
                    EncoderEvent::Press => slot_press,
                    EncoderEvent::Cw => slot_cw,
                };
                state.select_slot(clicked_slot);
            }
        }

        // Disegna ombra/bezel della corona esterna
        painter.circle_filled(center, radius_outer + 3.0, Color32::from_rgb(18, 18, 20));
        painter.circle_stroke(
            center,
            radius_outer + 3.0,
            Stroke::new(1.0_f32, Color32::from_rgb(63, 63, 70)),
        );

        // 1. Semi-corona Sinistra (↺ CCW)
        let ccw_bg = if is_selected_ccw {
            Color32::from_rgb(45, 75, 110)
        } else if hovered_event == Some(EncoderEvent::Ccw) {
            Color32::from_rgb(55, 55, 62)
        } else {
            KEY_DEFAULT_BG
        };
        let ccw_stroke = if is_selected_ccw {
            Stroke::new(2.5_f32, SELECTED_GLOW)
        } else {
            Stroke::new(1.0_f32, KEY_BORDER_DEFAULT)
        };
        draw_semi_annulus(
            painter,
            SemiAnnulusParams {
                center,
                r_in: radius_inner,
                r_out: radius_outer,
                a_start: FRAC_PI_2,
                a_end: FRAC_PI_2 + PI,
            },
            ccw_bg,
            ccw_stroke,
        );

        // 2. Semi-corona Destra (↻ CW)
        let cw_bg = if is_selected_cw {
            Color32::from_rgb(45, 75, 110)
        } else if hovered_event == Some(EncoderEvent::Cw) {
            Color32::from_rgb(55, 55, 62)
        } else {
            KEY_DEFAULT_BG
        };
        let cw_stroke = if is_selected_cw {
            Stroke::new(2.5_f32, SELECTED_GLOW)
        } else {
            Stroke::new(1.0_f32, KEY_BORDER_DEFAULT)
        };
        draw_semi_annulus(
            painter,
            SemiAnnulusParams {
                center,
                r_in: radius_inner,
                r_out: radius_outer,
                a_start: -FRAC_PI_2,
                a_end: FRAC_PI_2,
            },
            cw_bg,
            cw_stroke,
        );

        // 3. Cerchio Centrale (● PRESS)
        let press_bg = if is_selected_press {
            Color32::from_rgb(56, 189, 248)
        } else if hovered_event == Some(EncoderEvent::Press) {
            Color32::from_rgb(70, 70, 80)
        } else {
            Color32::from_rgb(30, 30, 34)
        };
        let press_stroke = if is_selected_press {
            Stroke::new(2.5_f32, Color32::WHITE)
        } else {
            Stroke::new(1.5_f32, Color32::from_rgb(113, 113, 122))
        };
        painter.circle_filled(center, radius_inner, press_bg);
        painter.circle_stroke(center, radius_inner, press_stroke);

        // -------------------------------------------------------------
        // Etichette e Icone per le 3 zone
        // -------------------------------------------------------------
        // Testo CCW
        let ccw_text_pos = pos2(center.x - (radius_outer + radius_inner) * 0.48, center.y);
        painter.text(
            ccw_text_pos,
            Align2::CENTER_CENTER,
            "↺ CCW",
            FontId::proportional(10.0),
            if is_selected_ccw {
                Color32::WHITE
            } else {
                Color32::from_rgb(212, 212, 216)
            },
        );

        // Testo CW
        let cw_text_pos = pos2(center.x + (radius_outer + radius_inner) * 0.48, center.y);
        painter.text(
            cw_text_pos,
            Align2::CENTER_CENTER,
            "↻ CW",
            FontId::proportional(10.0),
            if is_selected_cw {
                Color32::WHITE
            } else {
                Color32::from_rgb(212, 212, 216)
            },
        );

        // Testo PRESS
        painter.text(
            center,
            Align2::CENTER_CENTER,
            "● PRESS",
            FontId::proportional(9.0),
            if is_selected_press {
                Color32::BLACK
            } else {
                Color32::from_rgb(244, 244, 245)
            },
        );

        // Titolo Knob sopra/sotto
        let label_name = if enc_idx == 0 { "ENC 1 (Top)" } else { "ENC 2 (Bot)" };
        painter.text(
            pos2(center.x, center.y - radius_outer - 5.0),
            Align2::CENTER_BOTTOM,
            label_name,
            FontId::proportional(10.0),
            Color32::from_rgb(161, 161, 170),
        );
    }
}

pub struct SemiAnnulusParams {
    pub center: Pos2,
    pub r_in: f32,
    pub r_out: f32,
    pub a_start: f32,
    pub a_end: f32,
}

/// Disegna un arco/settore circolare (semi-corona) tra angoli `a_start` e `a_end`
fn draw_semi_annulus(
    painter: &egui::Painter,
    params: SemiAnnulusParams,
    fill_col: Color32,
    stroke: Stroke,
) {
    const SEGMENTS: usize = 16;
    let mut points = Vec::with_capacity(SEGMENTS * 2 + 2);

    // Arco esterno (in senso orario)
    for i in 0..=SEGMENTS {
        let t = i as f32 / SEGMENTS as f32;
        let angle = params.a_start + (params.a_end - params.a_start) * t;
        points.push(pos2(
            params.center.x + angle.cos() * params.r_out,
            params.center.y + angle.sin() * params.r_out,
        ));
    }

    // Arco interno (in senso antiorario per chiudere il poligono)
    for i in (0..=SEGMENTS).rev() {
        let t = i as f32 / SEGMENTS as f32;
        let angle = params.a_start + (params.a_end - params.a_start) * t;
        points.push(pos2(
            params.center.x + angle.cos() * params.r_in,
            params.center.y + angle.sin() * params.r_in,
        ));
    }

    painter.add(Shape::convex_polygon(points, fill_col, stroke));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pad_canvas_default() {
        let canvas = PadCanvas::default();
        assert!(canvas.desired_size.x >= 500.0);
        assert!(canvas.desired_size.y >= 300.0);
    }

    #[test]
    fn test_badge_colors() {
        assert_eq!(badge_color("HID"), Color32::from_rgb(37, 99, 235));
        assert_eq!(badge_color("WIN"), Color32::from_rgb(22, 163, 74));
        assert_eq!(badge_color("WSP"), Color32::from_rgb(147, 51, 234));
        assert_eq!(badge_color("MAC"), Color32::from_rgb(234, 88, 12));
        assert_eq!(badge_color("CMD"), Color32::from_rgb(217, 119, 6));
        assert_eq!(badge_color("CTX"), Color32::from_rgb(13, 148, 136));
        assert_eq!(badge_color("OTHER"), Color32::from_rgb(82, 82, 91));
    }
}
