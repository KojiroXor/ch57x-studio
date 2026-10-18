//! UI crate: Canvas 4x3+2, Inspector, Doctor panel, HUD window

pub mod doctor_panel;
pub mod hud_window;
pub mod inspector;
pub mod pad_canvas;
pub mod state;

pub use doctor_panel::DoctorPanel;
pub use hud_window::HudWindow;
pub use inspector::InspectorPanel;
pub use pad_canvas::PadCanvas;
pub use state::UiState;

use ch57x_platform::PlatformBackend;
use egui::Ui;

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

    /// Disegna l'interfaccia completa integrata
    pub fn show(
        &mut self,
        ui: &mut Ui,
        state: &mut UiState,
        platform: Option<&dyn PlatformBackend>,
    ) {
        // 1. Barra di navigazione superiore (Layer Tabs & View Selector)
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("VortexDeck CH57x Studio")
                    .font(egui::FontId::proportional(15.0))
                    .strong(),
            );

            ui.separator();

            // Layer Tabs (1, 2, 3)
            for l in 0..3u8 {
                let is_current = state.editing_layer.0 == l;
                let label = format!("Layer {}", l + 1);
                if ui.selectable_label(is_current, label).clicked() {
                    state.set_editing_layer(ch57x_domain::hardware::LayerIndex(l));
                }
            }

            ui.separator();

            // View Tabs (Editor, Doctor, HUD)
            if ui.selectable_label(state.active_view == state::ActiveView::Editor, "🎛️ Editor").clicked() {
                state.active_view = state::ActiveView::Editor;
            }
            if ui.selectable_label(state.active_view == state::ActiveView::Doctor, "🩺 Doctor").clicked() {
                state.active_view = state::ActiveView::Doctor;
            }
            if ui.selectable_label(state.active_view == state::ActiveView::Hud, "🖥️ HUD OSD").clicked() {
                state.active_view = state::ActiveView::Hud;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if state.is_dirty {
                    ui.label(
                        egui::RichText::new("● Modifiche non salvate")
                            .color(egui::Color32::from_rgb(234, 179, 8)),
                    );
                }
            });
        });

        ui.add_space(6.0);
        ui.separator();
        ui.add_space(6.0);

        // 2. Contenuto della vista attiva
        match state.active_view {
            state::ActiveView::Editor => {
                ui.horizontal(|ui| {
                    // Left: Pad Canvas
                    ui.vertical(|ui| {
                        self.canvas.show(ui, state);
                    });

                    ui.separator();

                    // Right: Inspector Sidebar
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        self.inspector.show(ui, state, platform);
                    });
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

        ui.add_space(6.0);
        ui.separator();

        // 3. Status Bar inferiore
        ui.horizontal(|ui| {
            if let Some(alert) = &state.status_message {
                let color = match alert.level {
                    state::StatusLevel::Info => egui::Color32::from_rgb(56, 189, 248),
                    state::StatusLevel::Success => egui::Color32::from_rgb(34, 197, 94),
                    state::StatusLevel::Warning => egui::Color32::from_rgb(234, 179, 8),
                    state::StatusLevel::Error => egui::Color32::from_rgb(239, 68, 68),
                };
                ui.label(egui::RichText::new(&alert.text).color(color));
            } else {
                ui.label(
                    egui::RichText::new("Pronto.")
                        .color(egui::Color32::from_rgb(113, 113, 122)),
                );
            }
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
