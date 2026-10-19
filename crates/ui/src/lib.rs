//! UI crate: Canvas 4x3+2, Inspector, Doctor panel, HUD window, and Cupertino Pro Dark Theme

pub mod doctor_panel;
pub mod hud_window;
pub mod inspector;
pub mod pad_canvas;
pub mod state;
pub mod theme;

pub use doctor_panel::DoctorPanel;
pub use hud_window::HudWindow;
pub use inspector::InspectorPanel;
pub use pad_canvas::PadCanvas;
pub use state::UiState;
pub use theme::*;

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

    /// Disegna il contenuto centrale della vista attiva (senza duplicare la toolbar superiore)
    pub fn show(
        &mut self,
        ui: &mut Ui,
        state: &mut UiState,
        platform: Option<&dyn PlatformBackend>,
    ) {
        match state.active_view {
            state::ActiveView::Editor => {
                ui.horizontal(|ui| {
                    // Left: Pad Canvas hardware
                    ui.vertical(|ui| {
                        self.canvas.show(ui, state);
                    });

                    ui.separator();

                    // Right: Inset Grouped Cards Inspector
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
