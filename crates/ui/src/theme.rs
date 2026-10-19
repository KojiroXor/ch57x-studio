//! Apple macOS Sequoia / Cupertino Pro Dark Design System Theme
//!
//! Fornisce palette colori, stili per i widget egui e helper di disegno
//! per indicatori geometrici luminosi e pill badges minimali.

use egui::{vec2, Color32, Context, FontId, Margin, Pos2, Rect, Rounding, Stroke, Ui, Visuals};

// -----------------------------------------------------------------------------
// PALETTE COLORI APPLE CUPERTINO PRO DARK
// -----------------------------------------------------------------------------

pub const COLOR_OBSIDIAN_BG: Color32 = Color32::from_rgb(28, 28, 30); // #1C1C1E
pub const COLOR_TOOLBAR_BG: Color32 = Color32::from_rgb(36, 36, 38); // #242426
pub const COLOR_SURFACE_CARD: Color32 = Color32::from_rgb(44, 44, 46); // #2C2C2E
pub const COLOR_CONTROL_INACTIVE: Color32 = Color32::from_rgb(58, 58, 60); // #3A3A3C
pub const COLOR_CONTROL_HOVER: Color32 = Color32::from_rgb(72, 72, 74); // #48484A
pub const COLOR_BORDER_SUBTLE: Color32 = Color32::from_rgb(56, 56, 58); // #38383A
pub const COLOR_BORDER_STRONG: Color32 = Color32::from_rgb(84, 84, 88); // #545458

// Colori Semantici Apple
pub const COLOR_APPLE_BLUE: Color32 = Color32::from_rgb(10, 132, 255); // #0A84FF
pub const COLOR_APPLE_GREEN: Color32 = Color32::from_rgb(48, 209, 88); // #30D158
pub const COLOR_APPLE_ORANGE: Color32 = Color32::from_rgb(255, 159, 10); // #FF9F0A
pub const COLOR_APPLE_RED: Color32 = Color32::from_rgb(255, 69, 58); // #FF453A
pub const COLOR_APPLE_PURPLE: Color32 = Color32::from_rgb(191, 90, 242); // #BF5AF2
pub const COLOR_APPLE_TEAL: Color32 = Color32::from_rgb(100, 210, 255); // #64D2FF

// Tipografia
pub const COLOR_TEXT_PRIMARY: Color32 = Color32::from_rgb(245, 245, 247); // #F5F5F7
pub const COLOR_TEXT_SECONDARY: Color32 = Color32::from_rgb(142, 142, 147); // #8E8E93
pub const COLOR_TEXT_TERTIARY: Color32 = Color32::from_rgb(99, 99, 102); // #636366

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
    nw: 12.0,
    ne: 12.0,
    sw: 12.0,
    se: 12.0,
};

/// Applica il tema Cupertino Pro Dark completo all'istanza di `egui::Context`
pub fn apply_cupertino_theme(ctx: &Context) {
    let mut visuals = Visuals::dark();

    // Sfondi finestre e pannelli principali
    visuals.window_fill = COLOR_OBSIDIAN_BG;
    visuals.panel_fill = COLOR_OBSIDIAN_BG;
    visuals.faint_bg_color = COLOR_SURFACE_CARD;
    visuals.extreme_bg_color = Color32::from_rgb(20, 20, 22);

    // Stile bottoni e controlli interattivi inattivi
    visuals.widgets.inactive.bg_fill = COLOR_CONTROL_INACTIVE;
    visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE);
    visuals.widgets.inactive.rounding = RADIUS_CONTROL;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, COLOR_TEXT_PRIMARY);

    // Stile controlli su Hover
    visuals.widgets.hovered.bg_fill = COLOR_CONTROL_HOVER;
    visuals.widgets.hovered.weak_bg_fill = COLOR_CONTROL_INACTIVE;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, COLOR_BORDER_STRONG);
    visuals.widgets.hovered.rounding = RADIUS_CONTROL;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, COLOR_TEXT_PRIMARY);

    // Stile controlli attivi / premuti
    visuals.widgets.active.bg_fill = COLOR_APPLE_BLUE;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, COLOR_APPLE_BLUE);
    visuals.widgets.active.rounding = RADIUS_CONTROL;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);

    // Stile campi aperti / selezionati
    visuals.widgets.open.bg_fill = COLOR_SURFACE_CARD;
    visuals.widgets.open.rounding = RADIUS_CONTROL;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0_f32, COLOR_APPLE_BLUE);

    // Finestre popup e modali
    visuals.window_rounding = RADIUS_CARD;
    visuals.window_stroke = Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE);
    visuals.window_shadow.blur = 24.0;
    visuals.window_shadow.color = Color32::from_black_alpha(160);

    ctx.set_visuals(visuals);

    // Spaziature ergonomiche macOS
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = vec2(10.0, 10.0);
    style.spacing.button_padding = vec2(14.0, 7.0);
    style.spacing.window_margin = Margin::same(16.0);
    ctx.set_style(style);
}

// -----------------------------------------------------------------------------
// HELPER DI DISEGNO GRAFICO MINIMALISTA (ZERO EMOJI)
// -----------------------------------------------------------------------------

/// Disegna un punto di stato geometrico luminoso con alone soffuso stile Apple
pub fn draw_status_dot(ui: &mut Ui, center: Pos2, color: Color32, label: &str) {
    let painter = ui.painter();

    // Alone esterno sfumato (glow)
    let halo_color = Color32::from_rgba_premultiplied(
        (color.r() as f32 * 0.25) as u8,
        (color.g() as f32 * 0.25) as u8,
        (color.b() as f32 * 0.25) as u8,
        60,
    );
    painter.circle_filled(center, 6.0, halo_color);

    // Nucleo solido centrale
    painter.circle_filled(center, 3.5, color);

    // Etichetta a destra se presente
    if !label.is_empty() {
        let text_pos = Pos2::new(center.x + 10.0, center.y);
        painter.text(
            text_pos,
            egui::Align2::LEFT_CENTER,
            label,
            FontId::proportional(12.0),
            COLOR_TEXT_PRIMARY,
        );
    }
}

/// Disegna un badge a pillola arrotondata con colore semantico
pub fn draw_pill_badge(ui: &mut Ui, text: &str, fill_color: Color32, text_color: Color32) -> Rect {
    let padding = vec2(8.0, 3.0);
    let galley =
        ui.painter()
            .layout_no_wrap(text.to_string(), FontId::proportional(11.0), text_color);
    let size = vec2(
        galley.size().x + padding.x * 2.0,
        galley.size().y + padding.y * 2.0,
    );
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());

    ui.painter().rect_filled(rect, RADIUS_PILL, fill_color);
    ui.painter().rect_stroke(
        rect,
        RADIUS_PILL,
        Stroke::new(1.0_f32, fill_color.gamma_multiply(1.2)),
    );

    let text_pos = Pos2::new(rect.center().x, rect.center().y);
    ui.painter().galley(
        Pos2::new(
            text_pos.x - galley.size().x / 2.0,
            text_pos.y - galley.size().y / 2.0,
        ),
        galley,
        text_color,
    );

    rect
}

/// Disegna una scheda raggruppata incassata (Inset Grouped Card) in puro stile Impostazioni macOS
pub fn inset_grouped_card<R>(
    ui: &mut Ui,
    title: Option<&str>,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    if let Some(t) = title {
        ui.label(
            egui::RichText::new(t.to_uppercase())
                .font(FontId::proportional(10.5))
                .color(COLOR_TEXT_SECONDARY)
                .strong(),
        );
        ui.add_space(3.0);
    }

    let frame = egui::Frame {
        inner_margin: Margin::same(14.0),
        rounding: RADIUS_CARD,
        fill: COLOR_SURFACE_CARD,
        stroke: Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE),
        ..Default::default()
    };

    frame.show(ui, add_contents).inner
}

/// Segmented Control orizzontale in stile macOS
pub fn segmented_control<T: PartialEq + Copy>(
    ui: &mut Ui,
    selected: &mut T,
    options: &[(T, &str)],
) -> bool {
    let mut changed = false;
    let height = 30.0;
    let container_frame = egui::Frame {
        inner_margin: Margin::same(2.0),
        rounding: Rounding::same(8.0),
        fill: Color32::from_rgb(24, 24, 26),
        stroke: Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE),
        ..Default::default()
    };

    container_frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(2.0, 0.0);
            for (val, label) in options {
                let is_active = *selected == *val;
                let bg = if is_active {
                    COLOR_CONTROL_INACTIVE
                } else {
                    Color32::TRANSPARENT
                };
                let text_color = if is_active {
                    COLOR_TEXT_PRIMARY
                } else {
                    COLOR_TEXT_SECONDARY
                };

                let btn = egui::Button::new(
                    egui::RichText::new(*label)
                        .font(FontId::proportional(12.0))
                        .color(text_color)
                        .strong(),
                )
                .fill(bg)
                .stroke(if is_active {
                    Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE)
                } else {
                    Stroke::NONE
                })
                .rounding(Rounding::same(6.0))
                .min_size(vec2(0.0, height - 4.0));

                if ui.add(btn).clicked() {
                    *selected = *val;
                    changed = true;
                }
            }
        });
    });

    changed
}
