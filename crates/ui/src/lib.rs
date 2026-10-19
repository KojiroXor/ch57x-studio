//! UI crate: Canvas 4x3+2, Inspector, Doctor panel, HUD window, Cupertino Pro Dark Theme & Liquid Glass

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
use egui::{vec2, Color32, FontId, Ui};

/// Vista principale composita che orchestra PadCanvas, Inspector, Doctor e HUD
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

    /// Disegna il contenuto centrale con distribuzione Sezione Aurea (60% Macropad / 40% Inspector)
    pub fn show(
        &mut self,
        ui: &mut Ui,
        state: &mut UiState,
        platform: Option<&dyn PlatformBackend>,
    ) {
        match state.active_view {
            state::ActiveView::Editor => {
                let avail_width = ui.available_width();
                let right_width = (avail_width * 0.40).max(380.0);
                let left_width = (avail_width - right_width - 16.0).max(480.0);

                ui.horizontal_top(|ui| {
                    // Colonna Sinistra (Hardware Macropad Canvas — ~60% larghezza)
                    ui.allocate_ui_with_layout(
                        vec2(left_width, ui.available_height()),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            liquid_glass_card(ui, None, |ui| {
                                ui.vertical(|ui| {
                                    // Titolo chiaro e Sottotitolo
                                    ui.horizontal(|ui| {
                                        ui.vertical(|ui| {
                                            ui.label(
                                                egui::RichText::new("CH57x Dual-Encoder Macropad")
                                                    .font(FontId::proportional(15.0))
                                                    .strong()
                                                    .color(COLOR_TEXT_PRIMARY),
                                            );
                                            ui.label(
                                                egui::RichText::new("Matrice 4x3 Tasti Meccanici + 2 Manopole Rotative")
                                                    .font(FontId::proportional(11.5))
                                                    .color(COLOR_TEXT_SECONDARY),
                                            );
                                        });

                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            let current_layer = state.editing_layer.display_number();
                                            draw_pill_badge(
                                                ui,
                                                &format!("Layer Attivo: L{}", current_layer),
                                                Color32::from_rgb(18, 44, 75),
                                                COLOR_APPLE_BLUE,
                                            );
                                        });
                                    });

                                    ui.add_space(8.0);

                                    // Badge e Legenda Colori Tipologia disposti in una barra orizzontale ordinata
                                    ui.horizontal_wrapped(|ui| {
                                        ui.label(
                                            egui::RichText::new("Legenda:")
                                                .font(FontId::proportional(10.5))
                                                .color(COLOR_TEXT_TERTIARY),
                                        );
                                        let badges = [
                                            ("HID", "Nativo"),
                                            ("WIN", "Finestra"),
                                            ("WSP", "Workspace"),
                                            ("MAC", "Macro"),
                                            ("CMD", "Processo"),
                                            ("CTX", "Contesto"),
                                        ];
                                        for (b, desc) in badges {
                                            let col = pad_canvas::badge_color(b);
                                            ui.horizontal(|ui| {
                                                let (r, _) = ui.allocate_exact_size(vec2(8.0, 8.0), egui::Sense::hover());
                                                ui.painter().circle_filled(r.center(), 3.5, col);
                                                ui.label(
                                                    egui::RichText::new(format!("{} {}", b, desc))
                                                        .font(FontId::proportional(10.5))
                                                        .color(COLOR_TEXT_SECONDARY),
                                                );
                                            });
                                            ui.add_space(4.0);
                                        }
                                    });

                                    ui.add_space(10.0);
                                    ui.separator();
                                    ui.add_space(8.0);

                                    // Canvas centrato con piastra acrilica e tasti 3D
                                    ui.vertical_centered(|ui| {
                                        self.canvas.show(ui, state);
                                    });
                                });
                            });
                        },
                    );

                    ui.add_space(8.0);

                    // Colonna Destra (Inspector Proprietà — ~40% larghezza, min 380px)
                    ui.allocate_ui_with_layout(
                        vec2(ui.available_width(), ui.available_height()),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                self.inspector.show(ui, state, platform);
                            });
                        },
                    );
                });
            }
            state::ActiveView::Doctor => {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.doctor.show(ui, state, platform);
                });
            }
            state::ActiveView::Hud => {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.hud.show_content(ui, state);
                });
            }
        }
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
