//! Device doctor diagnostic panel in Cupertino Pro Dark style
//!
//! Visualizza:
//! - Indicatori di stato per binario CLI, USB hardware, driver UsbDk e Hotkeys F13..F24
//! - Inset Grouped Cards per diagnostica e calibrazione fisica
//! - Zero emoji: indicatori geometrici con glow Apple

use crate::state::{StatusLevel, UiState};
use crate::theme::*;
use ch57x_domain::hardware::EncoderEvent;
use ch57x_hardware::usb_probe::UsbStatus;
use ch57x_platform::PlatformBackend;
use egui::{
    vec2, Color32, FontId, ProgressBar, Rounding, Sense, Stroke, Ui,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrafficLight {
    Green,
    Yellow,
    Red,
}

impl TrafficLight {
    pub fn color(&self) -> Color32 {
        match self {
            TrafficLight::Green => COLOR_APPLE_GREEN,
            TrafficLight::Yellow => COLOR_APPLE_ORANGE,
            TrafficLight::Red => COLOR_APPLE_RED,
        }
    }
}

pub struct DoctorPanel;

impl Default for DoctorPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl DoctorPanel {
    pub fn new() -> Self {
        Self
    }

    /// Disegna il pannello di diagnostica e calibrazione hardware
    pub fn show(
        &mut self,
        ui: &mut Ui,
        state: &mut UiState,
        platform: Option<&dyn PlatformBackend>,
    ) {
        // Auto-refresh se prima apertura
        if state.doctor_cache.last_checked.is_none() {
            state.doctor_cache.refresh(platform);
        }

        ui.vertical(|ui| {
            // Header e pulsante Refresh
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("DIAGNOSTICA HARDWARE & AMBIENTE")
                        .font(FontId::proportional(15.0))
                        .strong()
                        .color(COLOR_TEXT_PRIMARY),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Esegui Diagnostica Completa").clicked() {
                        state.doctor_cache.refresh(platform);
                        state.set_status("Diagnostica di sistema aggiornata.", StatusLevel::Info);
                    }
                });
            });

            ui.add_space(10.0);

            // -------------------------------------------------------------
            // SEZIONE 1: Indicatori di Stato Sistema (Card Inset)
            // -------------------------------------------------------------
            inset_grouped_card(ui, Some("Stato Componenti di Sistema"), |ui| {
                // 1. Binario ch57x-keyboard-tool
                let (cli_light, cli_title, cli_sub) = if state.doctor_cache.cli_valid {
                    let path_str = state
                        .doctor_cache
                        .cli_path
                        .as_ref()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Trovato nel sistema".into());
                    (
                        TrafficLight::Green,
                        "Binario ch57x-keyboard-tool valido",
                        format!("Posizione: {}", path_str),
                    )
                } else if let Some(err) = &state.doctor_cache.cli_error {
                    (
                        TrafficLight::Red,
                        "Binario ch57x-keyboard-tool non valido o mancante",
                        format!("Errore: {}", err),
                    )
                } else {
                    (
                        TrafficLight::Red,
                        "Binario ch57x-keyboard-tool non trovato",
                        "Posiziona l'eseguibile nella cartella bin/ o nel PATH di sistema.".into(),
                    )
                };
                Self::draw_diagnostic_row(ui, cli_light, cli_title, &cli_sub);

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                // 2. Rilevamento USB del Macropad (VID 0x1189, PID 0x8890 / 0x8840 / 0x8842)
                let (usb_light, usb_title, usb_sub) = match state.doctor_cache.usb_status {
                    UsbStatus::Ready { vid, pid } => (
                        TrafficLight::Green,
                        "Macropad USB Connesso e Pronto",
                        format!("Dispositivo ID {:04X}:{:04X} pronto per il flashing hardware.", vid, pid),
                    ),
                    UsbStatus::DetectedAccessDenied => (
                        TrafficLight::Yellow,
                        "Macropad USB Rilevato (Accesso Diretto Negato)",
                        "Dispositivo presente: installa il driver UsbDk per consentire la programmazione flash.".into(),
                    ),
                    UsbStatus::Disconnected => (
                        TrafficLight::Red,
                        "Nessun Macropad USB Rilevato",
                        "Collega il macropad via cavo USB-C per abilitare la scrittura hardware.".into(),
                    ),
                };
                Self::draw_diagnostic_row(ui, usb_light, usb_title, &usb_sub);

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                // 3. Driver UsbDk
                let (usbdk_light, usbdk_title, usbdk_sub): (TrafficLight, &str, String) = if state.doctor_cache.usbdk_installed {
                    (
                        TrafficLight::Green,
                        "Driver UsbDk Installato e Attivo",
                        "Il servizio UsbDk consente l'accesso USB diretto di basso livello.".to_string(),
                    )
                } else if cfg!(target_os = "windows") {
                    (
                        TrafficLight::Yellow,
                        "Driver UsbDk non rilevato nel sistema",
                        "Necessario su Windows per programmare il firmware flash senza conflitti HID.".to_string(),
                    )
                } else {
                    (
                        TrafficLight::Green,
                        "UsbDk non richiesto",
                        "I sistemi Linux utilizzano le regole udev standard.".to_string(),
                    )
                };
                Self::draw_diagnostic_row(ui, usbdk_light, usbdk_title, &usbdk_sub);

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                // 4. Registrazione Hotkey F13..F24
                let (hk_light, hk_title, hk_sub) = if !state.doctor_cache.hotkeys_tested {
                    (
                        TrafficLight::Yellow,
                        "Hotkeys F13..F24 non verificate",
                        "Clicca 'Esegui Diagnostica' per testare conflitti di hotkey nel sistema.".into(),
                    )
                } else if state.doctor_cache.hotkey_conflicts.is_empty() {
                    (
                        TrafficLight::Green,
                        "Hotkeys F13..F24 Pienamente Disponibili",
                        "Nessun conflitto rilevato con altre applicazioni in esecuzione.".into(),
                    )
                } else {
                    let conflicts_str = state
                        .doctor_cache
                        .hotkey_conflicts
                        .iter()
                        .map(|t| t.yaml_token())
                        .collect::<Vec<_>>()
                        .join(", ");
                    (
                        TrafficLight::Yellow,
                        "Conflitti rilevati su Hotkey F13..F24",
                        format!("In conflitto con altre app: {}", conflicts_str),
                    )
                };
                Self::draw_diagnostic_row(ui, hk_light, hk_title, &hk_sub);
            });

            ui.add_space(12.0);

            // -------------------------------------------------------------
            // SEZIONE 2: Wizard di Calibrazione Fisica (Card Inset)
            // -------------------------------------------------------------
            inset_grouped_card(ui, Some("Calibrazione e Test Hardware"), |ui| {
                Self::show_calibration_wizard(ui, state);
            });
        });
    }

    /// Disegna una riga di diagnostica con dot luminoso Apple
    fn draw_diagnostic_row(ui: &mut Ui, light: TrafficLight, title: &str, subtitle: &str) {
        ui.horizontal(|ui| {
            let (circle_rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
            let center = circle_rect.center();

            let color = light.color();
            let halo = Color32::from_rgba_premultiplied(
                (color.r() as f32 * 0.25) as u8,
                (color.g() as f32 * 0.25) as u8,
                (color.b() as f32 * 0.25) as u8,
                60,
            );

            ui.painter().circle_filled(center, 7.0, halo);
            ui.painter().circle_filled(center, 4.0, color);
            ui.painter().circle_stroke(center, 4.0, Stroke::new(1.0_f32, Color32::WHITE));

            ui.vertical(|ui| {
                ui.label(egui::RichText::new(title).font(FontId::proportional(12.5)).strong().color(COLOR_TEXT_PRIMARY));
                ui.label(
                    egui::RichText::new(subtitle)
                        .font(FontId::proportional(11.0))
                        .color(COLOR_TEXT_SECONDARY),
                );
            });
        });
    }

    /// Wizard di calibrazione fisica
    fn show_calibration_wizard(ui: &mut Ui, state: &mut UiState) {
        ui.label(
            egui::RichText::new(
                "Verifica la risposta hardware di ogni controllo fisico premendo i tasti e ruotando le manopole.",
            )
            .font(FontId::proportional(11.5))
            .color(COLOR_TEXT_SECONDARY),
        );

        ui.add_space(8.0);

        let tested = state.calibration.tested_count();
        let total = state.calibration.total_controls();
        let pct = tested as f32 / total as f32;

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(format!("Progresso: {} di {} controlli testati", tested, total)).color(COLOR_TEXT_PRIMARY));
            if state.calibration.is_complete() {
                ui.label(
                    egui::RichText::new("Completato")
                        .color(COLOR_APPLE_GREEN)
                        .strong(),
                );
            }
        });
        ui.add_space(3.0);
        ui.add(ProgressBar::new(pct).desired_width(ui.available_width()).fill(COLOR_APPLE_GREEN));

        ui.add_space(10.0);

        // 1. Griglia Tasti 4x3
        ui.label(egui::RichText::new("Matrice Tasti (12 Pulsanti)").font(FontId::proportional(11.0)).strong().color(COLOR_TEXT_SECONDARY));
        ui.add_space(4.0);

        for r in 0..3 {
            ui.horizontal(|ui| {
                for c in 0..4 {
                    let idx = r * 4 + c;
                    let is_tested = state.calibration.tested_buttons[idx];

                    let btn_text = format!("K{}", idx + 1);
                    let btn_col = if is_tested {
                        COLOR_APPLE_GREEN
                    } else {
                        COLOR_TEXT_PRIMARY
                    };

                    let btn = egui::Button::new(
                        egui::RichText::new(btn_text).color(btn_col).strong(),
                    )
                    .fill(if is_tested { Color32::from_rgb(25, 45, 30) } else { COLOR_CONTROL_INACTIVE })
                    .stroke(Stroke::new(1.0_f32, if is_tested { COLOR_APPLE_GREEN } else { COLOR_BORDER_SUBTLE }))
                    .rounding(Rounding::same(6.0))
                    .min_size(vec2(52.0, 30.0));

                    if ui.add(btn).clicked() {
                        state.calibration.press_button(idx);
                    }
                }
            });
        }

        ui.add_space(10.0);

        // 2. Test Encoders
        ui.label(egui::RichText::new("Encoders Rotativi (Rotazione & Click)").font(FontId::proportional(11.0)).strong().color(COLOR_TEXT_SECONDARY));
        ui.add_space(4.0);

        for enc_idx in 0..2 {
            let enc_name = if enc_idx == 0 { "Encoder 1 (Superiore)" } else { "Encoder 2 (Inferiore)" };
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("{}:", enc_name)).color(COLOR_TEXT_SECONDARY));

                let events = [
                    (EncoderEvent::Ccw, "CCW", 0),
                    (EncoderEvent::Press, "PUSH", 1),
                    (EncoderEvent::Cw, "CW", 2),
                ];

                for (ev, label, ev_idx) in events {
                    let is_tested = state.calibration.tested_encoders[enc_idx][ev_idx];
                    let col = if is_tested {
                        COLOR_APPLE_GREEN
                    } else {
                        COLOR_TEXT_PRIMARY
                    };

                    let btn = egui::Button::new(egui::RichText::new(label).color(col).strong())
                        .fill(if is_tested { Color32::from_rgb(25, 45, 30) } else { COLOR_CONTROL_INACTIVE })
                        .stroke(Stroke::new(1.0_f32, if is_tested { COLOR_APPLE_GREEN } else { COLOR_BORDER_SUBTLE }))
                        .rounding(Rounding::same(6.0));

                    if ui.add(btn).clicked() {
                        state.calibration.press_encoder(enc_idx, ev);
                    }
                }
            });
        }

        ui.add_space(10.0);

        // 3. Swap Encoders
        ui.horizontal(|ui| {
            let status_swap = if state.calibration.swap_order {
                "Invertito (Knob 0 <-> Knob 1)"
            } else {
                "Standard (Knob 0 in alto)"
            };

            ui.label(egui::RichText::new(format!("Ordine Fisico: {}", status_swap)).color(COLOR_TEXT_PRIMARY));

            if ui.button("Inverti Ordine Encoder (Swap)").clicked() {
                state.calibration.toggle_swap();
                state.profile.device.encoder_order_swapped = state.calibration.swap_order;
                state.mark_dirty();
                state.set_status(
                    format!(
                        "Ordine encoder aggiornato: {}",
                        if state.calibration.swap_order { "Invertito" } else { "Standard" }
                    ),
                    StatusLevel::Info,
                );
            }
        });

        ui.add_space(6.0);
        if ui.button("Azzera Test Calibrazione").clicked() {
            state.calibration.reset();
            state.set_status("Wizard calibrazione azzerato.", StatusLevel::Info);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traffic_light_colors() {
        assert_eq!(TrafficLight::Green.color(), COLOR_APPLE_GREEN);
        assert_eq!(TrafficLight::Yellow.color(), COLOR_APPLE_ORANGE);
        assert_eq!(TrafficLight::Red.color(), COLOR_APPLE_RED);
    }

    #[test]
    fn test_doctor_panel_instantiation() {
        let doctor = DoctorPanel::new();
        let _ = doctor;
    }
}
