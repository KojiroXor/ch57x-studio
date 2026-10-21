//! Motore grafico Apple Liquid Glass
//! Nessun bordo sporgente oltre il raggio delle card e layout verticale garantito.

use crate::theme::*;
use egui::epaint::{Mesh, Vertex};
use egui::{
    pos2, vec2, Color32, FontId, Margin, Painter, Pos2, Rect, Response, Rounding, Stroke, Ui,
};

/// Sfondo atmosferico morbido a 9 vertici (Gouraud shading).
pub fn paint_deep_space_canvas(painter: &Painter, rect: Rect) {
    let mut mesh = Mesh::default();

    let c_top_left = Color32::from_rgb(15, 20, 32);
    let c_top_center = Color32::from_rgb(20, 28, 46);
    let c_top_right = Color32::from_rgb(14, 18, 28);
    let c_mid = Color32::from_rgb(13, 17, 26);
    let c_center = Color32::from_rgb(18, 25, 40);
    let c_bot = Color32::from_rgb(10, 12, 18);

    let l = rect.left();
    let r = rect.right();
    let t = rect.top();
    let b = rect.bottom();
    let cx = rect.center().x;
    let cy = rect.center().y;

    mesh.vertices.extend([
        Vertex {
            pos: pos2(l, t),
            uv: Pos2::ZERO,
            color: c_top_left,
        },
        Vertex {
            pos: pos2(cx, t),
            uv: Pos2::ZERO,
            color: c_top_center,
        },
        Vertex {
            pos: pos2(r, t),
            uv: Pos2::ZERO,
            color: c_top_right,
        },
        Vertex {
            pos: pos2(l, cy),
            uv: Pos2::ZERO,
            color: c_mid,
        },
        Vertex {
            pos: pos2(cx, cy),
            uv: Pos2::ZERO,
            color: c_center,
        },
        Vertex {
            pos: pos2(r, cy),
            uv: Pos2::ZERO,
            color: c_mid,
        },
        Vertex {
            pos: pos2(l, b),
            uv: Pos2::ZERO,
            color: c_bot,
        },
        Vertex {
            pos: pos2(cx, b),
            uv: Pos2::ZERO,
            color: c_bot,
        },
        Vertex {
            pos: pos2(r, b),
            uv: Pos2::ZERO,
            color: c_bot,
        },
    ]);

    for (a, b_idx, c, d) in [(0, 1, 4, 3), (1, 2, 5, 4), (3, 4, 7, 6), (4, 5, 8, 7)] {
        mesh.add_triangle(a, b_idx, c);
        mesh.add_triangle(a, c, d);
    }

    painter.add(mesh);
}

/// Riflesso speculare superiore ben rientrato rispetto agli angoli arrotondati (zero linee sporgenti!).
pub fn paint_specular_rim_light(painter: &Painter, rect: Rect, rounding: Rounding) {
    let inset = rounding.nw.max(12.0) + 4.0;
    if rect.width() > inset * 2.2 {
        let y = rect.top() + 1.0;
        painter.line_segment(
            [pos2(rect.left() + inset, y), pos2(rect.right() - inset, y)],
            Stroke::new(1.0_f32, rgba(255, 255, 255, 42)),
        );
    }
}

/// Card principale in vetro satinato.
/// IMPORTANTE: avvolge sempre il contenuto in `ui.vertical` per impedire layout orizzontali accidentali.
pub fn liquid_glass_card<R>(
    ui: &mut Ui,
    header: Option<&str>,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    let response = egui::Frame {
        inner_margin: Margin::same(18.0),
        outer_margin: Margin::symmetric(0.0, 4.0),
        rounding: RADIUS_CARD,
        fill: COLOR_SURFACE_CARD,
        stroke: Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE),
        shadow: egui::epaint::Shadow {
            blur: 20.0,
            offset: vec2(0.0, 6.0),
            color: Color32::from_black_alpha(100),
            ..Default::default()
        },
    }
    .show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.vertical(|ui| {
            if let Some(title) = header {
                ui.label(
                    egui::RichText::new(title.to_uppercase())
                        .font(FontId::proportional(11.0))
                        .strong()
                        .color(COLOR_APPLE_CYAN),
                );
                ui.add_space(4.0);
                ui.separator();
                ui.add_space(8.0);
            }
            add_contents(ui)
        })
        .inner
    });

    paint_specular_rim_light(ui.painter(), response.response.rect, RADIUS_CARD);
    response.inner
}

pub fn liquid_segmented_control<T: Copy + PartialEq>(
    ui: &mut Ui,
    _id_salt: &str,
    selected: &mut T,
    options: &[(T, &str)],
) -> bool {
    crate::theme::segmented_control(ui, selected, options)
}

pub fn liquid_pill_button(ui: &mut Ui, label: &str, is_primary: bool) -> Response {
    let fill = if is_primary {
        COLOR_APPLE_BLUE
    } else {
        COLOR_CONTROL_INACTIVE
    };
    let fg = if is_primary {
        Color32::WHITE
    } else {
        COLOR_TEXT_PRIMARY
    };
    let btn = egui::Button::new(
        egui::RichText::new(label)
            .font(FontId::proportional(12.0))
            .strong()
            .color(fg),
    )
    .fill(fill)
    .rounding(RADIUS_CONTROL)
    .min_size(vec2(0.0, 28.0));
    ui.add(btn)
}

pub use crate::theme::{inset_grouped_card, segmented_control};
