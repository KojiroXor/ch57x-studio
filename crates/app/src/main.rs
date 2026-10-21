#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
//! VortexDeck (CH57x Studio) — Main Executable Entrypoint
//! Toolbar macOS Sequoia senza collisioni (flexbox nativa) e architettura multi-pannello.

mod coordinator;
mod tray;

use std::env;
use std::time::{Duration, Instant};
use tracing::info;

use ch57x_hardware::usb_probe::UsbStatus;
use ch57x_ui::liquid_glass::{inset_grouped_card, paint_specular_rim_light, segmented_control};
use ch57x_ui::state::{ActiveView, StatusLevel};
use ch57x_ui::theme::*;

use coordinator::{AppCoordinator, CoordinatorCommand, HardwareFlashPreview};
use egui::{vec2, Align, Color32, FontId, Layout, Margin, Pos2, Rounding, Stroke, TopBottomPanel};
use tray::{TrayAction, TrayHandle};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Forza sempre la working directory nella cartella dell'eseguibile (fondamentale per l'avvio automatico con Windows!)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let _ = std::env::set_current_dir(exe_dir);
        }
    }

    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();

    #[cfg(target_os = "windows")]
    if args.len() > 1 {
        unsafe {
            let _ = windows::Win32::System::Console::AttachConsole(
                windows::Win32::System::Console::ATTACH_PARENT_PROCESS,
            );
        }
    }

    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_help();
        return Ok(());
    }

    if args.contains(&"--version".to_string()) || args.contains(&"-v".to_string()) {
        println!("VortexDeck (CH57x Studio) v{}", VERSION);
        return Ok(());
    }

    if args.contains(&"--doctor".to_string()) {
        run_cli_doctor();
        return Ok(());
    }

    let is_background = args.contains(&"--background".to_string());
    let is_safe_mode = args.contains(&"--safe-mode".to_string());

    #[cfg(target_os = "windows")]
    let _mutex_guard = {
        use windows::core::w;
        use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS, LPARAM, WPARAM};
        use windows::Win32::System::Threading::CreateMutexW;
        use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, PostMessageW, WM_COMMAND};

        let handle =
            unsafe { CreateMutexW(None, true, w!("Local\\VortexDeck_SingleInstance_Mutex")) };
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            unsafe {
                if let Ok(tray_hwnd) =
                    FindWindowW(w!("VortexDeckTrayClass"), w!("VortexDeckTrayWindow"))
                {
                    let _ = PostMessageW(Some(tray_hwnd), WM_COMMAND, WPARAM(1001), LPARAM(0));
                }
            }
            return Ok(());
        }
        handle
    };

    info!(
        "Avvio VortexDeck v{} (safe_mode: {}, background: {})",
        VERSION, is_safe_mode, is_background
    );

    let mut coordinator = AppCoordinator::new(is_safe_mode)?;
    let tray = TrayHandle::new();

    if is_background {
        let _tray_handle = &tray;
        loop {
            while let Ok(action) = _tray_handle.receiver.try_recv() {
                match action {
                    TrayAction::ShowWindow => {
                        #[cfg(target_os = "windows")]
                        if let Ok(handle) = _mutex_guard {
                            unsafe {
                                let _ = windows::Win32::Foundation::CloseHandle(handle);
                            };
                        }
                        let exe = std::env::current_exe()?;
                        let mut cmd = std::process::Command::new(exe);
                        #[cfg(target_os = "windows")]
                        {
                            use std::os::windows::process::CommandExt;
                            cmd.creation_flags(0x08000000);
                        }
                        let _ = cmd.spawn();
                        return Ok(());
                    }
                    TrayAction::TogglePause => {
                        coordinator
                            .command_sender
                            .send(CoordinatorCommand::TogglePause)?;
                    }
                    TrayAction::ToggleAutostart => {
                        coordinator.toggle_autostart();
                    }
                    TrayAction::ShowHud => {}
                    TrayAction::Quit => {
                        drop(tray);
                        std::process::exit(0);
                    }
                }
            }
            coordinator.process_pending_commands();
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("VortexDeck — CH57x Studio")
            .with_inner_size([1260.0, 800.0])
            .with_min_inner_size([1040.0, 660.0])
            .with_visible(true)
            .with_active(true),
        ..Default::default()
    };

    let result = eframe::run_native(
        "VortexDeck",
        native_options,
        Box::new(|cc| {
            tray.set_repaint_ctx(cc.egui_ctx.clone());
            apply_cupertino_theme(&cc.egui_ctx);
            Ok(Box::new(VortexDeckApp {
                coordinator,
                studio_ui: ch57x_ui::StudioUi::new(),
                tray,
                is_quitting: false,
                flash_modal_preview: None,
                hud_dismiss_at: None,
            }))
        }),
    );

    result?;
    Ok(())
}

struct VortexDeckApp {
    coordinator: AppCoordinator,
    studio_ui: ch57x_ui::StudioUi,
    tray: TrayHandle,
    is_quitting: bool,
    flash_modal_preview: Option<HardwareFlashPreview>,
    hud_dismiss_at: Option<Instant>,
}

impl eframe::App for VortexDeckApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.viewport().close_requested()) && !self.is_quitting {
            if self.coordinator.ui_state.is_dirty {
                self.coordinator.save_active_profile();
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }

        while let Ok(action) = self.tray.receiver.try_recv() {
            match action {
                TrayAction::ShowWindow => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                }
                TrayAction::TogglePause => {
                    let _ = self
                        .coordinator
                        .command_sender
                        .send(CoordinatorCommand::TogglePause);
                }
                TrayAction::ToggleAutostart => {
                    self.coordinator.autostart_enabled =
                        self.coordinator.platform.is_autostart_enabled();
                    let msg = if self.coordinator.autostart_enabled {
                        "Avvio automatico con Windows ATTIVATO (partirà in background all'accensione)."
                    } else {
                        "Avvio automatico con Windows DISATTIVATO."
                    };
                    self.coordinator
                        .ui_state
                        .set_status(msg, StatusLevel::Success);
                }
                TrayAction::ShowHud => {
                    self.hud_dismiss_at = Some(Instant::now() + Duration::from_millis(1500));
                    ctx.request_repaint();
                }
                TrayAction::Quit => {
                    if self.coordinator.ui_state.is_dirty {
                        self.coordinator.save_active_profile();
                    }
                    std::process::exit(0);
                }
            }
        }

        self.coordinator.process_pending_commands();

        // ---------------------------------------------------------------------
        // BARRA SUPERIORE UNIFICATA (Layout Orizzontale Fluido = ZERO Sovrapposizioni!)
        // ---------------------------------------------------------------------
        TopBottomPanel::top("macos_toolbar")
            .exact_height(54.0)
            .frame(egui::Frame {
                inner_margin: Margin::symmetric(18.0, 10.0),
                fill: COLOR_TOOLBAR_BG,
                stroke: Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    // 1. Titolo e Stato USB a Sinistra
                    ui.label(
                        egui::RichText::new("VortexDeck")
                            .font(FontId::proportional(16.0))
                            .strong()
                            .color(COLOR_TEXT_PRIMARY),
                    );

                    ui.add_space(6.0);

                    let usb_status = self.coordinator.get_usb_status();
                    match usb_status {
                        UsbStatus::Ready { vid, pid } => {
                            draw_pill_badge(
                                ui,
                                &format!("USB Pronta {:04X}:{:04X}", vid, pid),
                                Color32::TRANSPARENT,
                                COLOR_APPLE_GREEN,
                            );
                        }
                        UsbStatus::DetectedAccessDenied => {
                            draw_pill_badge(
                                ui,
                                "USB Rilevata (Richiede UsbDk)",
                                Color32::TRANSPARENT,
                                COLOR_APPLE_ORANGE,
                            );
                        }
                        UsbStatus::Disconnected => {
                            draw_pill_badge(
                                ui,
                                "Modalita Runtime Host",
                                Color32::TRANSPARENT,
                                COLOR_TEXT_SECONDARY,
                            );
                        }
                    }

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(12.0);

                    // 2. Selettore Vista Principale (Non entra mai in collisione con i pulsanti a destra)
                    let mut current_view = self.coordinator.ui_state.active_view;
                    let view_opts = [
                        (ActiveView::Editor, "Configuratore"),
                        (ActiveView::Doctor, "Diagnostica"),
                        (ActiveView::Hud, "HUD Preview"),
                    ];
                    if segmented_control(ui, &mut current_view, &view_opts) {
                        self.coordinator.ui_state.active_view = current_view;
                    }

                    // 3. Azioni a Destra
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let flash_btn = egui::Button::new(
                            egui::RichText::new("Scrivi su Dispositivo...")
                                .font(FontId::proportional(12.0))
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(COLOR_APPLE_BLUE)
                        .stroke(Stroke::new(1.0_f32, rgba(255, 255, 255, 55)))
                        .rounding(Rounding::same(8.0))
                        .min_size(vec2(0.0, 28.0));

                        let is_flashing = self.coordinator.is_flashing();
                        if ui.add_enabled(!is_flashing, flash_btn).clicked() {
                            match self.coordinator.generate_flash_preview() {
                                Ok(preview) => self.flash_modal_preview = Some(preview),
                                Err(e) => {
                                    self.coordinator.ui_state.set_status(e, StatusLevel::Error)
                                }
                            }
                        }

                        if ui.button("Salva").clicked() {
                            self.coordinator.save_active_profile();
                        }

                        if ui.button("Valida").clicked() {
                            self.coordinator.validate_configuration();
                        }

                        let is_paused = self.coordinator.dispatcher.is_paused();
                        let (pause_lbl, pause_col) = if is_paused {
                            ("Riprendi", COLOR_APPLE_ORANGE)
                        } else {
                            ("Pausa", COLOR_TEXT_SECONDARY)
                        };
                        if ui
                            .button(egui::RichText::new(pause_lbl).color(pause_col))
                            .clicked()
                        {
                            let _ = self
                                .coordinator
                                .command_sender
                                .send(CoordinatorCommand::TogglePause);
                        }

                        // Toggle Avvio con Windows (ON/OFF)
                        let is_auto = self.coordinator.autostart_enabled;
                        let (auto_text, auto_fill, auto_stroke, auto_text_col) = if is_auto {
                            (
                                "Avvio Win: ON",
                                Color32::from_rgba_unmultiplied(48, 209, 88, 38),
                                Stroke::new(1.0_f32, COLOR_APPLE_GREEN),
                                COLOR_APPLE_GREEN,
                            )
                        } else {
                            (
                                "Avvio Win: OFF",
                                COLOR_CONTROL_INACTIVE,
                                Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE),
                                COLOR_TEXT_SECONDARY,
                            )
                        };

                        let auto_btn = egui::Button::new(
                            egui::RichText::new(auto_text)
                                .font(FontId::proportional(11.5))
                                .strong()
                                .color(auto_text_col),
                        )
                        .fill(auto_fill)
                        .stroke(auto_stroke)
                        .rounding(Rounding::same(8.0))
                        .min_size(vec2(0.0, 28.0));

                        if ui
                            .add(auto_btn)
                            .on_hover_text(
                                "Attiva o disattiva l'avvio automatico di VortexDeck in background all'accensione di Windows.",
                            )
                            .clicked()
                        {
                            self.coordinator.toggle_autostart();
                        }

                        // Badge di avviso se è necessario il Flash USB per aggiornare il firmware / BLE
                        if self.coordinator.requires_hardware_flash() {
                            let flash_warn_btn = egui::Button::new(
                                egui::RichText::new(
                                    "⚠️ Flash USB necessario per aggiornare il Bluetooth",
                                )
                                .font(FontId::proportional(11.0))
                                .strong()
                                .color(COLOR_APPLE_ORANGE),
                            )
                            .fill(Color32::from_rgba_premultiplied(60, 40, 10, 220))
                            .stroke(Stroke::new(1.0_f32, COLOR_APPLE_ORANGE))
                            .rounding(Rounding::same(8.0))
                            .min_size(vec2(0.0, 28.0));

                            if ui
                                .add(flash_warn_btn)
                                .on_hover_text(
                                    "Le azioni host o le mappature sono cambiate: collega il cavo USB e premi 'Scrivi su Dispositivo...' per renderle effettive anche via Bluetooth o ad app chiusa.",
                                )
                                .clicked()
                            {
                                match self.coordinator.generate_flash_preview() {
                                    Ok(preview) => self.flash_modal_preview = Some(preview),
                                    Err(e) => {
                                        self.coordinator
                                            .ui_state
                                            .set_status(e, StatusLevel::Error)
                                    }
                                }
                            }
                        }
                    });
                });
            });

        // ---------------------------------------------------------------------
        // BARRA DI STATO INFERIORE (28px)
        // ---------------------------------------------------------------------
        TopBottomPanel::bottom("macos_statusbar")
            .exact_height(28.0)
            .frame(egui::Frame {
                inner_margin: Margin::symmetric(18.0, 5.0),
                fill: COLOR_TOOLBAR_BG,
                stroke: Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    if let Some(alert) = &self.coordinator.ui_state.status_message {
                        let col = match alert.level {
                            StatusLevel::Info => COLOR_APPLE_BLUE,
                            StatusLevel::Success => COLOR_APPLE_GREEN,
                            StatusLevel::Warning => COLOR_APPLE_ORANGE,
                            StatusLevel::Error => COLOR_APPLE_RED,
                        };
                        draw_status_dot(
                            ui,
                            Pos2::new(ui.cursor().min.x + 5.0, ui.cursor().min.y + 9.0),
                            col,
                            &alert.text,
                        );
                    } else {
                        draw_status_dot(
                            ui,
                            Pos2::new(ui.cursor().min.x + 5.0, ui.cursor().min.y + 9.0),
                            COLOR_APPLE_GREEN,
                            "Pronto",
                        );
                    }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("v{}", VERSION))
                                .font(FontId::proportional(11.0))
                                .color(COLOR_TEXT_TERTIARY),
                        );
                        ui.separator();
                        let trigger_count = self.coordinator.profile.trigger_allocation.len();
                        ui.label(
                            egui::RichText::new(format!("{} Trigger Attivi", trigger_count))
                                .font(FontId::proportional(11.0))
                                .color(COLOR_TEXT_SECONDARY),
                        );
                        ui.separator();
                        ui.label(
                            egui::RichText::new(&self.coordinator.profile.name)
                                .font(FontId::proportional(11.0))
                                .color(COLOR_TEXT_SECONDARY),
                        );
                    });
                });
            });

        // ---------------------------------------------------------------------
        // CORPO CENTRALE (Contiene SidePanel::right + CentralPanel in Editor)
        // ---------------------------------------------------------------------
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: COLOR_BG_DEEP,
                inner_margin: Margin::same(12.0),
                ..Default::default()
            })
            .show(ctx, |ui| {
                self.studio_ui.show(
                    ui,
                    &mut self.coordinator.ui_state,
                    Some(self.coordinator.platform.as_ref()),
                );
            });

        // Modale di conferma scrittura Flash
        if let Some(preview) = self.flash_modal_preview.clone() {
            egui::Window::new("Conferma Scrittura Hardware")
                .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .collapsible(false)
                .resizable(false)
                .fixed_size(vec2(430.0, 250.0))
                .frame(egui::Frame {
                    inner_margin: Margin::same(20.0),
                    rounding: RADIUS_CARD,
                    fill: COLOR_SURFACE_ELEVATED,
                    stroke: Stroke::new(1.0_f32, COLOR_BORDER_STRONG),
                    ..Default::default()
                })
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("Applicazione Mappatura su Flash USB")
                                .font(FontId::proportional(14.0))
                                .strong()
                                .color(COLOR_TEXT_PRIMARY),
                        );
                        ui.add_space(6.0);
                        inset_grouped_card(ui, Some("Riepilogo"), |ui| {
                            ui.label(format!("Slot Attivi: {}", preview.modified_slots_count));
                            let short_h = if preview.current_hash.len() > 12 {
                                &preview.current_hash[..12]
                            } else {
                                &preview.current_hash
                            };
                            ui.label(format!("SHA-256 YAML: {}", short_h));
                        });
                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            if ui.button("Annulla").clicked() {
                                self.flash_modal_preview = None;
                            }
                            let ok_btn = egui::Button::new(
                                egui::RichText::new("Conferma e Scrivi")
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(COLOR_APPLE_BLUE);
                            if ui.add(ok_btn).clicked() {
                                if let Err(e) = self
                                    .coordinator
                                    .start_hardware_flash(preview.yaml_str, preview.current_hash)
                                {
                                    self.coordinator.ui_state.set_status(e, StatusLevel::Error);
                                }
                                self.flash_modal_preview = None;
                            }
                        });
                    });
                });
        }

        // HUD OSD Flottante Temporizzato (1.5s)
        if let Some(dismiss_time) = self.hud_dismiss_at {
            if Instant::now() < dismiss_time {
                ctx.show_viewport_immediate(
                    egui::ViewportId::from_hash_of("vortexdeck_hud_osd"),
                    egui::ViewportBuilder::default()
                        .with_title("VortexDeck HUD OSD")
                        .with_decorations(false)
                        .with_always_on_top()
                        .with_transparent(true)
                        .with_mouse_passthrough(true)
                        .with_resizable(false)
                        .with_inner_size([720.0, 250.0]),
                    |sub_ctx, _class| {
                        egui::CentralPanel::default()
                            .frame(egui::Frame {
                                fill: rgba(16, 20, 30, 238),
                                rounding: Rounding::same(16.0),
                                stroke: Stroke::new(1.0_f32, COLOR_BORDER_STRONG),
                                inner_margin: Margin::same(14.0),
                                ..Default::default()
                            })
                            .show(sub_ctx, |ui| {
                                paint_specular_rim_light(
                                    ui.painter(),
                                    ui.max_rect(),
                                    Rounding::same(16.0),
                                );
                                self.studio_ui
                                    .hud
                                    .show_content(ui, &mut self.coordinator.ui_state);
                            });
                    },
                );
                ctx.request_repaint_after(Duration::from_millis(30));
            } else {
                self.hud_dismiss_at = None;
            }
        }

        if self.coordinator.ui_state.window_capture.is_some() {
            ctx.request_repaint_after(Duration::from_millis(50));
        }
    }
}

fn print_help() {
    println!("VortexDeck (CH57x Studio) v{}", VERSION);
    println!("Uso: vortexdeck [OPZIONI]\n");
    println!("  --background    Esegue come demone in background");
    println!("  --doctor        Esegue la diagnostica hardware da terminale");
    println!("  --safe-mode     Avvia senza registrare le hotkey globali");
}

fn run_cli_doctor() {
    println!("=== VortexDeck Device Doctor ===");
    let usb = ch57x_hardware::usb_probe::probe_usb_status();
    println!("USB Status: {:?}", usb);
    println!(
        "UsbDk Installed: {}",
        ch57x_hardware::usb_probe::is_usbdk_installed()
    );
}
