//! Apple macOS Tahoe / visionOS Liquid Glass Theme & Vector Primitives
//! Zero `from_rgba_premultiplied` con valori non pre-moltiplicati, zero glifi Unicode mancanti.

use egui::{
    pos2, vec2, Align2, Color32, Context, FontId, Margin, Pos2, Rect, Rounding, Sense, Stroke, Ui,
    Visuals,
};

/// Helper const per calcolare il corretto colore premoltiplicato a partire da valori non premoltiplicati
/// Evita la saturazione additiva della GPU (garantisce sempre r, g, b <= a)
pub const fn unmultiplied_rgba(r: u8, g: u8, b: u8, a: u8) -> Color32 {
    let r_pre = ((r as u32 * a as u32 + 127) / 255) as u8;
    let g_pre = ((g as u32 * a as u32 + 127) / 255) as u8;
    let b_pre = ((b as u32 * a as u32 + 127) / 255) as u8;
    Color32::from_rgba_premultiplied(r_pre, g_pre, b_pre, a)
}

// Palette Base Deep Space & Liquid Glass
pub const COLOR_BG_DEEP: Color32 = Color32::from_rgb(10, 12, 18); // #0A0C12
pub const COLOR_TOOLBAR_BG: Color32 = unmultiplied_rgba(18, 22, 32, 235);
pub const COLOR_SURFACE_CARD: Color32 = unmultiplied_rgba(26, 31, 44, 185);
pub const COLOR_SURFACE_ELEVATED: Color32 = unmultiplied_rgba(38, 45, 62, 210);
pub const COLOR_SURFACE_INSET: Color32 = unmultiplied_rgba(14, 17, 24, 215);

pub const COLOR_BORDER_SUBTLE: Color32 = unmultiplied_rgba(255, 255, 255, 22);
pub const COLOR_BORDER_STRONG: Color32 = unmultiplied_rgba(255, 255, 255, 48);
pub const COLOR_RIM_HIGHLIGHT: Color32 = unmultiplied_rgba(255, 255, 255, 70);

pub const COLOR_CONTROL_INACTIVE: Color32 = unmultiplied_rgba(46, 54, 72, 180);
pub const COLOR_CONTROL_HOVER: Color32 = unmultiplied_rgba(64, 74, 96, 220);

// Accenti Semantici Apple HIG
pub const COLOR_APPLE_BLUE: Color32 = Color32::from_rgb(10, 132, 255); // #0A84FF
pub const COLOR_APPLE_GREEN: Color32 = Color32::from_rgb(48, 209, 88); // #30D158
pub const COLOR_APPLE_ORANGE: Color32 = Color32::from_rgb(255, 159, 10); // #FF9F0A
pub const COLOR_APPLE_RED: Color32 = Color32::from_rgb(255, 69, 58); // #FF453A
pub const COLOR_APPLE_PURPLE: Color32 = Color32::from_rgb(191, 90, 242); // #BF5AF2
pub const COLOR_APPLE_CYAN: Color32 = Color32::from_rgb(100, 210, 255); // #64D2FF

// Tipografia
pub const COLOR_TEXT_PRIMARY: Color32 = Color32::from_rgb(245, 247, 250);
pub const COLOR_TEXT_SECONDARY: Color32 = Color32::from_rgb(152, 160, 176);
pub const COLOR_TEXT_TERTIARY: Color32 = Color32::from_rgb(98, 106, 122);

// Raggi di curvatura Apple Squircle
pub const RADIUS_CARD: Rounding = Rounding {
    nw: 16.0,
    ne: 16.0,
    sw: 16.0,
    se: 16.0,
};
pub const RADIUS_CONTROL: Rounding = Rounding {
    nw: 9.0,
    ne: 9.0,
    sw: 9.0,
    se: 9.0,
};
pub const RADIUS_PILL: Rounding = Rounding {
    nw: 99.0,
    ne: 99.0,
    sw: 99.0,
    se: 99.0,
};

pub fn apply_cupertino_theme(ctx: &Context) {
    let mut visuals = Visuals::dark();

    visuals.window_fill = COLOR_BG_DEEP;
    visuals.panel_fill = COLOR_BG_DEEP;
    visuals.faint_bg_color = COLOR_SURFACE_CARD;
    visuals.extreme_bg_color = Color32::from_rgb(8, 10, 15);

    visuals.widgets.noninteractive.bg_fill = COLOR_SURFACE_CARD;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, COLOR_TEXT_PRIMARY);
    visuals.widgets.noninteractive.rounding = RADIUS_CARD;

    visuals.widgets.inactive.bg_fill = COLOR_CONTROL_INACTIVE;
    visuals.widgets.inactive.weak_bg_fill = COLOR_CONTROL_INACTIVE;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE);
    visuals.widgets.inactive.rounding = RADIUS_CONTROL;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, COLOR_TEXT_PRIMARY);

    visuals.widgets.hovered.bg_fill = COLOR_CONTROL_HOVER;
    visuals.widgets.hovered.weak_bg_fill = COLOR_CONTROL_HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, COLOR_BORDER_STRONG);
    visuals.widgets.hovered.rounding = RADIUS_CONTROL;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);

    visuals.widgets.active.bg_fill = COLOR_APPLE_BLUE;
    visuals.widgets.active.weak_bg_fill = COLOR_APPLE_BLUE;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, COLOR_RIM_HIGHLIGHT);
    visuals.widgets.active.rounding = RADIUS_CONTROL;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);

    visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(10, 132, 255, 90);
    visuals.selection.stroke = Stroke::new(1.0_f32, COLOR_APPLE_BLUE);

    visuals.window_rounding = RADIUS_CARD;
    visuals.window_stroke = Stroke::new(1.0_f32, COLOR_BORDER_STRONG);
    visuals.window_shadow.blur = 28.0;
    visuals.window_shadow.color = Color32::from_black_alpha(150);

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = vec2(10.0, 10.0);
    style.spacing.button_padding = vec2(14.0, 7.0);
    style.spacing.window_margin = Margin::same(16.0);
    ctx.set_style(style);
}

/// Disegna un punto luminoso vettoriale (senza usare caratteri Unicode `●` che producono `□`)
pub fn draw_status_dot(ui: &mut Ui, center: Pos2, color: Color32, label: &str) {
    let painter = ui.painter();
    let halo = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 45);
    painter.circle_filled(center, 6.5, halo);
    painter.circle_filled(center, 3.5, color);

    if !label.is_empty() {
        painter.text(
            pos2(center.x + 11.0, center.y),
            Align2::LEFT_CENTER,
            label,
            FontId::proportional(12.0),
            COLOR_TEXT_PRIMARY,
        );
    }
}

/// Disegna un badge a pillola Liquid Glass con pallino di stato vettoriale integrato opzionale
pub fn draw_pill_badge(ui: &mut Ui, text: &str, _bg_unused: Color32, accent: Color32) -> Rect {
    let clean_text = text.trim_start_matches('●').trim_start_matches('□').trim();
    let has_dot = text.contains('●')
        || text.contains("USB")
        || text.contains("Modifiche")
        || text.contains("Solo");

    let galley =
        ui.painter()
            .layout_no_wrap(clean_text.to_string(), FontId::proportional(11.5), accent);

    let dot_space = if has_dot { 14.0 } else { 0.0 };
    let padding = vec2(10.0, 5.0);
    let size = vec2(
        galley.size().x + padding.x * 2.0 + dot_space,
        (galley.size().y + padding.y * 2.0).max(24.0),
    );
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());

    let fill = Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 28);
    let stroke_col = Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 95);

    ui.painter().rect_filled(rect, RADIUS_PILL, fill);
    ui.painter()
        .rect_stroke(rect, RADIUS_PILL, Stroke::new(1.0_f32, stroke_col));

    let mut text_x = rect.min.x + padding.x;
    if has_dot {
        let dot_center = pos2(rect.min.x + padding.x + 3.5, rect.center().y);
        ui.painter().circle_filled(
            dot_center,
            5.5,
            Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 50),
        );
        ui.painter().circle_filled(dot_center, 3.0, accent);
        text_x += dot_space;
    }

    let text_pos = pos2(text_x, rect.center().y - galley.size().y / 2.0);
    ui.painter().galley(text_pos, galley, accent);

    rect
}

/// Segmented Control stile macOS Tahoe / Liquid Glass Pill
pub fn segmented_control<T: Copy + PartialEq>(
    ui: &mut Ui,
    selected: &mut T,
    options: &[(T, &str)],
) -> bool {
    let mut changed = false;
    let height = 30.0;

    let frame = egui::Frame {
        inner_margin: Margin::same(3.0),
        rounding: Rounding::same(10.0),
        fill: COLOR_SURFACE_INSET,
        stroke: Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE),
        ..Default::default()
    };

    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(3.0, 0.0);
            for (val, label) in options {
                let is_active = *selected == *val;
                let fill = if is_active {
                    COLOR_APPLE_BLUE
                } else {
                    Color32::TRANSPARENT
                };
                let text_col = if is_active {
                    Color32::WHITE
                } else {
                    COLOR_TEXT_SECONDARY
                };

                let btn = egui::Button::new(
                    egui::RichText::new(*label)
                        .font(FontId::proportional(12.0))
                        .color(text_col)
                        .strong(),
                )
                .fill(fill)
                .stroke(if is_active {
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 70))
                } else {
                    Stroke::NONE
                })
                .rounding(Rounding::same(7.0))
                .min_size(vec2(0.0, height - 6.0));

                if ui.add(btn).clicked() {
                    *selected = *val;
                    changed = true;
                }
            }
        });
    });

    changed
}

pub fn inset_grouped_card<R>(
    ui: &mut Ui,
    title: Option<&str>,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    crate::liquid_glass::liquid_glass_card(ui, title, add_contents)
}
