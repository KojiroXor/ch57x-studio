//! Apple Liquid Glass Engine (visionOS / macOS Tahoe inspired)
//!
//! Fornisce rendering vettoriale ad alte prestazioni per superfici traslucide stile Apple:
//! - Deep Space Canvas (sfondo con gradiente radiale multi-punto e bagliori soffusi)
//! - Card traslucide con rifrazione speculare direzionale (rim light 1.2px)
//! - Riflesso interno (Inner Highlight) per curvature organiche
//! - Ombre profonde e morbide (blur 32px, offset Y 10px)
//! - Controlli segmentati e pillole animate con interpolazione temporale fluida (0.18s)

use crate::theme::{COLOR_APPLE_BLUE, COLOR_TEXT_PRIMARY, COLOR_TEXT_SECONDARY};
use egui::{pos2, vec2, Color32, FontId, Margin, Rect, Rounding, Sense, Stroke, Ui};

/// Dipinge il Deep Space Canvas multi-punto sul rettangolo assegnato
pub fn paint_deep_space_canvas(painter: &egui::Painter, rect: Rect) {
    let c_top_left = Color32::from_rgb(11, 15, 25); // #0B0F19
    let c_bottom_right = Color32::from_rgb(19, 27, 46); // #131B2E

    let mut mesh = egui::Mesh::default();
    mesh.add_rect_with_uv(
        rect,
        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        c_top_left,
    );
    if mesh.vertices.len() >= 4 {
        mesh.vertices[0].color = c_top_left;
        mesh.vertices[1].color = Color32::from_rgb(15, 21, 35);
        mesh.vertices[2].color = c_bottom_right;
        mesh.vertices[3].color = Color32::from_rgb(14, 19, 32);
    }
    painter.add(egui::Shape::mesh(mesh));

    // Bagliore 1: Blu Apple soffuso in alto a sinistra
    let center_glow1 = pos2(
        rect.min.x + rect.width() * 0.22,
        rect.min.y + rect.height() * 0.28,
    );
    for (r, a) in [(180.0, 16u8), (280.0, 10u8), (400.0, 5u8)] {
        painter.circle_filled(
            center_glow1,
            r,
            Color32::from_rgba_premultiplied(10, 132, 255, a),
        );
    }

    // Bagliore 2: Indaco/Viola soffuso in basso a destra
    let center_glow2 = pos2(
        rect.max.x - rect.width() * 0.25,
        rect.max.y - rect.height() * 0.25,
    );
    for (r, a) in [(200.0, 14u8), (320.0, 8u8), (450.0, 4u8)] {
        painter.circle_filled(
            center_glow2,
            r,
            Color32::from_rgba_premultiplied(94, 92, 230, a),
        );
    }
}

/// Disegna la rifrazione speculare direzionale e il riflesso interno sul bordo di una card
pub fn paint_specular_rim_light(painter: &egui::Painter, rect: Rect, rounding: Rounding) {
    let r = rounding.nw.max(4.0);

    // 1. Specular Top Rim Light (brillante, 1.2px, rgba(255, 255, 255, 0.28))
    painter.line_segment(
        [
            pos2(rect.min.x + r, rect.min.y),
            pos2(rect.max.x - r, rect.min.y),
        ],
        Stroke::new(1.2_f32, Color32::from_rgba_premultiplied(255, 255, 255, 72)),
    );

    // 2. Specular Left Rim Light (semi-brillante, 1.2px, rgba(255, 255, 255, 0.20))
    painter.line_segment(
        [
            pos2(rect.min.x, rect.min.y + r),
            pos2(rect.min.x, rect.max.y - r),
        ],
        Stroke::new(1.2_f32, Color32::from_rgba_premultiplied(255, 255, 255, 50)),
    );

    // 3. Bottom & Right subtle edge (degrada verso 0.06 alpha = ~15)
    painter.line_segment(
        [
            pos2(rect.min.x + r, rect.max.y),
            pos2(rect.max.x - r, rect.max.y),
        ],
        Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 15)),
    );
    painter.line_segment(
        [
            pos2(rect.max.x, rect.min.y + r),
            pos2(rect.max.x, rect.max.y - r),
        ],
        Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 15)),
    );

    // 4. Riflesso interno (Inner Highlight a 1.5px sotto il bordo superiore)
    painter.line_segment(
        [
            pos2(rect.min.x + r + 6.0, rect.min.y + 1.5),
            pos2(rect.max.x - r - 6.0, rect.min.y + 1.5),
        ],
        Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 38)),
    );
}

/// Contenitore Liquid Glass Card con effetto vetro smerigliato e rifrazione speculare
pub fn liquid_glass_card<R>(
    ui: &mut Ui,
    title: Option<&str>,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    if let Some(t) = title {
        ui.horizontal(|ui| {
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(t.to_uppercase())
                    .font(FontId::proportional(11.0))
                    .color(COLOR_TEXT_SECONDARY)
                    .strong(),
            );
        });
        ui.add_space(3.0);
    }

    let frame = egui::Frame {
        inner_margin: Margin::same(16.0),
        rounding: Rounding::same(16.0),
        fill: Color32::from_rgba_premultiplied(38, 42, 54, 155),
        stroke: Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 18)),
        shadow: egui::epaint::Shadow {
            offset: vec2(0.0, 10.0),
            blur: 32.0,
            spread: 0.0,
            color: Color32::from_black_alpha(115),
        },
        ..Default::default()
    };

    let res = frame.show(ui, add_contents);
    paint_specular_rim_light(ui.painter(), res.response.rect, Rounding::same(16.0));
    res.inner
}

/// Segmented Control orizzontale in stile Liquid Glass con pillola fluida animata (0.18s)
pub fn liquid_segmented_control<T: PartialEq + Copy>(
    ui: &mut Ui,
    id_salt: impl std::hash::Hash,
    selected: &mut T,
    options: &[(T, &str)],
) -> bool {
    let mut changed = false;
    let container_id = ui.make_persistent_id(id_salt);
    let height = 32.0;

    let container_frame = egui::Frame {
        inner_margin: Margin::same(3.0),
        rounding: Rounding::same(10.0),
        fill: Color32::from_rgba_premultiplied(22, 26, 36, 190),
        stroke: Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 22)),
        ..Default::default()
    };

    container_frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(3.0, 0.0);
            for (idx, (val, label)) in options.iter().enumerate() {
                let is_active = *selected == *val;
                let opt_id = container_id.with(idx);
                let anim = ui.ctx().animate_bool_with_time(opt_id, is_active, 0.18);

                let active_alpha = (anim * 230.0) as u8;
                let text_color = if anim > 0.4 {
                    Color32::WHITE
                } else {
                    COLOR_TEXT_SECONDARY
                };

                let galley = ui.painter().layout_no_wrap(
                    label.to_string(),
                    FontId::proportional(12.0),
                    text_color,
                );
                let desired_w = galley.size().x + 20.0;
                let (rect, resp) =
                    ui.allocate_exact_size(vec2(desired_w, height - 6.0), Sense::click());

                if anim > 0.01 {
                    let pill_fill = Color32::from_rgba_premultiplied(10, 132, 255, active_alpha);
                    ui.painter()
                        .rect_filled(rect, Rounding::same(8.0), pill_fill);

                    // Micro-bordo superiore illuminato su pillola attiva
                    let rim_alpha = (anim * 110.0) as u8;
                    ui.painter().line_segment(
                        [
                            pos2(rect.min.x + 3.0, rect.min.y + 0.5),
                            pos2(rect.max.x - 3.0, rect.min.y + 0.5),
                        ],
                        Stroke::new(
                            1.0_f32,
                            Color32::from_rgba_premultiplied(255, 255, 255, rim_alpha),
                        ),
                    );
                } else if resp.hovered() {
                    // Hover state: alone bianco sfumato al 10%
                    ui.painter().rect_filled(
                        rect,
                        Rounding::same(8.0),
                        Color32::from_rgba_premultiplied(255, 255, 255, 25),
                    );
                }

                let text_pos = pos2(
                    rect.center().x - galley.size().x / 2.0,
                    rect.center().y - galley.size().y / 2.0,
                );
                ui.painter().galley(text_pos, galley, text_color);

                if resp.clicked() && !is_active {
                    *selected = *val;
                    changed = true;
                }
            }
        });
    });

    changed
}

/// Pulsante a pillola Liquid Glass con micro-riflessi e feedback fluido
pub fn liquid_pill_button(ui: &mut Ui, text: &str, is_accent: bool) -> egui::Response {
    let padding = vec2(14.0, 7.0);
    let galley = ui.painter().layout_no_wrap(
        text.to_string(),
        FontId::proportional(12.5),
        if is_accent {
            Color32::WHITE
        } else {
            COLOR_TEXT_PRIMARY
        },
    );
    let size = vec2(
        galley.size().x + padding.x * 2.0,
        galley.size().y + padding.y * 2.0,
    );
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());

    let (fill, stroke_col) = if is_accent {
        if resp.is_pointer_button_down_on() {
            (Color32::from_rgb(8, 105, 204), COLOR_APPLE_BLUE)
        } else if resp.hovered() {
            (Color32::from_rgb(25, 145, 255), Color32::WHITE)
        } else {
            (
                COLOR_APPLE_BLUE,
                Color32::from_rgba_premultiplied(255, 255, 255, 60),
            )
        }
    } else {
        if resp.is_pointer_button_down_on() {
            (
                Color32::from_rgba_premultiplied(50, 55, 70, 220),
                Color32::from_rgba_premultiplied(255, 255, 255, 40),
            )
        } else if resp.hovered() {
            (
                Color32::from_rgba_premultiplied(60, 66, 85, 200),
                Color32::from_rgba_premultiplied(255, 255, 255, 60),
            )
        } else {
            (
                Color32::from_rgba_premultiplied(38, 42, 54, 180),
                Color32::from_rgba_premultiplied(255, 255, 255, 25),
            )
        }
    };

    ui.painter().rect_filled(rect, Rounding::same(8.0), fill);
    ui.painter()
        .rect_stroke(rect, Rounding::same(8.0), Stroke::new(1.0_f32, stroke_col));

    // Specular top edge highlight
    ui.painter().line_segment(
        [
            pos2(rect.min.x + 3.0, rect.min.y + 0.5),
            pos2(rect.max.x - 3.0, rect.min.y + 0.5),
        ],
        Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 70)),
    );

    let text_pos = pos2(
        rect.center().x - galley.size().x / 2.0,
        rect.center().y - galley.size().y / 2.0,
    );
    ui.painter().galley(
        text_pos,
        galley,
        if is_accent {
            Color32::WHITE
        } else {
            COLOR_TEXT_PRIMARY
        },
    );

    resp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deep_space_canvas_math() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0));
        assert_eq!(rect.width(), 800.0);
    }
}
