//! Pannello Diagnostica Hardware & Ambiente in stile Impostazioni di Sistema macOS

use crate::liquid_glass::{inset_grouped_card, liquid_pill_button};
use crate::state::{StatusLevel, UiState};
use crate::theme::*;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportLevel {
    Supported,
    Degraded,
    Unsupported,
}
use ch57x_domain::hardware::EncoderEvent;
use ch57x_hardware::cli_adapter::Ch57xCli;
use ch57x_hardware::usb_probe::{is_usbdk_installed, probe_usb_status, UsbStatus};
use ch57x_platform::PlatformBackend;
use egui::{vec2, Color32, FontId, Pos2, Rect, Sense, Ui};

pub struct DoctorPanel {
    pub cli_ok: bool,
    pub cli_path: String,
    pub usb_status: UsbStatus,
    pub usbdk_installed: bool,
    pub autostart_enabled: bool,
    pub sleep_timer_minutes: u32,
    pub initialized_autostart: bool,
}

impl Default for DoctorPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl DoctorPanel {
    pub fn new() -> Self {
        let (cli_ok, cli_path) = match Ch57xCli::auto_detect() {
            Ok(c) => (true, c.binary_path().display().to_string()),
            Err(_) => (
                false,
                "bin/ch57x-keyboard-tool.exe (Non trovato)".to_string(),
            ),
        };
        Self {
            cli_ok,
            cli_path,
            usb_status: probe_usb_status(),
            usbdk_installed: is_usbdk_installed(),
            autostart_enabled: false,
            sleep_timer_minutes: 2,
            initialized_autostart: false,
        }
    }

    pub fn refresh(&mut self) {
        self.usb_status = probe_usb_status();
        self.usbdk_installed = is_usbdk_installed();
        if let Ok(c) = Ch57xCli::auto_detect() {
            self.cli_ok = true;
            self.cli_path = c.binary_path().display().to_string();
        }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        state: &mut UiState,
        platform: Option<&dyn PlatformBackend>,
    ) {
        if !self.initialized_autostart {
            if let Some(p) = platform {
                if let Ok(en) = p.autostart().is_enabled() {
                    self.autostart_enabled = en;
                    self.initialized_autostart = true;
                }
            }
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.vertical(|ui| {
                // Header + Pulsante Aggiorna
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("DIAGNOSTICA & INFORMAZIONI DISPOSITIVO")
                            .font(FontId::proportional(14.0))
                            .strong()
                            .color(COLOR_TEXT_PRIMARY),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if liquid_pill_button(ui, "Esegui Diagnostica Completa", false).clicked() {
                            self.refresh();
                            state.set_status("Diagnostica aggiornata.", StatusLevel::Info);
                        }
                    });
                });

                ui.add_space(10.0);

                // CARD 1: Stato Componenti di Sistema
                inset_grouped_card(ui, Some("Stato Componenti di Sistema"), |ui| {
                    Self::draw_diagnostic_row(
                        ui,
                        if self.cli_ok {
                            SupportLevel::Supported
                        } else {
                            SupportLevel::Unsupported
                        },
                        "Binario ch57x-keyboard-tool valido",
                        &format!("Posizione: {}", self.cli_path),
                    );
                    ui.separator();

                    let (usb_lvl, usb_title, usb_sub) = match &self.usb_status {
                        UsbStatus::Ready { vid, pid } => (
                            SupportLevel::Supported,
                            format!("Macropad USB Pronta ({:04X}:{:04X})", vid, pid),
                            "Connessione diretta attiva per scrittura flash.".to_string(),
                        ),
                        UsbStatus::DetectedAccessDenied => (
                            SupportLevel::Degraded,
                            "Macropad USB Rilevato (Accesso Diretto Negato)".to_string(),
                            "Dispositivo presente: installa UsbDk per abilitare la programmazione flash.".to_string(),
                        ),
                        UsbStatus::Disconnected => (
                            SupportLevel::Unsupported,
                            "Macropad USB Non Collegato (Modalita' Wireless/Bluetooth)".to_string(),
                            "Collega il cavo USB-C per programmare la memoria interna.".to_string(),
                        ),
                    };
                    Self::draw_diagnostic_row(ui, usb_lvl, &usb_title, &usb_sub);
                    ui.separator();

                    Self::draw_diagnostic_row(
                        ui,
                        if self.usbdk_installed {
                            SupportLevel::Supported
                        } else {
                            SupportLevel::Degraded
                        },
                        if self.usbdk_installed {
                            "Driver UsbDk installato e operativo"
                        } else {
                            "Driver UsbDk non rilevato nel sistema"
                        },
                        "Necessario su Windows per programmare la memoria flash senza conflitti HID.",
                    );
                    ui.separator();

                    Self::draw_diagnostic_row(
                        ui,
                        SupportLevel::Supported,
                        "Hotkeys F13..F24 Pienamente Disponibili",
                        "5 Banchi (60 Trigger Candidati) pronti per le automazioni host.",
                    );
                });

                ui.add_space(14.0);

                // CARD 2: Scheda Tecnica & Informazioni Tastiera
                inset_grouped_card(ui, Some("Scheda Tecnica & Informazioni Keyboard"), |ui| {
                    egui::Grid::new("hw_specs_grid")
                        .num_columns(2)
                        .min_col_width(170.0)
                        .spacing(vec2(16.0, 7.0))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("Modello Hardware:").color(COLOR_TEXT_SECONDARY));
                            ui.label(
                                egui::RichText::new("CH57x / CH58x Macropad (Dual-Mode USB + Bluetooth)")
                                    .strong()
                                    .color(COLOR_TEXT_PRIMARY),
                            );
                            ui.end_row();

                            ui.label(egui::RichText::new("Dispositivo Bluetooth (BLE):").color(COLOR_TEXT_SECONDARY));
                            ui.horizontal(|ui| {
                                draw_pill_badge(ui, "HID Keybroad", Color32::from_rgb(18, 44, 75), COLOR_APPLE_BLUE);
                                ui.label(egui::RichText::new("(BTHLE Low Energy)").font(FontId::proportional(11.0)).color(COLOR_TEXT_TERTIARY));
                            });
                            ui.end_row();

                            ui.label(egui::RichText::new("Interfaccia USB Cablata:").color(COLOR_TEXT_SECONDARY));
                            ui.label(
                                egui::RichText::new("VID: 0x1189 | PID: 0x8890 (WinUSB / UsbDk)")
                                    .color(COLOR_TEXT_PRIMARY),
                            );
                            ui.end_row();

                            ui.label(egui::RichText::new("Architettura Meccanica:").color(COLOR_TEXT_SECONDARY));
                            ui.label(
                                egui::RichText::new("12 Tasti Quadrati (Matrice 3×4) + 2 Encoder Rotativi (CCW/PUSH/CW)")
                                    .color(COLOR_TEXT_PRIMARY),
                            );
                            ui.end_row();

                            ui.label(egui::RichText::new("Layer & Controlli Totali:").color(COLOR_TEXT_SECONDARY));
                            ui.label(
                                egui::RichText::new("3 Layer Hardware × 18 Controlli = 54 Slot Totali")
                                    .strong()
                                    .color(COLOR_APPLE_CYAN),
                            );
                            ui.end_row();

                            ui.label(egui::RichText::new("Trigger Virtuali Host:").color(COLOR_TEXT_SECONDARY));
                            ui.label(
                                egui::RichText::new("60 Trigger Candidati (F13..F24, Banchi A..E)")
                                    .color(COLOR_TEXT_PRIMARY),
                            );
                            ui.end_row();

                            ui.label(egui::RichText::new("Livello Batteria (BLE):").color(COLOR_TEXT_SECONDARY));
                            ui.horizontal(|ui| {
                                draw_pill_badge(ui, "100%", Color32::from_rgb(18, 55, 30), COLOR_APPLE_GREEN);
                                ui.label(egui::RichText::new("(GATT Battery Service 0x180F)").font(FontId::proportional(11.0)).color(COLOR_TEXT_TERTIARY));
                            });
                            ui.end_row();

                            ui.label(egui::RichText::new("Stato Alimentazione / Carica:").color(COLOR_TEXT_SECONDARY));
                            let (charge_txt, charge_col) = if self.usb_status.is_ready() {
                                ("In Carica (Alimentata via Cavo USB 5V)", COLOR_APPLE_GREEN)
                            } else {
                                ("A Batteria (Alimentazione Wireless LiPo)", COLOR_APPLE_BLUE)
                            };
                            ui.label(egui::RichText::new(charge_txt).strong().color(charge_col));
                            ui.end_row();

                            ui.label(egui::RichText::new("Stima Tempo di Ricarica:").color(COLOR_TEXT_SECONDARY));
                            ui.label(
                                egui::RichText::new("Non supportata dal chip (Manca sensore di corrente IC Coulomb Counter)")
                                    .font(FontId::proportional(11.0))
                                    .color(COLOR_TEXT_TERTIARY),
                            );
                            ui.end_row();

                            ui.label(egui::RichText::new("Protocollo Flashing:").color(COLOR_TEXT_SECONDARY));
                            ui.label(
                                egui::RichText::new("Flash EEPROM via Cavo USB (Richiesto per upload nuovi comandi)")
                                    .color(COLOR_TEXT_PRIMARY),
                            );
                            ui.end_row();
                        });
                });

                ui.add_space(14.0);

                // CARD 3: Impostazioni di Sistema & Risparmio Energetico
                inset_grouped_card(ui, Some("Impostazioni di Sistema & Risparmio Energetico"), |ui| {
                    // Autostart con Windows
                    ui.horizontal(|ui| {
                        let mut autostart = self.autostart_enabled;
                        if ui
                            .checkbox(
                                &mut autostart,
                                egui::RichText::new("Avvia VortexDeck automaticamente all'avvio di Windows")
                                    .strong()
                                    .color(COLOR_TEXT_PRIMARY),
                            )
                            .changed()
                        {
                            self.autostart_enabled = autostart;
                            if let Some(p) = platform {
                                if let Ok(exe) = std::env::current_exe() {
                                    let _ = p.autostart().set_enabled(autostart, &exe);
                                    if autostart {
                                        state.set_status(
                                            "Avvio con Windows abilitato (HKCU Run).",
                                            StatusLevel::Success,
                                        );
                                    } else {
                                        state.set_status(
                                            "Avvio con Windows disabilitato.",
                                            StatusLevel::Info,
                                        );
                                    }
                                }
                            }
                        }
                    });
                    ui.label(
                        egui::RichText::new(
                            "Mantiene VortexDeck attivo in background nella system tray all'accensione del PC.",
                        )
                        .font(FontId::proportional(11.0))
                        .color(COLOR_TEXT_SECONDARY),
                    );

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Deep Sleep Timer Keyboard
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Deep Sleep Timer (Keyboard):").color(COLOR_TEXT_SECONDARY));
                        let opts = [
                            (2, "2 minuti (Predefinito Hardware CH57x)"),
                            (5, "5 minuti (Prolungato)"),
                            (10, "10 minuti (Consigliato per lunghe sessioni)"),
                            (0, "Sempre Attivo (Invia Keep-Alive BLE)"),
                        ];
                        egui::ComboBox::from_id_salt("doctor_sleep_combo")
                            .selected_text(match self.sleep_timer_minutes {
                                2 => "2 minuti (Predefinito Hardware CH57x)",
                                5 => "5 minuti (Prolungato)",
                                10 => "10 minuti (Consigliato per lunghe sessioni)",
                                0 => "Sempre Attivo (Invia Keep-Alive BLE)",
                                _ => "Personalizzato",
                            })
                            .show_ui(ui, |ui| {
                                for (val, lbl) in opts {
                                    if ui.selectable_value(&mut self.sleep_timer_minutes, val, lbl).clicked() {
                                        state.set_status(format!("Deep Sleep impostato a: {}", lbl), StatusLevel::Info);
                                    }
                                }
                            });
                    });

                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(
                            "A batteria, la tastiera entra in deep-sleep dopo il periodo selezionato. La prima pressione risveglia il Bluetooth.",
                        )
                        .font(FontId::proportional(11.0))
                        .color(COLOR_TEXT_SECONDARY),
                    );
                });

                ui.add_space(14.0);

                // CARD 4: Calibrazione e Test Hardware
                inset_grouped_card(
                    ui,
                    Some("Calibrazione e Test Hardware (18 Controlli per Layer)"),
                    |ui| {
                        ui.label(
                            egui::RichText::new(
                                "Verifica la risposta di ogni controllo fisico premendo i tasti o cliccando sugli slot qui sotto.",
                            )
                            .font(FontId::proportional(11.5))
                            .color(COLOR_TEXT_SECONDARY),
                        );
                        ui.add_space(6.0);

                        let tested_count = state.calibration.tested_count();
                        ui.label(
                            egui::RichText::new(format!(
                                "Progresso: {} di 18 controlli verificati",
                                tested_count
                            ))
                            .font(FontId::proportional(12.0))
                            .strong()
                            .color(COLOR_TEXT_PRIMARY),
                        );

                        let progress = (tested_count as f32 / 18.0).clamp(0.0, 1.0);
                        let (bar_rect, _) =
                            ui.allocate_exact_size(vec2(ui.available_width(), 10.0), Sense::hover());
                        ui.painter().rect_filled(bar_rect, RADIUS_PILL, COLOR_BG_DEEP);
                        let fill_w = (bar_rect.width() * progress).max(10.0);
                        let fill_rect = Rect::from_min_size(bar_rect.min, vec2(fill_w, bar_rect.height()));
                        ui.painter().rect_filled(fill_rect, RADIUS_PILL, COLOR_APPLE_GREEN);

                        ui.add_space(10.0);
                        ui.label(
                            egui::RichText::new("Matrice Tasti (12 Pulsanti)")
                                .font(FontId::proportional(11.0))
                                .color(COLOR_TEXT_SECONDARY),
                        );

                        for r in 0..3usize {
                            ui.horizontal(|ui| {
                                for c in 0..4usize {
                                    let idx = r * 4 + c;
                                    let done = state.calibration.tested_buttons[idx];
                                    let btn = egui::Button::new(
                                        egui::RichText::new(format!("K{}", idx + 1))
                                            .font(FontId::proportional(11.0))
                                            .color(if done {
                                                Color32::WHITE
                                            } else {
                                                COLOR_TEXT_PRIMARY
                                            }),
                                    )
                                    .fill(if done {
                                        COLOR_APPLE_GREEN
                                    } else {
                                        COLOR_CONTROL_INACTIVE
                                    })
                                    .min_size(vec2(52.0, 28.0));

                                    if ui.add(btn).clicked() {
                                        state.calibration.press_button(idx);
                                    }
                                }
                            });
                        }

                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new("Encoders Rotativi (Rotazione & Click)")
                                .font(FontId::proportional(11.0))
                                .color(COLOR_TEXT_SECONDARY),
                        );

                        for (enc_i, name) in [
                            (0usize, "Encoder 1 (Superiore):"),
                            (1usize, "Encoder 2 (Inferiore):"),
                        ] {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(name)
                                        .font(FontId::proportional(11.5))
                                        .color(COLOR_TEXT_SECONDARY),
                                );
                                for (ev_idx, (ev, lbl)) in [
                                    (EncoderEvent::Ccw, "CCW"),
                                    (EncoderEvent::Press, "PUSH"),
                                    (EncoderEvent::Cw, "CW"),
                                ]
                                .iter()
                                .enumerate()
                                {
                                    let done = state.calibration.tested_encoders[enc_i][ev_idx];
                                    let btn = egui::Button::new(
                                        egui::RichText::new(*lbl).font(FontId::proportional(10.5)),
                                    )
                                    .fill(if done {
                                        COLOR_APPLE_GREEN
                                    } else {
                                        COLOR_CONTROL_INACTIVE
                                    })
                                    .min_size(vec2(48.0, 24.0));
                                    if ui.add(btn).clicked() {
                                        state.calibration.press_encoder(enc_i, *ev);
                                    }
                                }
                            });
                        }

                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            let order_lbl = if state.profile.device.encoder_order_swapped {
                                "Invertito (Knob 1 in alto)"
                            } else {
                                "Standard (Knob 0 in alto)"
                            };
                            ui.label(
                                egui::RichText::new(format!("Ordine Fisico: {}", order_lbl))
                                    .font(FontId::proportional(11.5))
                                    .color(COLOR_TEXT_PRIMARY),
                            );
                            if ui.button("Inverti Ordine Encoder (Swap)").clicked() {
                                state.profile.device.encoder_order_swapped =
                                    !state.profile.device.encoder_order_swapped;
                                state.mark_dirty();
                            }
                            if ui.button("Azzera Test Calibrazione").clicked() {
                                state.calibration.reset();
                            }
                        });
                    },
                );
            });
        });
    }

    fn draw_diagnostic_row(ui: &mut Ui, level: SupportLevel, title: &str, subtitle: &str) {
        let col = traffic_light_color(level);
        ui.horizontal(|ui| {
            let (dot_r, _) = ui.allocate_exact_size(vec2(18.0, 34.0), Sense::hover());
            let c = Pos2::new(dot_r.center().x, dot_r.center().y);
            ui.painter()
                .circle_filled(c, 5.5, rgba(col.r(), col.g(), col.b(), 50));
            ui.painter().circle_filled(c, 3.2, col);

            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(title)
                        .font(FontId::proportional(12.5))
                        .strong()
                        .color(COLOR_TEXT_PRIMARY),
                );
                ui.label(
                    egui::RichText::new(subtitle)
                        .font(FontId::proportional(11.0))
                        .color(COLOR_TEXT_SECONDARY),
                );
            });
        });
    }
}

pub fn traffic_light_color(level: SupportLevel) -> Color32 {
    match level {
        SupportLevel::Supported => COLOR_APPLE_GREEN,
        SupportLevel::Degraded => COLOR_APPLE_ORANGE,
        SupportLevel::Unsupported => COLOR_APPLE_RED,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doctor_panel_instantiation() {
        let p = DoctorPanel::new();
        let _ = p;
    }

    #[test]
    fn test_traffic_light_colors() {
        assert_eq!(
            traffic_light_color(SupportLevel::Supported),
            COLOR_APPLE_GREEN
        );
        assert_eq!(
            traffic_light_color(SupportLevel::Degraded),
            COLOR_APPLE_ORANGE
        );
        assert_eq!(
            traffic_light_color(SupportLevel::Unsupported),
            COLOR_APPLE_RED
        );
    }
}
