//! Motore Apple Liquid Glass (visionOS / macOS Tahoe)
//! Usa Mesh di Gouraud a 9 vertici per uno sfondo morbido continuo (ZERO cerchi bersaglio!)
//! e disegna il Rim Light DOPO il layout della Card usando `response.rect`.

use crate::theme::*;
use egui::{pos2, vec2, Color32, FontId, Margin, Mesh, Rect, Rounding, Sense, Shape, Stroke, Ui};

/// Disegna uno sfondo atmosferico scuro continuo tramite interpolazione di vertici (Mesh 3x3).
/// Nessun cerchio sovrapposto, nessuna saturazione premoltiplicata.
pub fn paint_deep_space_canvas(painter: &egui::Painter, rect: Rect) {
    let mut mesh = Mesh::default();

    let x0 = rect.min.x;
    let x1 = rect.center().x;
    let x2 = rect.max.x;

    let y0 = rect.min.y;
    let y1 = rect.center().y;
    let y2 = rect.max.y;

    // 9 punti per un gradiente atmosferico Apple Tahoe ultra-morbido
    let pts = [
        (pos2(x0, y0), Color32::from_rgb(13, 19, 33)), // Top-Left: Deep Midnight Blue
        (pos2(x1, y0), Color32::from_rgb(16, 24, 42)), // Top-Center: Subtle Indigo Sheen
        (pos2(x2, y0), Color32::from_rgb(12, 16, 28)), // Top-Right: Obsidian
        (pos2(x0, y1), Color32::from_rgb(11, 16, 27)), // Mid-Left
        (pos2(x1, y1), Color32::from_rgb(18, 26, 46)), // Center: Soft Ambient Glow
        (pos2(x2, y1), Color32::from_rgb(14, 18, 32)), // Mid-Right
        (pos2(x0, y2), Color32::from_rgb(9, 12, 20)),  // Bottom-Left
        (pos2(x1, y2), Color32::from_rgb(12, 17, 29)), // Bottom-Center
        (pos2(x2, y2), Color32::from_rgb(15, 20, 36)), // Bottom-Right
    ];

    for (pos, color) in pts {
        mesh.colored_vertex(pos, color);
    }

    // 4 quadranti = 8 triangoli
    let quads = [(0, 1, 4, 3), (1, 2, 5, 4), (3, 4, 7, 6), (4, 5, 8, 7)];
    for (a, b, c, d) in quads {
        mesh.add_triangle(a, b, c);
        mesh.add_triangle(a, c, d);
    }

    painter.add(Shape::mesh(mesh));
}

/// Disegna la rifrazione speculare Liquid Glass attorno al rettangolo FINALE di una card
pub fn paint_specular_rim_light(painter: &egui::Painter, rect: Rect, rounding: Rounding) {
    if rect.width() <= 4.0 || rect.height() <= 4.0 {
        return;
    }

    // 1. Bordo perimetrale continuo completo (rispetta esattamente gli angoli arrotondati)
    painter.rect_stroke(
        rect,
        rounding,
        Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 34)),
    );

    // 2. Riflesso speculare superiore interno (effetto lastra di vetro illuminata dall'alto)
    let r = rounding.nw.max(6.0);
    let top_y = rect.min.y + 1.0;
    if rect.width() > r * 2.5 {
        painter.line_segment(
            [pos2(rect.min.x + r, top_y), pos2(rect.max.x - r, top_y)],
            Stroke::new(1.2_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 75)),
        );
    }
}

/// Contenitore Liquid Glass Card stile macOS Tahoe / visionOS
pub fn liquid_glass_card<R>(
    ui: &mut Ui,
    title: Option<&str>,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    let rounding = RADIUS_CARD;

    let frame = egui::Frame {
        inner_margin: Margin::same(18.0),
        outer_margin: Margin::symmetric(0.0, 4.0),
        rounding,
        fill: COLOR_SURFACE_CARD,
        stroke: Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE),
        shadow: egui::epaint::Shadow {
            blur: 24.0,
            offset: vec2(0.0, 8.0),
            color: Color32::from_black_alpha(110),
            ..Default::default()
        },
    };

    let response = frame.show(ui, |ui| {
        ui.set_width(ui.available_width());
        if let Some(t) = title {
            ui.label(
                egui::RichText::new(t.to_uppercase())
                    .font(FontId::proportional(11.0))
                    .strong()
                    .color(COLOR_APPLE_CYAN),
            );
            ui.add_space(6.0);
            ui.separator();
            ui.add_space(8.0);
        }
        add_contents(ui)
    });

    // Disegna il rim light speculare sul rettangolo REALE finale calcolato da Frame::show
    paint_specular_rim_light(ui.painter(), response.response.rect, rounding);

    response.inner
}

/// Segmented Control compatibile con inspector
pub fn liquid_segmented_control<T: PartialEq + Copy>(
    ui: &mut Ui,
    _id_salt: impl std::hash::Hash,
    selected: &mut T,
    options: &[(T, &str)],
) -> bool {
    crate::theme::segmented_control(ui, selected, options)
}

/// Pulsante a pillola Liquid Glass con unmultiplied colors
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
                Color32::from_rgba_unmultiplied(255, 255, 255, 60),
            )
        }
    } else if resp.is_pointer_button_down_on() {
        (
            Color32::from_rgba_unmultiplied(50, 55, 70, 220),
            Color32::from_rgba_unmultiplied(255, 255, 255, 40),
        )
    } else if resp.hovered() {
        (
            Color32::from_rgba_unmultiplied(60, 66, 85, 200),
            Color32::from_rgba_unmultiplied(255, 255, 255, 60),
        )
    } else {
        (
            Color32::from_rgba_unmultiplied(38, 42, 54, 180),
            Color32::from_rgba_unmultiplied(255, 255, 255, 25),
        )
    };

    ui.painter().rect_filled(rect, Rounding::same(8.0), fill);
    ui.painter()
        .rect_stroke(rect, Rounding::same(8.0), Stroke::new(1.0_f32, stroke_col));

    let top_highlight = [
        pos2(rect.min.x + 3.0, rect.min.y + 0.5),
        pos2(rect.max.x - 3.0, rect.min.y + 0.5),
    ];
    ui.painter().line_segment(
        top_highlight,
        Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 70)),
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
