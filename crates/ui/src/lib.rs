//! VortexDeck Studio UI Root Module
//! Usa `SidePanel::right` nativo per l'Inspector + `CentralPanel` verticale per il Macropad:
//! garantisce zero clipping, zero testo verticale e proporzioni impeccabili su qualsiasi risoluzione.

pub mod doctor_panel;
pub mod hud_window;
pub mod inspector;
pub mod liquid_glass;
pub mod pad_canvas;
pub mod presets;
pub mod state;
pub mod theme;

pub use doctor_panel::DoctorPanel;
pub use hud_window::HudWindow;
pub use inspector::InspectorPanel;
pub use liquid_glass::{liquid_glass_card, paint_deep_space_canvas, paint_specular_rim_light};
pub use pad_canvas::PadCanvas;
pub use presets::{apply_encoder_preset, ENCODER_PRESETS, KEY_PRESETS, LAYER_TEMPLATES};
pub use state::UiState;
pub use theme::{apply_cupertino_theme, draw_pill_badge, draw_status_dot};

use ch57x_domain::hardware::LayerIndex;
use ch57x_platform::PlatformBackend;
use egui::{vec2, Align, Color32, FontId, Layout, Margin, Rounding, ScrollArea, Stroke, Ui};
use state::ActiveView;
use theme::*;

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

    /// Punto di ingresso principale compatibile con `CentralPanel`
    pub fn show(
        &mut self,
        ui: &mut Ui,
        state: &mut UiState,
        platform: Option<&dyn PlatformBackend>,
    ) {
        let bg_rect = ui.max_rect().expand(20.0);
        paint_deep_space_canvas(ui.painter(), bg_rect);

        match state.active_view {
            ActiveView::Editor => {
                self.show_editor_split(ui, state, platform);
            }
            ActiveView::Doctor => {
                ScrollArea::vertical()
                    .id_salt("doctor_main_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_max_width(980.0);
                        self.doctor.show(ui, state, platform);
                    });
            }
            ActiveView::Hud => {
                ScrollArea::vertical()
                    .id_salt("hud_main_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(20.0);
                            ui.allocate_ui(
                                vec2(760.0_f32.min(ui.available_width()), 360.0),
                                |ui| {
                                    liquid_glass_card(
                                        ui,
                                        Some("Anteprima On-Screen Display (OSD)"),
                                        |ui| {
                                            self.hud.show_content(ui, state);
                                        },
                                    );
                                },
                            );
                        });
                    });
            }
        }
    }

    fn show_editor_split(
        &mut self,
        ui: &mut Ui,
        state: &mut UiState,
        platform: Option<&dyn PlatformBackend>,
    ) {
        let ctx = ui.ctx().clone();

        // 1. PANNELLO DESTRO FISSO (Inspector - 410px, sempre visibile e mai tagliato fuori schermo)
        egui::SidePanel::right("apple_inspector_sidepanel")
            .resizable(true)
            .default_width(410.0)
            .width_range(360.0..=500.0)
            .frame(egui::Frame {
                inner_margin: Margin::symmetric(14.0, 6.0),
                fill: Color32::TRANSPARENT,
                stroke: Stroke::NONE,
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                liquid_glass_card(ui, None, |ui| {
                    ScrollArea::vertical()
                        .id_salt("inspector_vertical_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            self.inspector.show(ui, state, platform);
                        });
                });
            });

        // 2. AREA CENTRALE SINISTRA (Stage Hardware Macropad - Verticale garantito)
        egui::CentralPanel::default()
            .frame(egui::Frame {
                inner_margin: Margin::same(6.0),
                fill: Color32::TRANSPARENT,
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                let _ = ctx;
                liquid_glass_card(ui, None, |ui| {
                    // Riga 1: Intestazione Hardware + Selettore Layer [ Layer 1 | Layer 2 | Layer 3 ]
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("CH57x Dual-Encoder Macropad")
                                    .font(FontId::proportional(16.0))
                                    .strong()
                                    .color(COLOR_TEXT_PRIMARY),
                            );
                            ui.label(
                                egui::RichText::new("12 Tasti Meccanici (4x3)  •  3 LED Stato  •  2 Encoder CNC (54 Slot)")
                                    .font(FontId::proportional(11.5))
                                    .color(COLOR_TEXT_SECONDARY),
                            );
                        });

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let mut cur_layer = state.editing_layer.0;
                            let layer_opts = [
                                (0u8, "Layer 1"),
                                (1u8, "Layer 2"),
                                (2u8, "Layer 3"),
                            ];
                            if segmented_control(ui, &mut cur_layer, &layer_opts) {
                                state.set_editing_layer(LayerIndex(cur_layer));
                            }
                        });
                    });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // Riga 2: Legenda Orizzontale in Capsula (Larghezza esplicita = MAI più testo verticale!)
                    egui::Frame {
                        inner_margin: Margin::symmetric(12.0, 7.0),
                        rounding: Rounding::same(9.0),
                        fill: COLOR_SURFACE_INSET,
                        stroke: Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE),
                        ..Default::default()
                    }
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = vec2(14.0, 4.0);
                            ui.label(
                                egui::RichText::new("LEGENDA:")
                                    .font(FontId::proportional(10.0))
                                    .strong()
                                    .color(COLOR_TEXT_TERTIARY),
                            );
                            for (code, desc) in [
                                ("HID", "HID Nativo"),
                                ("WIN", "Finestra"),
                                ("WSP", "Workspace"),
                                ("MAC", "Macro"),
                                ("CMD", "Processo"),
                                ("CTX", "Contesto"),
                            ] {
                                let col = pad_canvas::badge_color(code);
                                let (dot_r, _) = ui.allocate_exact_size(vec2(8.0, 12.0), egui::Sense::hover());
                                ui.painter().circle_filled(dot_r.center(), 3.5, col);
                                ui.label(
                                    egui::RichText::new(desc)
                                        .font(FontId::proportional(11.0))
                                        .color(COLOR_TEXT_SECONDARY),
                                );
                            }
                        });
                    });

                    ui.add_space(16.0);

                    // Riga 3: Il Macropad centrato orizzontalmente
                    ui.vertical_centered(|ui| {
                        self.canvas.show(ui, state);
                    });

                    ui.add_space(8.0);
                });
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
