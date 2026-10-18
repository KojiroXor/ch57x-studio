//! Device doctor diagnostic panel
//!
//! Visualizza:
//! - Semafori Verde/Giallo/Rosso per:
//!   - Presenza e validità del binario `ch57x-keyboard-tool`.
//!   - Rilevamento USB del Macropad (`VID 0x1189`, `PID 0x8890` / `0x8840` / `0x8842`).
//!   - Stato del driver `UsbDk`.
//!   - Stato registrazione hotkey F13..F24.
//! - Wizard di calibrazione fisica (test tasti, test encoder 0/1, test swap).

use crate::state::{StatusLevel, UiState};
use ch57x_domain::hardware::EncoderEvent;
use ch57x_hardware::usb_probe::UsbStatus;
use ch57x_platform::PlatformBackend;
use egui::{
    vec2, Color32, FontId, ProgressBar, Sense, Stroke, Ui,
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
            TrafficLight::Green => Color32::from_rgb(34, 197, 94), // Green 500
            TrafficLight::Yellow => Color32::from_rgb(234, 179, 8), // Yellow 500
            TrafficLight::Red => Color32::from_rgb(239, 68, 68),   // Red 500
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
                    egui::RichText::new("🩺 DIAGNOSTICA HARDWARE & DRIVER")
                        .font(FontId::proportional(16.0))
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🔄 Esegui Diagnostica Completa").clicked() {
                        state.doctor_cache.refresh(platform);
                        state.set_status("Diagnostica aggiornata.", StatusLevel::Info);
                    }
                });
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // -------------------------------------------------------------
            // SEZIONE 1: Semafori di Stato Sistema
            // -------------------------------------------------------------
            ui.label(egui::RichText::new("STATO COMPONENTI").font(FontId::proportional(12.0)).strong());
            ui.add_space(4.0);

            ui.group(|ui| {
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

                ui.separator();

                // 2. Rilevamento USB del Macropad (VID 0x1189, PID 0x8890 / 0x8840 / 0x8842)
                let (usb_light, usb_title, usb_sub) = match state.doctor_cache.usb_status {
                    UsbStatus::Ready { vid, pid } => (
                        TrafficLight::Green,
                        "Macropad USB Connesso e Pronto",
                        format!("Rilevato dispositivo ID {:04X}:{:04X} pronto per il flashing.", vid, pid),
                    ),
                    UsbStatus::DetectedAccessDenied => (
                        TrafficLight::Yellow,
                        "Macropad USB Rilevato (Accesso Negato)",
                        "Dispositivo presente ma il driver UsbDk o i permessi non sono disponibili.".into(),
                    ),
                    UsbStatus::Disconnected => (
                        TrafficLight::Red,
                        "Nessun Macropad USB Rilevato",
                        "Collega il macropad via USB-C (necessario per scrittura configurazione flash).".into(),
                    ),
                };
                Self::draw_diagnostic_row(ui, usb_light, usb_title, &usb_sub);

                ui.separator();

                // 3. Driver UsbDk
                let (usbdk_light, usbdk_title, usbdk_sub): (TrafficLight, &str, String) = if state.doctor_cache.usbdk_installed {
                    (
                        TrafficLight::Green,
                        "Driver UsbDk Installato",
                        "Il servizio UsbDk è attivo e consente l'accesso USB diretto.".to_string(),
                    )
                } else if cfg!(target_os = "windows") {
                    (
                        TrafficLight::Yellow,
                        "Driver UsbDk non rilevato",
                        "Necessario su Windows per programmare il firmware flash senza conflitti HID.".to_string(),
                    )
                } else {
                    (
                        TrafficLight::Green,
                        "UsbDk non richiesto",
                        "Sistemi Linux utilizzano le regole udev standard.".to_string(),
                    )
                };
                Self::draw_diagnostic_row(ui, usbdk_light, usbdk_title, &usbdk_sub);

                ui.separator();

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
                        "Tasti Hotkey F13..F24 Liberi",
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
            ui.separator();
            ui.add_space(12.0);

            // -------------------------------------------------------------
            // SEZIONE 2: Wizard di Calibrazione Fisica
            // -------------------------------------------------------------
            Self::show_calibration_wizard(ui, state);
        });
    }

    /// Disegna una riga di diagnostica con semaforo circolare, titolo e descrizione
    fn draw_diagnostic_row(ui: &mut Ui, light: TrafficLight, title: &str, subtitle: &str) {
        ui.horizontal(|ui| {
            let (circle_rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
            let center = circle_rect.center();

            // Glow semaforo
            let glow_col = Color32::from_rgba_unmultiplied(
                light.color().r(),
                light.color().g(),
                light.color().b(),
                80,
            );
            ui.painter().circle_filled(center, 7.0, glow_col);
            ui.painter().circle_filled(center, 5.0, light.color());
            ui.painter().circle_stroke(center, 5.0, Stroke::new(1.0_f32, Color32::WHITE));

            ui.vertical(|ui| {
                ui.label(egui::RichText::new(title).font(FontId::proportional(12.0)).strong());
                ui.label(
                    egui::RichText::new(subtitle)
                        .font(FontId::proportional(10.5))
                        .color(Color32::from_rgb(161, 161, 170)),
                );
            });
        });
    }

    /// Wizard di calibrazione fisica (test tasti, test encoder 0/1, test swap)
    fn show_calibration_wizard(ui: &mut Ui, state: &mut UiState) {
        ui.label(
            egui::RichText::new("🎯 WIZARD DI CALIBRAZIONE FISICA")
                .font(FontId::proportional(13.0))
                .strong(),
        );
        ui.label(
            egui::RichText::new(
                "Verifica la risposta hardware di ogni controllo fisico premendo i tasti e ruotando le manopole.",
            )
            .color(Color32::from_rgb(161, 161, 170)),
        );

        ui.add_space(8.0);

        // Barra di progresso
        let tested = state.calibration.tested_count();
        let total = state.calibration.total_controls();
        let pct = tested as f32 / total as f32;

        ui.horizontal(|ui| {
            ui.label(format!("Progresso: {} / {} controlli verificati", tested, total));
            if state.calibration.is_complete() {
                ui.label(
                    egui::RichText::new("✓ COMPLETATO")
                        .color(Color32::from_rgb(34, 197, 94))
                        .strong(),
                );
            }
        });
        ui.add(ProgressBar::new(pct).desired_width(ui.available_width()));

        ui.add_space(8.0);

        // 1. Griglia di test tasti 4x3
        ui.group(|ui| {
            ui.label(egui::RichText::new("Test Tasti (12 Pulsanti)").font(FontId::proportional(11.0)).strong());
            ui.add_space(4.0);

            for r in 0..3 {
                ui.horizontal(|ui| {
                    for c in 0..4 {
                        let idx = r * 4 + c;
                        let is_tested = state.calibration.tested_buttons[idx];

                        let btn_text = if is_tested {
                            format!("✓ K{}", idx + 1)
                        } else {
                            format!("K{}", idx + 1)
                        };

                        let btn_col = if is_tested {
                            Color32::from_rgb(34, 197, 94)
                        } else {
                            Color32::from_rgb(161, 161, 170)
                        };

                        let btn = egui::Button::new(
                            egui::RichText::new(btn_text).color(btn_col).strong(),
                        )
                        .min_size(vec2(52.0, 32.0));

                        if ui.add(btn).clicked() {
                            state.calibration.press_button(idx);
                        }
                    }
                });
            }
        });

        ui.add_space(8.0);

        // 2. Test Encoder 0 e Encoder 1
        ui.group(|ui| {
            ui.label(egui::RichText::new("Test Encoders (Rotazione & Pressione)").font(FontId::proportional(11.0)).strong());
            ui.add_space(4.0);

            for enc_idx in 0..2 {
                let enc_name = if enc_idx == 0 { "Encoder 1 (Superiore)" } else { "Encoder 2 (Inferiore)" };
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", enc_name));

                    let events = [
                        (EncoderEvent::Ccw, "↺ CCW", 0),
                        (EncoderEvent::Press, "● PRESS", 1),
                        (EncoderEvent::Cw, "↻ CW", 2),
                    ];

                    for (ev, label, ev_idx) in events {
                        let is_tested = state.calibration.tested_encoders[enc_idx][ev_idx];
                        let col = if is_tested {
                            Color32::from_rgb(34, 197, 94)
                        } else {
                            Color32::from_rgb(161, 161, 170)
                        };

                        let text = if is_tested {
                            format!("✓ {}", label)
                        } else {
                            label.to_string()
                        };

                        if ui.add(egui::Button::new(egui::RichText::new(text).color(col))).clicked() {
                            state.calibration.press_encoder(enc_idx, ev);
                        }
                    }
                });
            }
        });

        ui.add_space(8.0);

        // 3. Test Swap e Configurazione Swap
        ui.group(|ui| {
            ui.label(egui::RichText::new("Orientamento e Swap Encoders").font(FontId::proportional(11.0)).strong());
            ui.horizontal(|ui| {
                let status_swap = if state.calibration.swap_order {
                    "Invertito (Knob 0 ↔ Knob 1)"
                } else {
                    "Standard (Normale)"
                };

                ui.label(format!("Mappatura fisica attuale: {}", status_swap));

                if ui.button("⇄ Inverti Ordine Encoder (Swap)").clicked() {
                    state.calibration.toggle_swap();
                    state.profile.device.encoder_order_swapped = state.calibration.swap_order;
                    state.mark_dirty();
                    state.set_status(
                        format!(
                            "Ordine encoder aggiornato: {}",
                            if state.calibration.swap_order { "Invertito" } else { "Normale" }
                        ),
                        StatusLevel::Info,
                    );
                }
            });
            ui.label(
                egui::RichText::new(
                    "Se ruotando la manopola superiore risponde il controllo inferiore o viceversa, attiva lo swap.",
                )
                .font(FontId::proportional(10.5))
                .color(Color32::from_rgb(161, 161, 170)),
            );
        });

        ui.add_space(8.0);

        // Pulsante di Reset Test
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
        assert_eq!(TrafficLight::Green.color(), Color32::from_rgb(34, 197, 94));
        assert_eq!(TrafficLight::Yellow.color(), Color32::from_rgb(234, 179, 8));
        assert_eq!(TrafficLight::Red.color(), Color32::from_rgb(239, 68, 68));
    }

    #[test]
    fn test_doctor_panel_instantiation() {
        let doctor = DoctorPanel::new();
        let _ = doctor;
    }
}
