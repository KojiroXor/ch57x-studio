//! VortexDeck (CH57x Studio) — Main Executable Entrypoint
//!
//! Supporta:
//! - Modalità Grafica Desktop Completa (default) con `eframe`/`egui`
//! - Modalità Demone Background (`--background`): zero overhead GUI, solo Hotkeys e Dispatcher
//! - Modalità Diagnostica Terminale (`--doctor`): probe USB, CLI e driver UsbDk da terminale
//! - Modalità Provvisoria (`--safe-mode`): avvio senza registrazione scorciatoie globali
//! - Protezione Singola Istanza (`CreateMutexW` su Windows)

mod coordinator;
mod tray;

use std::env;
use tracing::info;
use ch57x_platform::traits::PlatformBackend;

use coordinator::{AppCoordinator, CoordinatorCommand};
use tray::{TrayAction, TrayHandle};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();

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

    // 2. Controllo Singola Istanza su Windows
    #[cfg(target_os = "windows")]
    {
        use windows::core::w;
        use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
        use windows::Win32::System::Threading::CreateMutexW;

        unsafe {
            let _mutex = CreateMutexW(None, true, w!("Global\\VortexDeck_SingleInstance_Mutex"));
            if GetLastError() == ERROR_ALREADY_EXISTS {
                eprintln!("Un'altra istanza di VortexDeck è già in esecuzione.");
                println!("Suggerimento: Controlla la System Tray (icona vicino all'orologio di Windows).");
                return Ok(());
            }
        }
    }

    info!("Avvio VortexDeck v{} (safe_mode: {}, background: {})", VERSION, is_safe_mode, is_background);

    // Inizializza Coordinatore centrale
    let mut coordinator = AppCoordinator::new(is_safe_mode)?;
    let tray = TrayHandle::new();

    // 3. Modalità Background (Zero GUI)
    if is_background {
        info!("Modalità --background attiva: avvio demone runtime headless.");
        println!("VortexDeck attivo in background. Premi Ctrl+C o usa la System Tray per uscire.");

        loop {
            // Processa eventi dalla System Tray
            while let Ok(action) = tray.receiver.try_recv() {
                match action {
                    TrayAction::ShowWindow => {
                        println!("Richiesta apertura finestra non supportata in modalità headless.");
                    }
                    TrayAction::TogglePause => {
                        coordinator.command_sender.send(CoordinatorCommand::TogglePause)?;
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

    // 4. Modalità Grafica Completa (eframe / egui)
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("VortexDeck — CH57x Studio")
            .with_inner_size([1240.0, 780.0])
            .with_min_inner_size([960.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "VortexDeck",
        native_options,
        Box::new(|_cc| {
            Ok(Box::new(VortexDeckApp {
                coordinator,
                studio_ui: ch57x_ui::StudioUi::new(),
                tray,
            }))
        }),
    )?;

    Ok(())
}

struct VortexDeckApp {
    coordinator: AppCoordinator,
    studio_ui: ch57x_ui::StudioUi,
    tray: TrayHandle,
}

impl eframe::App for VortexDeckApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Processa comandi interni e messaggi dal tray
        while let Ok(action) = self.tray.receiver.try_recv() {
            match action {
                TrayAction::ShowWindow => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                }
                TrayAction::TogglePause => {
                    let _ = self.coordinator.command_sender.send(CoordinatorCommand::TogglePause);
                }
                TrayAction::ShowHud => {
                    self.coordinator.ui_state.active_view = ch57x_ui::state::ActiveView::Hud;
                }
                TrayAction::Quit => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }

        self.coordinator.process_pending_commands();

        // Barra degli strumenti superiore
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("⚡ VORTEXDECK")
                        .font(egui::FontId::proportional(16.0))
                        .strong()
                        .color(egui::Color32::from_rgb(56, 189, 248)),
                );

                ui.separator();

                // Pulsante Salva Profilo
                if ui.button("💾 Salva").on_hover_text("Salva il profilo su disco e aggiorna il motore runtime host").clicked() {
                    self.coordinator.save_active_profile();
                }

                // Pulsante Valida YAML
                if ui.button("🔍 Valida").on_hover_text("Verifica la configurazione con ch57x-keyboard-tool").clicked() {
                    self.coordinator.validate_configuration();
                }

                // Pulsante Flashing Hardware
                let flash_btn = ui.button(
                    egui::RichText::new("⚡ Flasha su Tastiera")
                        .color(egui::Color32::from_rgb(34, 197, 94))
                        .strong(),
                ).on_hover_text("Scrive la mappatura dei tasti direttamente nella memoria flash del Macropad USB");

                if flash_btn.clicked() {
                    self.coordinator.upload_configuration_to_device();
                }

                ui.separator();

                // Toggle Pausa Emergenza
                let is_paused = self.coordinator.dispatcher.is_paused();
                let pause_text = if is_paused { "▶️ Riprendi" } else { "⏸️ Pausa" };
                let pause_color = if is_paused { egui::Color32::from_rgb(234, 179, 8) } else { egui::Color32::LIGHT_GRAY };

                if ui.button(egui::RichText::new(pause_text).color(pause_color)).clicked() {
                    let _ = self.coordinator.command_sender.send(CoordinatorCommand::TogglePause);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let vid_pid = ch57x_hardware::usb_probe::probe_usb_status();
                    match vid_pid {
                        ch57x_hardware::usb_probe::UsbStatus::Ready { vid, pid } => {
                            ui.label(
                                egui::RichText::new(format!("🟢 USB Pronta ({:04X}:{:04X})", vid, pid))
                                    .color(egui::Color32::from_rgb(34, 197, 94)),
                            );
                        }
                        ch57x_hardware::usb_probe::UsbStatus::DetectedAccessDenied => {
                            ui.label(
                                egui::RichText::new("🟡 USB Rilevata (Accesso Negato)")
                                    .color(egui::Color32::from_rgb(234, 179, 8)),
                            );
                        }
                        ch57x_hardware::usb_probe::UsbStatus::Disconnected => {
                            ui.label(
                                egui::RichText::new("⚪ USB Non Connessa (Solo Host)")
                                    .color(egui::Color32::from_rgb(161, 161, 170)),
                            );
                        }
                    }
                });
            });
        });

        // Contenuto principale con StudioUi
        egui::CentralPanel::default().show(ctx, |ui| {
            self.studio_ui.show(
                ui,
                &mut self.coordinator.ui_state,
                Some(self.coordinator.platform.as_ref()),
            );
        });

        // Richiedi repaint se c'è un countdown di cattura finestra attivo
        if self.coordinator.ui_state.window_capture.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
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
