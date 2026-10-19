#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
//! VortexDeck (CH57x Studio) — Main Executable Entrypoint
//!
//! Supporta:
//! - Modalità Grafica Desktop Completa (default) con `eframe`/`egui` in stile Apple Cupertino Pro Dark
//! - Modalità Demone Background (`--background`): zero overhead GUI, solo Hotkeys e Dispatcher
//! - Modalità Diagnostica Terminale (`--doctor`): probe USB, CLI e driver UsbDk da terminale
//! - Modalità Provvisoria (`--safe-mode`): avvio senza registrazione scorciatoie globali
//! - Protezione Singola Istanza (`CreateMutexW` con sessione locale su Windows)

mod coordinator;
mod tray;

use std::env;
use std::time::{Duration, Instant};
use tracing::info;

use ch57x_domain::hardware::LayerIndex;
use ch57x_hardware::usb_probe::UsbStatus;
use ch57x_platform::PlatformBackend;
use ch57x_ui::state::{ActiveView, StatusLevel};
use ch57x_ui::theme::*;

use coordinator::{AppCoordinator, CoordinatorCommand, HardwareFlashPreview};
use egui::{vec2, Color32, FontId, Margin, Pos2, Rounding, Stroke, TopBottomPanel};
use tray::{TrayAction, TrayHandle};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();

    // Collega la console padre se lanciato con argomenti CLI su Windows
    #[cfg(target_os = "windows")]
    if args.len() > 1 {
        unsafe {
            let _ = windows::Win32::System::Console::AttachConsole(
                windows::Win32::System::Console::ATTACH_PARENT_PROCESS,
            );
        }
    }

    // 1. Parsing argomenti CLI
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

    // 2. Controllo Singola Istanza su Windows (Bug 9)
    #[cfg(target_os = "windows")]
    let _mutex_guard = {
        use windows::core::w;
        use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
        use windows::Win32::System::Threading::CreateMutexW;

        let handle =
            unsafe { CreateMutexW(None, true, w!("Local\\VortexDeck_SingleInstance_Mutex")) };
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            eprintln!("Un'altra istanza di VortexDeck è già in esecuzione.");
            println!(
                "Suggerimento: Controlla la System Tray (icona vicino all'orologio di Windows)."
            );
            return Ok(());
        }
        handle
    };

    info!(
        "Avvio VortexDeck v{} (safe_mode: {}, background: {})",
        VERSION, is_safe_mode, is_background
    );

    // Inizializza Coordinatore centrale
    let mut coordinator = AppCoordinator::new(is_safe_mode)?;
    let tray = TrayHandle::new();

    // 3. Modalità Background (Headless)
    if is_background {
        info!("Modalità --background attiva: avvio demone runtime headless.");
        println!("VortexDeck attivo in background. Premi Ctrl+C o usa la System Tray per uscire.");

        loop {
            // Processa eventi dalla System Tray
            while let Ok(action) = tray.receiver.try_recv() {
                match action {
                    TrayAction::ShowWindow => {
                        info!("Apertura interfaccia grafica richiesta da System Tray mentre in background (Bug 11).");
                        #[cfg(target_os = "windows")]
                        {
                            if let Ok(handle) = _mutex_guard {
                                unsafe {
                                    let _ = windows::Win32::Foundation::CloseHandle(handle);
                                };
                            }
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
                    TrayAction::ShowHud => {
                        println!("Richiesta HUD OSD in background.");
                    }
                    TrayAction::Quit => {
                        info!("Chiusura demone background richiesta da System Tray.");
                        return Ok(());
                    }
                }
            }

            coordinator.process_pending_commands();
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    // 4. Modalità Grafica Completa (eframe / egui Cupertino Pro Dark)
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("VortexDeck")
            .with_inner_size([1240.0, 780.0])
            .with_min_inner_size([1000.0, 620.0])
            .with_transparent(true)
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
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
    )?;

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
        // Intercetta la chiusura della finestra con la 'X' per nascondere in Tray (Bug 6)
        if ctx.input(|i| i.viewport().close_requested()) && !self.is_quitting {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }

        // Processa azioni dalla System Tray
        while let Ok(action) = self.tray.receiver.try_recv() {
            match action {
                TrayAction::ShowWindow => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                }
                TrayAction::TogglePause => {
                    let _ = self
                        .coordinator
                        .command_sender
                        .send(CoordinatorCommand::TogglePause);
                }
                TrayAction::ShowHud => {
                    self.hud_dismiss_at = Some(Instant::now() + Duration::from_millis(1500));
                    ctx.request_repaint();
                }
                TrayAction::Quit => {
                    self.is_quitting = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }

        // Processa comandi del coordinatore
        self.coordinator.process_pending_commands();

        // ---------------------------------------------------------------------
        // BARRA SUPERIORE UNIFICATA STILE macOS TOOLBAR (54px) — (Bug 3 e Parte B)
        // ---------------------------------------------------------------------
        TopBottomPanel::top("macos_toolbar")
            .exact_height(54.0)
            .frame(egui::Frame {
                inner_margin: Margin::symmetric(16.0, 10.0),
                fill: COLOR_TOOLBAR_BG,
                stroke: Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // --- SINISTRA: Titolo Elegante + Badge Hardware USB + Dirty Dot ---
                    ui.label(
                        egui::RichText::new("VortexDeck")
                            .font(FontId::proportional(15.0))
                            .strong()
                            .color(COLOR_TEXT_PRIMARY),
                    );

                    ui.add_space(8.0);

                    // Badge Hardware USB (interrogato con cooldown di 2s dal coordinator - Bug 3)
                    let usb_status = self.coordinator.get_usb_status();
                    match usb_status {
                        UsbStatus::Ready { vid, pid } => {
                            let text = format!("● USB Pronta {:04X}:{:04X}", vid, pid);
                            draw_pill_badge(ui, &text, Color32::from_rgb(20, 42, 28), COLOR_APPLE_GREEN);
                        }
                        UsbStatus::DetectedAccessDenied => {
                            draw_pill_badge(ui, "● USB Accesso Negato", Color32::from_rgb(45, 32, 16), COLOR_APPLE_ORANGE);
                        }
                        UsbStatus::Disconnected => {
                            draw_pill_badge(ui, "● Solo Runtime Host", Color32::from_rgb(34, 34, 38), COLOR_TEXT_SECONDARY);
                        }
                    }

                    if self.coordinator.ui_state.is_dirty {
                        ui.add_space(4.0);
                        draw_pill_badge(ui, "● Modifiche non salvate", Color32::from_rgb(45, 32, 16), COLOR_APPLE_ORANGE);
                    }

                    // --- CENTRO: Segmented Controls Stile macOS ---
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.add_space(24.0);

                        // 1. Selettore Layer [ Layer 1 | Layer 2 | Layer 3 ]
                        let mut current_layer = self.coordinator.ui_state.editing_layer.0;
                        let layer_opts = [(0u8, "Layer 1"), (1u8, "Layer 2"), (2u8, "Layer 3")];
                        if segmented_control(ui, &mut current_layer, &layer_opts) {
                            self.coordinator.ui_state.set_editing_layer(LayerIndex(current_layer));
                        }

                        ui.add_space(14.0);

                        // 2. Selettore Vista [ Configuratore | Diagnostica | HUD Preview ]
                        let mut current_view = self.coordinator.ui_state.active_view;
                        let view_opts = [
                            (ActiveView::Editor, "Configuratore"),
                            (ActiveView::Doctor, "Diagnostica"),
                            (ActiveView::Hud, "HUD Preview"),
                        ];
                        if segmented_control(ui, &mut current_view, &view_opts) {
                            self.coordinator.ui_state.active_view = current_view;
                        }
                    });

                    // --- DESTRA: Azioni minimali e Pulsante Call-To-Action Blu Apple ---
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // CTA Blu Scrivi su Dispositivo (Bug 4)
                        let flash_btn = egui::Button::new(
                            egui::RichText::new("Scrivi su Dispositivo…")
                                .font(FontId::proportional(12.5))
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(COLOR_APPLE_BLUE)
                        .stroke(Stroke::NONE)
                        .rounding(Rounding::same(8.0))
                        .min_size(vec2(0.0, 30.0));

                        let is_flashing = self.coordinator.is_flashing();
                        if ui.add_enabled(!is_flashing, flash_btn)
                            .on_hover_text("Genera anteprima diff e apre il modulo di conferma prima di scrivere nella flash USB.")
                            .clicked()
                        {
                            match self.coordinator.generate_flash_preview() {
                                Ok(preview) => {
                                    self.flash_modal_preview = Some(preview);
                                }
                                Err(e) => {
                                    self.coordinator.ui_state.set_status(e, StatusLevel::Error);
                                }
                            }
                        }

                        ui.add_space(6.0);

                        // Pulsante Salva
                        if ui.button(egui::RichText::new("Salva").color(COLOR_TEXT_PRIMARY))
                            .on_hover_text("Salva il profilo su disco e sincronizza il motore host.")
                            .clicked()
                        {
                            self.coordinator.save_active_profile();
                        }

                        // Pulsante Valida
                        if ui.button(egui::RichText::new("Valida").color(COLOR_TEXT_PRIMARY))
                            .on_hover_text("Verifica la configurazione con ch57x-keyboard-tool.")
                            .clicked()
                        {
                            self.coordinator.validate_configuration();
                        }

                        // Pulsante Pausa / Riprendi
                        let is_paused = self.coordinator.dispatcher.is_paused();
                        let (pause_lbl, pause_col) = if is_paused {
                            ("Riprendi", COLOR_APPLE_ORANGE)
                        } else {
                            ("Pausa", COLOR_TEXT_SECONDARY)
                        };
                        if ui.button(egui::RichText::new(pause_lbl).color(pause_col))
                            .on_hover_text("Sospende o riattiva l'esecuzione di tutte le automazioni host.")
                            .clicked()
                        {
                            let _ = self.coordinator.command_sender.send(CoordinatorCommand::TogglePause);
                        }
                    });
                });
            });

        // ---------------------------------------------------------------------
        // BARRA INFERIORE DI STATO STILE macOS STATUSBAR (28px)
        // ---------------------------------------------------------------------
        TopBottomPanel::bottom("macos_statusbar")
            .exact_height(28.0)
            .frame(egui::Frame {
                inner_margin: Margin::symmetric(16.0, 5.0),
                fill: COLOR_TOOLBAR_BG,
                stroke: Stroke::new(1.0_f32, COLOR_BORDER_SUBTLE),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Alert attivo o Pronto
                    if let Some(alert) = &self.coordinator.ui_state.status_message {
                        let col = match alert.level {
                            StatusLevel::Info => COLOR_APPLE_BLUE,
                            StatusLevel::Success => COLOR_APPLE_GREEN,
                            StatusLevel::Warning => COLOR_APPLE_ORANGE,
                            StatusLevel::Error => COLOR_APPLE_RED,
                        };
                        draw_status_dot(
                            ui,
                            Pos2::new(ui.cursor().min.x + 4.0, ui.cursor().min.y + 9.0),
                            col,
                            &alert.text,
                        );
                    } else {
                        draw_status_dot(
                            ui,
                            Pos2::new(ui.cursor().min.x + 4.0, ui.cursor().min.y + 9.0),
                            COLOR_TEXT_SECONDARY,
                            "Pronto",
                        );
                    }

                    // Destra: Info Profilo, Trigger Allocati e Versione
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
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
        // PANNELLO CENTRALE PRINCIPALE (StudioUi con Deep Space Canvas)
        // ---------------------------------------------------------------------
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: Color32::from_rgb(11, 15, 25),
                inner_margin: Margin::same(16.0),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ch57x_ui::paint_deep_space_canvas(ui.painter(), ui.max_rect());
                self.studio_ui.show(
                    ui,
                    &mut self.coordinator.ui_state,
                    Some(self.coordinator.platform.as_ref()),
                );
            });

        // ---------------------------------------------------------------------
        // MODALE DI CONFERMA SCRITTURA FLASH STILE macOS SHEET (Bug 4)
        // ---------------------------------------------------------------------
        if let Some(preview) = self.flash_modal_preview.clone() {
            egui::Window::new("Conferma Scrittura Hardware")
                .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .collapsible(false)
                .resizable(false)
                .fixed_size(vec2(440.0, 260.0))
                .frame(egui::Frame {
                    inner_margin: Margin::same(20.0),
                    rounding: RADIUS_CARD,
                    fill: COLOR_SURFACE_CARD,
                    stroke: Stroke::new(1.0_f32, COLOR_BORDER_STRONG),
                    shadow: egui::epaint::Shadow {
                        blur: 28.0,
                        color: Color32::from_black_alpha(180),
                        ..Default::default()
                    },
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
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(
                                "Stai per inviare la configurazione fisica alla memoria flash permanente del Macropad.",
                            )
                            .font(FontId::proportional(12.0))
                            .color(COLOR_TEXT_SECONDARY),
                        );
                        ui.add_space(10.0);

                        inset_grouped_card(ui, Some("Dettagli Operazione"), |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Slot Attivi Configurati:").color(COLOR_TEXT_SECONDARY));
                                ui.label(egui::RichText::new(format!("{}", preview.modified_slots_count)).strong().color(COLOR_TEXT_PRIMARY));
                            });
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Hash Configurazione YAML:").color(COLOR_TEXT_SECONDARY));
                                let short_hash = if preview.current_hash.len() > 12 { &preview.current_hash[..12] } else { &preview.current_hash };
                                ui.label(egui::RichText::new(short_hash).monospace().color(COLOR_APPLE_BLUE));
                            });
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Scrittura Flash Necessaria:").color(COLOR_TEXT_SECONDARY));
                                if preview.requires_hardware_flash {
                                    draw_pill_badge(ui, "SI (Modifiche Hardware)", Color32::from_rgb(45, 32, 16), COLOR_APPLE_ORANGE);
                                } else {
                                    draw_pill_badge(ui, "NO (Già Sincronizzato)", Color32::from_rgb(20, 42, 28), COLOR_APPLE_GREEN);
                                }
                            });
                        });

                        ui.add_space(12.0);

                        ui.horizontal(|ui| {
                            draw_status_dot(ui, Pos2::new(ui.cursor().min.x + 4.0, ui.cursor().min.y + 8.0), COLOR_APPLE_ORANGE, "");
                            ui.add_space(14.0);
                            ui.label(
                                egui::RichText::new("Non scollegare il cavo USB durante il processo.")
                                    .font(FontId::proportional(11.0))
                                    .color(COLOR_APPLE_ORANGE),
                            );
                        });

                        ui.add_space(14.0);

                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let confirm_btn = egui::Button::new(
                                    egui::RichText::new("Scrivi su Dispositivo")
                                        .font(FontId::proportional(12.5))
                                        .strong()
                                        .color(Color32::WHITE),
                                )
                                .fill(COLOR_APPLE_BLUE)
                                .rounding(Rounding::same(8.0))
                                .min_size(vec2(140.0, 32.0));

                                if ui.add(confirm_btn).clicked() {
                                    if let Err(e) = self.coordinator.start_hardware_flash(preview.yaml_str.clone(), preview.current_hash.clone()) {
                                        self.coordinator.ui_state.set_status(e, StatusLevel::Error);
                                    }
                                    self.flash_modal_preview = None;
                                }

                                ui.add_space(8.0);

                                let cancel_btn = egui::Button::new(
                                    egui::RichText::new("Annulla").color(COLOR_TEXT_PRIMARY),
                                )
                                .min_size(vec2(80.0, 32.0));

                                if ui.add(cancel_btn).clicked() {
                                    self.flash_modal_preview = None;
                                }
                            });
                        });
                    });
                });
        }

        // Indicatore non bloccante durante il flashing hardware
        if self.coordinator.is_flashing() {
            egui::Window::new("Scrittura Hardware")
                .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .collapsible(false)
                .resizable(false)
                .title_bar(false)
                .fixed_size(vec2(320.0, 90.0))
                .frame(egui::Frame {
                    inner_margin: Margin::same(20.0),
                    rounding: RADIUS_CARD,
                    fill: COLOR_SURFACE_CARD,
                    stroke: Stroke::new(1.0_f32, COLOR_APPLE_BLUE),
                    ..Default::default()
                })
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add(egui::Spinner::new().size(24.0));
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new("Scrittura memoria flash USB in corso...")
                                .font(FontId::proportional(12.5))
                                .color(COLOR_TEXT_PRIMARY)
                                .strong(),
                        );
                    });
                });
            ctx.request_repaint_after(Duration::from_millis(50));
        }

        // ---------------------------------------------------------------------
        // HUD OSD FLOTTANTE SECONDARIO (Multi-Viewport indipendente anche in Tray)
        // ---------------------------------------------------------------------
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
                        .with_inner_size([720.0, 240.0]),
                    |sub_ctx, _class| {
                        egui::CentralPanel::default()
                            .frame(egui::Frame {
                                fill: Color32::from_rgba_premultiplied(16, 20, 30, 235),
                                rounding: Rounding::same(16.0),
                                stroke: Stroke::new(
                                    1.0_f32,
                                    Color32::from_rgba_premultiplied(255, 255, 255, 30),
                                ),
                                inner_margin: Margin::same(14.0),
                                ..Default::default()
                            })
                            .show(sub_ctx, |ui| {
                                ch57x_ui::paint_specular_rim_light(
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

        // Richiedi repaint se c'è un countdown di cattura finestra attivo
        if self.coordinator.ui_state.window_capture.is_some() {
            ctx.request_repaint_after(Duration::from_millis(50));
        }
    }
}

fn print_help() {
    println!(
        r#"VortexDeck (CH57x Studio) — Visual Configurator & Automation Runtime

USO:
    vortexdeck.exe [OPZIONI]

OPZIONI:
    --background    Esegue VortexDeck come demone di sfondo (senza GUI)
    --doctor        Esegue la diagnostica hardware e dell'ambiente da riga di comando
    --safe-mode     Avvia senza registrare le hotkey globali F13..F24 (modalità provvisoria)
    -h, --help      Mostra questo messaggio di aiuto
    -v, --version   Mostra la versione del programma
"#
    );
}

fn run_cli_doctor() {
    println!("=== VortexDeck Device Doctor CLI ===");
    println!("Versione: {}", VERSION);
    println!();

    // 1. Probe USB
    print!("[1/3] Rilevamento Macropad USB... ");
    let status = ch57x_hardware::usb_probe::probe_usb_status();
    match status {
        ch57x_hardware::usb_probe::UsbStatus::Ready { vid, pid } => {
            println!("OK (Trovato {:04X}:{:04X})", vid, pid);
        }
        ch57x_hardware::usb_probe::UsbStatus::DetectedAccessDenied => {
            println!("ATTENZIONE (Dispositivo trovato ma UsbDk/permessi mancanti)");
        }
        ch57x_hardware::usb_probe::UsbStatus::Disconnected => {
            println!("NON COLLEGATO (Collega il cavo USB-C per la scrittura)");
        }
    }

    // 2. Probe CLI upstream
    print!("[2/3] Verifica ch57x-keyboard-tool... ");
    match ch57x_hardware::cli_adapter::Ch57xCli::auto_detect() {
        Ok(cli) => match cli.show_keys() {
            Ok(keys) => {
                let count = keys.lines().count();
                println!("OK (Binario funzionante, {} chiavi lette)", count);
            }
            Err(e) => {
                println!("ERRORE ({})", e);
            }
        },
        Err(e) => {
            println!("NON TROVATO ({})", e);
        }
    }

    // 3. Capability Matrix
    print!("[3/3] Verifica Funzionalità Sistema Operativo... ");
    let platform = ch57x_platform::get_platform();
    let caps = platform.capability_matrix();
    println!("OK");
    println!("      Window Placement: {:?}", caps.window_placement);
    println!("      Global Hotkeys:   {:?}", caps.global_hotkeys);
    println!("      Input Injection:  {:?}", caps.input_injection);
    println!("      System Tray:      {:?}", caps.system_tray);

    println!();
    println!("Diagnostica completata.");
}
