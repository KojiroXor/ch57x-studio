//! Apple macOS Sequoia / Tahoe Pro Dark Design System
//! Zero saturazione GPU, zero glifi Unicode mancanti, layout 100% deterministico.

use egui::{
    pos2, vec2, Align2, Color32, Context, FontId, Margin, Pos2, Rect, Rounding, Sense, Stroke, Ui,
    Visuals,
};

/// Calcola il colore premoltiplicato corretto a partire da RGBA standard (0..255).
pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color32 {
    let r_p = ((r as u32 * a as u32 + 127) / 255) as u8;
    let g_p = ((g as u32 * a as u32 + 127) / 255) as u8;
    let b_p = ((b as u32 * a as u32 + 127) / 255) as u8;
    Color32::from_rgba_premultiplied(r_p, g_p, b_p, a)
}

// Palette Apple Pro Dark & Liquid Glass
pub const COLOR_BG_DEEP: Color32 = Color32::from_rgb(13, 15, 20); // #0D0F14
pub const COLOR_TOOLBAR_BG: Color32 = rgba(22, 25, 34, 242); // #161922
pub const COLOR_SURFACE_CARD: Color32 = rgba(28, 32, 43, 215); // #1C202B
pub const COLOR_SURFACE_ELEVATED: Color32 = rgba(42, 48, 64, 235); // #2A3040
pub const COLOR_SURFACE_INSET: Color32 = rgba(16, 19, 26, 230); // #10131A

pub const COLOR_BORDER_SUBTLE: Color32 = rgba(255, 255, 255, 22);
pub const COLOR_BORDER_STRONG: Color32 = rgba(255, 255, 255, 45);
pub const COLOR_RIM_HIGHLIGHT: Color32 = rgba(255, 255, 255, 65);

pub const COLOR_CONTROL_INACTIVE: Color32 = rgba(44, 50, 66, 200);
pub const COLOR_CONTROL_HOVER: Color32 = rgba(60, 68, 88, 235);

// Accenti Semantici Apple HIG
pub const COLOR_APPLE_BLUE: Color32 = Color32::from_rgb(10, 132, 255); // #0A84FF
pub const COLOR_APPLE_GREEN: Color32 = Color32::from_rgb(48, 209, 88); // #30D158
pub const COLOR_APPLE_ORANGE: Color32 = Color32::from_rgb(255, 159, 10); // #FF9F0A
pub const COLOR_APPLE_RED: Color32 = Color32::from_rgb(255, 69, 58); // #FF453A
pub const COLOR_APPLE_PURPLE: Color32 = Color32::from_rgb(191, 90, 242); // #BF5AF2
pub const COLOR_APPLE_CYAN: Color32 = Color32::from_rgb(100, 210, 255); // #64D2FF

// Tipografia
pub const COLOR_TEXT_PRIMARY: Color32 = Color32::from_rgb(245, 247, 250);
pub const COLOR_TEXT_SECONDARY: Color32 = Color32::from_rgb(156, 164, 180);
pub const COLOR_TEXT_TERTIARY: Color32 = Color32::from_rgb(104, 112, 128);

// Raggi di curvatura Apple Squircle
pub const RADIUS_CARD: Rounding = Rounding {
    nw: 14.0,
    ne: 14.0,
    sw: 14.0,
    se: 14.0,
};
pub const RADIUS_CONTROL: Rounding = Rounding {
    nw: 8.0,
    ne: 8.0,
    sw: 8.0,
    se: 8.0,
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
    visuals.extreme_bg_color = Color32::from_rgb(9, 11, 15);

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

    visuals.selection.bg_fill = rgba(10, 132, 255, 80);
    visuals.selection.stroke = Stroke::new(1.0_f32, COLOR_APPLE_BLUE);

    visuals.window_rounding = RADIUS_CARD;
    visuals.window_stroke = Stroke::new(1.0_f32, COLOR_BORDER_STRONG);
    visuals.window_shadow.blur = 24.0;
    visuals.window_shadow.color = Color32::from_black_alpha(140);

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = vec2(8.0, 8.0);
    style.spacing.button_padding = vec2(12.0, 6.0);
    style.spacing.window_margin = Margin::same(14.0);
    ctx.set_style(style);
}

/// Disegna un punto di stato vettoriale con alone luminoso (senza usare glifi Unicode).
pub fn draw_status_dot(ui: &mut Ui, center: Pos2, color: Color32, label: &str) {
    let painter = ui.painter();
    let halo = rgba(color.r(), color.g(), color.b(), 45);
    painter.circle_filled(center, 6.0, halo);
    painter.circle_filled(center, 3.2, color);

    if !label.is_empty() {
        painter.text(
            pos2(center.x + 11.0, center.y),
            Align2::LEFT_CENTER,
            label,
            FontId::proportional(11.5),
            COLOR_TEXT_PRIMARY,
        );
    }
}

/// Disegna un badge a pillola con pallino luminoso integrato.
pub fn draw_pill_badge(ui: &mut Ui, text: &str, _bg_unused: Color32, accent: Color32) -> Rect {
    let clean = text.trim_start_matches('●').trim_start_matches('□').trim();
    let galley = ui
        .painter()
        .layout_no_wrap(clean.to_string(), FontId::proportional(11.0), accent);

    let dot_space = 13.0;
    let pad = vec2(10.0, 4.0);
    let size = vec2(
        galley.size().x + pad.x * 2.0 + dot_space,
        (galley.size().y + pad.y * 2.0).max(22.0),
    );
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());

    let fill = rgba(accent.r(), accent.g(), accent.b(), 26);
    let border = rgba(accent.r(), accent.g(), accent.b(), 90);

    ui.painter().rect_filled(rect, RADIUS_PILL, fill);
    ui.painter()
        .rect_stroke(rect, RADIUS_PILL, Stroke::new(1.0_f32, border));

    let dot_c = pos2(rect.min.x + pad.x + 3.0, rect.center().y);
    ui.painter()
        .circle_filled(dot_c, 5.0, rgba(accent.r(), accent.g(), accent.b(), 45));
    ui.painter().circle_filled(dot_c, 2.8, accent);

    let text_pos = pos2(
        rect.min.x + pad.x + dot_space,
        rect.center().y - galley.size().y * 0.5,
    );
    ui.painter().galley(text_pos, galley, accent);

    rect
}

/// Segmented Control compatto stile macOS Sequoia.
pub fn segmented_control<T: Copy + PartialEq>(
    ui: &mut Ui,
    selected: &mut T,
    options: &[(T, &str)],
) -> bool {
    let mut changed = false;

    egui::Frame {
        inner_margin: Margin::same(2.5),
        rounding: Rounding::same(9.0),
        fill: COLOR_SURFACE_INSET,
        stroke: Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE),
        ..Default::default()
    }
    .show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(2.0, 0.0);
            for (val, label) in options {
                let is_active = *selected == *val;
                let fill = if is_active {
                    COLOR_APPLE_BLUE
                } else {
                    Color32::TRANSPARENT
                };
                let fg = if is_active {
                    Color32::WHITE
                } else {
                    COLOR_TEXT_SECONDARY
                };

                let btn = egui::Button::new(
                    egui::RichText::new(*label)
                        .font(FontId::proportional(11.5))
                        .strong()
                        .color(fg),
                )
                .fill(fill)
                .stroke(if is_active {
                    Stroke::new(1.0_f32, rgba(255, 255, 255, 65))
                } else {
                    Stroke::NONE
                })
                .rounding(Rounding::same(6.5))
                .min_size(vec2(0.0, 23.0));

                if ui.add(btn).clicked() {
                    *selected = *val;
                    changed = true;
                }
            }
        });
    });

    changed
}

/// Card interna incassata (stile Impostazioni di Sistema macOS).
/// Forza SEMPRE `ui.vertical` all'interno per evitare il bug dell'ereditarietà orizzontale di egui!
pub fn inset_grouped_card<R>(
    ui: &mut Ui,
    header: Option<&str>,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    ui.vertical(|ui| {
        if let Some(title) = header {
            ui.label(
                egui::RichText::new(title.to_uppercase())
                    .font(FontId::proportional(10.5))
                    .strong()
                    .color(COLOR_TEXT_TERTIARY),
            );
            ui.add_space(3.0);
        }

        egui::Frame {
            inner_margin: Margin::symmetric(14.0, 12.0),
            rounding: Rounding::same(11.0),
            fill: COLOR_SURFACE_INSET,
            stroke: Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical(add_contents).inner
        })
        .inner
    })
    .inner
}
