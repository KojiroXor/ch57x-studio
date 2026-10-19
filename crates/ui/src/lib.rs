//! UI crate: Layout Master-Detail simmetrico in stile Apple Liquid Glass

pub mod doctor_panel;
pub mod hud_window;
pub mod inspector;
pub mod liquid_glass;
pub mod pad_canvas;
pub mod state;
pub mod theme;

pub use doctor_panel::DoctorPanel;
pub use hud_window::HudWindow;
pub use inspector::InspectorPanel;
pub use liquid_glass::*;
pub use pad_canvas::PadCanvas;
pub use state::UiState;
pub use theme::*;

use ch57x_platform::PlatformBackend;
use egui::{vec2, Align, Color32, FontId, Layout, Margin, Rounding, Stroke, Ui};

pub struct StudioUi {
    pub canvas: PadCanvas,
    pub inspector: InspectorPanel,
    pub doctor: DoctorPanel,
    pub hud: HudWindow,
}

impl Default for StudioUi {
    fn default() -> Self {
        Self::new()
    }
}

impl StudioUi {
    pub fn new() -> Self {
        Self {
            canvas: PadCanvas::new(),
            inspector: InspectorPanel::new(),
            doctor: DoctorPanel::new(),
            hud: HudWindow::new(),
        }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        state: &mut UiState,
        platform: Option<&dyn PlatformBackend>,
    ) {
        // 1. Dipingi lo sfondo atmosferico continuo Deep Space (senza cerchi bersaglio!)
        let bg_rect = ui.max_rect().expand(24.0);
        paint_deep_space_canvas(ui.painter(), bg_rect);

        match state.active_view {
            state::ActiveView::Editor => {
                self.show_editor_workspace(ui, state, platform);
            }
            state::ActiveView::Doctor => {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.doctor.show(ui, state, platform);
                    });
            }
            state::ActiveView::Hud => {
                // Centra la preview dell'HUD dentro una vera Glass Card elegante
                ui.vertical_centered(|ui| {
                    ui.add_space(28.0);
                    egui::Frame {
                        inner_margin: Margin::same(28.0),
                        rounding: RADIUS_CARD,
                        fill: COLOR_SURFACE_CARD,
                        stroke: Stroke::new(1.0_f32, COLOR_BORDER_STRONG),
                        shadow: egui::epaint::Shadow {
                            blur: 32.0,
                            offset: vec2(0.0, 12.0),
                            color: Color32::from_black_alpha(140),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                    .show(ui, |ui| {
                        ui.set_max_width(780.0);
                        self.hud.show_content(ui, state);
                    });
                });
            }
        }
    }

    fn show_editor_workspace(
        &mut self,
        ui: &mut Ui,
        state: &mut UiState,
        platform: Option<&dyn PlatformBackend>,
    ) {
        let total_rect = ui.available_rect_before_wrap();
        let right_width = 430.0_f32.min(total_rect.width() * 0.42).max(350.0);
        let gap = 16.0;
        let left_width = (total_rect.width() - right_width - gap).max(520.0);
        let full_height = total_rect.height();

        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing = vec2(gap, 0.0);

            // -----------------------------------------------------------------
            // COLONNA SINISTRA (Stage Hardware Macropad a tutta altezza)
            // -----------------------------------------------------------------
            let left_frame = egui::Frame {
                inner_margin: Margin::same(22.0),
                rounding: RADIUS_CARD,
                fill: COLOR_SURFACE_CARD,
                stroke: Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE),
                shadow: egui::epaint::Shadow {
                    blur: 28.0,
                    offset: vec2(0.0, 10.0),
                    color: Color32::from_black_alpha(120),
                    ..Default::default()
                },
                ..Default::default()
            };

            let left_resp = left_frame.show(ui, |ui| {
                ui.set_width((left_width - 44.0).max(400.0));
                ui.set_min_height((full_height - 46.0).max(440.0));

                // Header della Card Sinistra
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("CH57x Dual-Encoder Macropad")
                                .font(FontId::proportional(17.0))
                                .strong()
                                .color(COLOR_TEXT_PRIMARY),
                        );
                        ui.label(
                            egui::RichText::new("Matrice 4x3 Switch Meccanici  -  2 Encoder Rotativi CNC  -  3 Layer (54 Slot)")
                                .font(FontId::proportional(12.0))
                                .color(COLOR_TEXT_SECONDARY),
                        );
                    });

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let layer_badge = format!("Layer Attivo: L{}", state.editing_layer.0 + 1);
                        draw_pill_badge(ui, &layer_badge, Color32::TRANSPARENT, COLOR_APPLE_BLUE);
                    });
                });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(12.0);

                // Barra Legenda Colori in una capsula di vetro incassata
                egui::Frame {
                    inner_margin: Margin::symmetric(14.0, 8.0),
                    rounding: Rounding::same(10.0),
                    fill: COLOR_SURFACE_INSET,
                    stroke: Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE),
                    ..Default::default()
                }
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = vec2(16.0, 6.0);
                        ui.label(
                            egui::RichText::new("LEGENDA:")
                                .font(FontId::proportional(10.5))
                                .strong()
                                .color(COLOR_TEXT_TERTIARY),
                        );
                        for (code, desc) in [
                            ("HID", "HID Nativo"),
                            ("WIN", "WIN Finestra"),
                            ("WSP", "WSP Workspace"),
                            ("MAC", "MAC Macro"),
                            ("CMD", "CMD Processo"),
                            ("CTX", "CTX Contesto"),
                        ] {
                            let col = pad_canvas::badge_color(code);
                            let (dot_rect, _) =
                                ui.allocate_exact_size(vec2(8.0, 14.0), egui::Sense::hover());
                            ui.painter().circle_filled(dot_rect.center(), 3.5, col);
                            ui.label(
                                egui::RichText::new(desc)
                                    .font(FontId::proportional(11.5))
                                    .color(COLOR_TEXT_SECONDARY),
                            );
                        }
                    });
                });

                // Centra verticalmente e orizzontalmente il Macropad nello spazio rimanente
                let rem_h = ui.available_height();
                let pad_top_space = ((rem_h - self.canvas.desired_size.y) * 0.5).max(12.0);
                ui.add_space(pad_top_space);

                ui.horizontal(|ui| {
                    let rem_w = ui.available_width();
                    let pad_left_space = ((rem_w - self.canvas.desired_size.x) * 0.5).max(0.0);
                    ui.add_space(pad_left_space);
                    self.canvas.show(ui, state);
                });
            });
            paint_specular_rim_light(ui.painter(), left_resp.response.rect, RADIUS_CARD);

            // -----------------------------------------------------------------
            // COLONNA DESTRA (Inspector Laterale a tutta altezza)
            // -----------------------------------------------------------------
            let right_frame = egui::Frame {
                inner_margin: Margin::same(20.0),
                rounding: RADIUS_CARD,
                fill: COLOR_SURFACE_CARD,
                stroke: Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE),
                shadow: egui::epaint::Shadow {
                    blur: 28.0,
                    offset: vec2(0.0, 10.0),
                    color: Color32::from_black_alpha(120),
                    ..Default::default()
                },
                ..Default::default()
            };

            let right_resp = right_frame.show(ui, |ui| {
                ui.set_width((right_width - 40.0).max(300.0));
                ui.set_min_height((full_height - 46.0).max(440.0));

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.inspector.show(ui, state, platform);
                    });
            });
            paint_specular_rim_light(ui.painter(), right_resp.response.rect, RADIUS_CARD);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_studio_ui_instantiation() {
        let ui = StudioUi::new();
        let _ = ui;
    }
}
