# AUDIT_SOURCE.md — VortexDeck (CH57x Studio) Audit Report

Data: 2026-10-19
Workspace: VortexDeck (CH57x Studio)

================================================================================
1. OUTPUT DI cargo test --workspace
================================================================================
```text
Finished `test` profile [unoptimized + debuginfo] target(s) in 34.17s
     Running unittests src\lib.rs (target\debug\deps\ch57x_compiler-7cc8330dc0818199.exe)

running 10 tests
test allocator::tests::test_collision_detection ... ok
test allocator::tests::test_freed_slots_are_cleaned_up ... ok
test allocator::tests::test_preserve_existing_allocations ... ok
test diff::tests::test_deployment_decision ... ok
test yaml_gen::tests::test_encoder_order_swapped ... ok
test diff::tests::test_requires_hardware_flash_logic ... ok
test diff::tests::test_yaml_hash_deterministic ... ok
test yaml_gen::tests::test_generate_yaml_with_all_action_types ... ok
test yaml_gen::tests::test_generate_yaml_default_profile ... ok
test allocator::tests::test_allocate_all_54_slots ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\validation_test.rs (target\debug\deps\validation_test-289c04ccb1f6016c.exe)

running 1 test
test test_full_54_slots_validation_against_upstream_binary ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s

     Running unittests src\lib.rs (target\debug\deps\ch57x_domain-30d763557b36212d.exe)

running 11 tests
test hardware::tests::test_slots_distribution_per_layer ... ok
test triggers::tests::test_exactly_60_candidate_triggers ... ok
test validation::tests::test_corrupted_slot_id_fails_validation ... ok
test validation::tests::test_default_profile_is_valid ... ok
test hardware::tests::test_exactly_54_unique_slots ... ok
test triggers::tests::test_trigger_banks ... ok
test validation::tests::test_context_dispatch_validation ... ok
test validation::tests::test_duplicate_trigger_allocation_fails ... ok
test validation::tests::test_invalid_window_coordinates_fails ... ok
test validation::tests::test_nonexistent_action_reference_fails ... ok
test validation::tests::test_missing_slot_fails_validation ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running unittests src\lib.rs (target\debug\deps\ch57x_hardware-a5b8d85b69b7c178.exe)

running 6 tests
test cli_adapter::tests::test_cli_binary_not_found ... ok
test usb_probe::tests::test_probe_runs_without_panic ... ok
test usb_probe::tests::test_usb_status_methods ... ok
test cli_adapter::tests::test_validate_invalid_yaml ... ok
test cli_adapter::tests::test_show_keys ... ok
test cli_adapter::tests::test_validate_valid_yaml ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s

     Running unittests src\lib.rs (target\debug\deps\ch57x_persistence-136e011fcfe182ad.exe)

running 9 tests
test journal::tests::test_invalid_state_transition_is_blocked ... ok
test trust::tests::test_compute_bytes_sha256_known_vector ... ok
test storage::tests::test_saving_invalid_profile_fails ... ok
test storage::tests::test_atomic_write_and_load_profile_roundtrip ... ok
test journal::tests::test_journal_host_only_deployment_lifecycle ... ok
test journal::tests::test_journal_full_hardware_deployment_lifecycle ... ok
test journal::tests::test_crash_recovery_during_hardware_flash ... ok
test storage::tests::test_atomic_overwrite_preserves_integrity ... ok
test trust::tests::test_compute_file_sha256_and_trust_verification ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s

     Running unittests src\lib.rs (target\debug\deps\ch57x_platform-21496c33b826b170.exe)

running 11 tests
test tests::test_capability_matrices ... ok
test tests::test_win32_overlay_invalid_handle ... ok
test tests::test_win32_autostart_query ... ok
test tests::test_win32_monitors_enumeration ... ok
test tests::test_platform_backend_instantiation ... ok
test tests::test_win32_launch_or_place_window_not_found ... ok
test tests::test_win32_list_visible_windows ... ok
test tests::test_win32_token_to_virtual_key ... ok
test tests::test_win32_trigger_conversion_all_banks ... ok
test tests::test_win32_hotkey_registration_lifecycle ... ok
test tests::test_win32_uipi_check ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running unittests src\lib.rs (target\debug\deps\ch57x_runtime-194da7ebf278ca04.exe)

running 9 tests
test context_actor::tests::test_context_rule_matching_priority_and_fallback ... ok
test context_actor::tests::test_context_actor_window_manager_error_fallback ... ok
test dispatcher::tests::test_dispatcher_event_routing_and_pause ... ok
test window_actor::tests::test_coordinate_calculations ... ok
test macro_actor::tests::test_focus_guard_aborts_on_window_change ... ok
test macro_actor::tests::test_macro_actor_concurrency_policies ... ok
test window_actor::tests::test_window_actor_retries_and_succeeds ... ok
test workspace_actor::tests::test_workspace_restore_report_partial_and_full ... ok
test macro_actor::tests::test_macro_steps_and_clipboard_backup_restore ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

     Running unittests src\main.rs (target\debug\deps\vortexdeck-f3c7d10111d55e21.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src\lib.rs (target\debug\deps\ch57x_ui-be9d9fa49fa80034.exe)

running 16 tests
test hud_window::tests::test_hud_window_instantiation ... ok
test doctor_panel::tests::test_traffic_light_colors ... ok
test pad_canvas::tests::test_pad_canvas_default ... ok
test state::tests::test_badge_resolution ... ok
test state::tests::test_layer_switch_and_slot_selection ... ok
test state::tests::test_hex_conversion ... ok
test doctor_panel::tests::test_doctor_panel_instantiation ... ok
test inspector::tests::test_inspector_instantiation ... ok
test pad_canvas::tests::test_badge_colors ... ok
test state::tests::test_calibration_state ... ok
test state::tests::test_dirty_tracking ... ok
test state::tests::test_status_alert ... ok
test state::tests::test_select_slot_by_index ... ok
test state::tests::test_ui_state_initialization ... ok
test state::tests::test_window_capture_countdown ... ok
test tests::test_studio_ui_instantiation ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

   Doc-tests ch57x_compiler

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ch57x_domain

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ch57x_hardware

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ch57x_persistence

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ch57x_platform

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ch57x_runtime

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ch57x_ui

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
`

================================================================================
2. FILE: crates/app/src/main.rs
================================================================================
`
ust
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
`

================================================================================
3. FILE: crates/app/src/coordinator.rs
================================================================================
`
ust
//! Coordinator bridge between UI and Runtime engine
//!
//! Coordina:
//! - Il profilo attivo e lo stato UI (`UiState`)
//! - Il salvataggio atomico e ripristino da disco (`ch57x-persistence`)
//! - L'allocazione trigger deterministica e serializzazione YAML (`ch57x-compiler`)
//! - L'interfaccia con `ch57x-keyboard-tool` per validazione e flashing sicuro (`ch57x-hardware`)
//! - Il dispatching in tempo reale delle automazioni host (`ch57x-runtime`)
//! - La registrazione delle hotkey globali F13..F24 (`ch57x-platform`)

use std::path::PathBuf;
use std::sync::Arc;
use crossbeam_channel::{unbounded, Receiver, Sender};
use tracing::{error, info, warn};

use ch57x_compiler::allocator::allocate_triggers;
use ch57x_compiler::diff::DeploymentDecision;
use ch57x_compiler::yaml_gen::generate_yaml;
use ch57x_domain::profile::Profile;
use ch57x_domain::triggers::PhantomTrigger;
use ch57x_domain::validation::validate_profile;
use ch57x_hardware::cli_adapter::Ch57xCli;
use ch57x_hardware::usb_probe::probe_usb_status;
use ch57x_persistence::journal::DeploymentJournal;
use ch57x_persistence::storage::{get_app_data_dir, get_profiles_dir, load_profile_from_path, save_profile_to_path};
use ch57x_persistence::trust::TrustStore;
use ch57x_platform::get_platform;
use ch57x_platform::traits::PlatformBackend;
use ch57x_runtime::dispatcher::RuntimeDispatcher;
use ch57x_ui::state::{StatusLevel, UiState};

#[allow(dead_code)]
pub enum CoordinatorCommand {
    TriggerEvent(PhantomTrigger),
    SaveProfile,
    ValidateYaml,
    UploadToDevice,
    TogglePause,
    SwitchLayer(u8),
    ShowHud,
    Quit,
}

pub struct AppCoordinator {
    pub profile: Profile,
    pub ui_state: UiState,
    pub base_dir: PathBuf,
    pub journal: DeploymentJournal,
    #[allow(dead_code)]
    pub trust_store: TrustStore,
    pub cli: Ch57xCli,
    pub platform: Arc<dyn PlatformBackend>,
    pub dispatcher: Arc<RuntimeDispatcher>,
    pub command_sender: Sender<CoordinatorCommand>,
    pub command_receiver: Receiver<CoordinatorCommand>,
    pub profile_path: PathBuf,
}

impl AppCoordinator {
    /// Inizializza il coordinatore caricando il profilo attivo o creandone uno predefinito
    pub fn new(safe_mode: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let base_dir = get_app_data_dir();
        let journal = DeploymentJournal::load_or_recover(&base_dir).unwrap_or_default();
        let trust_store = TrustStore::open_or_default(&base_dir)?;

        let profiles_dir = get_profiles_dir(Some(&base_dir));
        let profile_path = profiles_dir.join("default.json");

        let profile = if profile_path.exists() {
            match load_profile_from_path(&profile_path) {
                Ok(p) => {
                    info!("Caricato profilo esistente da {:?}", profile_path);
                    p
                }
                Err(e) => {
                    warn!("Errore caricamento profilo ({:?}), uso profilo predefinito", e);
                    Profile::default()
                }
            }
        } else {
            info!("Nessun profilo trovato, inizializzazione profilo default a 54 slot");
            let mut def = Profile::default();
            let _ = allocate_triggers(&mut def);
            let _ = save_profile_to_path(&def, &profile_path);
            def
        };

        let cli = Ch57xCli::auto_detect()
            .unwrap_or_else(|_| Ch57xCli::new("bin/ch57x-keyboard-tool.exe"))
            .with_vendor_id(0x1189)
            .with_product_id(0x8890);

        let platform_inst = get_platform();
        let platform: Arc<dyn PlatformBackend> = Arc::new(platform_inst);

        // RuntimeDispatcher requires Arc<dyn WindowManagerBackend> and Arc<dyn InputInjectorBackend>
        // Let's create actors or dispatch directly
        let dispatcher = Arc::new(RuntimeDispatcher::new(
            profile.clone(),
            Arc::new(get_platform()),
            Arc::new(get_platform()),
        ));

        let (command_sender, command_receiver) = unbounded();

        let mut ui_state = UiState {
            profile: profile.clone(),
            ..Default::default()
        };

        // Se non siamo in safe mode, registra le hotkey globali per tutti i trigger allocati
        if !safe_mode {
            let triggers: Vec<PhantomTrigger> = profile.trigger_allocation.values().copied().collect();
            if !triggers.is_empty() {
                if let Err(e) = platform.hotkeys().register_triggers(&triggers) {
                    error!("Errore nella registrazione iniziale delle hotkey: {:?}", e);
                    ui_state.set_status(
                        format!("Conflitto Hotkey: {:?}", e),
                        StatusLevel::Warning,
                    );
                } else {
                    info!("Registrate {} hotkey globali F13..F24", triggers.len());
                }
            }
        } else {
            info!("Safe mode attiva: registrazione hotkey globali disabilitata.");
            ui_state.set_status("Avviato in Safe Mode (Hotkey disabilitate)", StatusLevel::Info);
        }

        Ok(Self {
            profile,
            ui_state,
            base_dir,
            journal,
            trust_store,
            cli,
            platform,
            dispatcher,
            command_sender,
            command_receiver,
            profile_path,
        })
    }

    /// Processa tutti i comandi in coda non-bloccanti
    pub fn process_pending_commands(&mut self) {
        while let Ok(cmd) = self.command_receiver.try_recv() {
            match cmd {
                CoordinatorCommand::TriggerEvent(trigger) => {
                    info!("Ricevuto trigger hardware/virtuale: {}", trigger.yaml_token());
                    let _ = self.dispatcher.trigger_received(trigger);
                }
                CoordinatorCommand::SaveProfile => {
                    self.save_active_profile();
                }
                CoordinatorCommand::ValidateYaml => {
                    self.validate_configuration();
                }
                CoordinatorCommand::UploadToDevice => {
                    self.upload_configuration_to_device();
                }
                CoordinatorCommand::TogglePause => {
                    let was_paused = self.dispatcher.is_paused();
                    self.dispatcher.set_paused(!was_paused);
                    if !was_paused {
                        self.ui_state.set_status("Automazioni in PAUSA", StatusLevel::Warning);
                    } else {
                        self.ui_state.set_status("Automazioni ATTIVE", StatusLevel::Success);
                    }
                }
                CoordinatorCommand::SwitchLayer(layer) => {
                    if let Some(l) = ch57x_domain::hardware::LayerIndex::new(layer) {
                        self.ui_state.set_editing_layer(l);
                    }
                }
                CoordinatorCommand::ShowHud => {
                    self.ui_state.active_view = ch57x_ui::state::ActiveView::Hud;
                }
                CoordinatorCommand::Quit => {
                    std::process::exit(0);
                }
            }
        }
    }

    /// Salva atomicamente il profilo su disco e aggiorna il dispatcher
    pub fn save_active_profile(&mut self) -> bool {
        self.profile = self.ui_state.profile.clone();

        // Verifica semantica prima del salvataggio
        if let Err(e) = validate_profile(&self.profile) {
            self.ui_state.set_status(format!("Validazione fallita: {}", e), StatusLevel::Error);
            return false;
        }

        // Alloca eventuali trigger mancanti
        let _ = allocate_triggers(&mut self.profile);
        self.ui_state.profile = self.profile.clone();

        match save_profile_to_path(&self.profile, &self.profile_path) {
            Ok(_) => {
                self.ui_state.clear_dirty();
                self.dispatcher.set_profile(self.profile.clone());
                self.ui_state.set_status("Profilo salvato e runtime aggiornato.", StatusLevel::Success);

                // Aggiorna registrazione hotkeys
                let triggers: Vec<PhantomTrigger> = self.profile.trigger_allocation.values().copied().collect();
                let _ = self.platform.hotkeys().unregister_all();
                let _ = self.platform.hotkeys().register_triggers(&triggers);

                true
            }
            Err(e) => {
                self.ui_state.set_status(format!("Errore salvataggio: {:?}", e), StatusLevel::Error);
                false
            }
        }
    }

    /// Genera e valida il YAML di configurazione con ch57x-keyboard-tool
    pub fn validate_configuration(&mut self) -> Option<String> {
        let mut profile = self.ui_state.profile.clone();
        let _ = allocate_triggers(&mut profile);

        let yaml_str = match generate_yaml(&profile) {
            Ok(y) => y,
            Err(e) => {
                self.ui_state.set_status(format!("Errore generazione YAML: {:?}", e), StatusLevel::Error);
                return None;
            }
        };

        match self.cli.validate(&yaml_str) {
            Ok(_) => {
                self.ui_state.set_status("Configurazione hardware VALIDA (ch57x-keyboard-tool OK)", StatusLevel::Success);
                Some(yaml_str)
            }
            Err(e) => {
                self.ui_state.set_status(format!("Validazione CLI fallita: {}", e), StatusLevel::Error);
                None
            }
        }
    }

    /// Esegue il flashing sicuro su dispositivo fisico previa verifica del journal
    pub fn upload_configuration_to_device(&mut self) -> bool {
        let status = probe_usb_status();
        if !status.is_ready() {
            self.ui_state.set_status(
                "Impossibile flashare: Macropad non collegato via USB o UsbDk assente.",
                StatusLevel::Error,
            );
            return false;
        }

        let yaml_str = match self.validate_configuration() {
            Some(y) => y,
            None => return false,
        };

        let last_hash = self.journal.last_uploaded_yaml_hash.clone();
        let decision = DeploymentDecision::new(&yaml_str, last_hash.as_deref());

        if !decision.requires_hardware_flash {
            self.ui_state.set_status(
                "Nessuna modifica hardware: tastiera già aggiornata. Salvataggio host completato.",
                StatusLevel::Info,
            );
            self.save_active_profile();
            return true;
        }

        let base_dir = self.base_dir.clone();

        // Transizione di stato Journal: WritingHardwareFlash
        if let Err(e) = self.journal.start_deployment(Some(&base_dir)) {
            self.ui_state.set_status(format!("Journal start error: {:?}", e), StatusLevel::Error);
            return false;
        }

        if let Err(e) = self.journal.step_to_hardware_flash(Some(&base_dir)) {
            self.ui_state.set_status(format!("Journal flash step error: {:?}", e), StatusLevel::Error);
            return false;
        }

        info!("Inizio flashing hardware su memoria dispositivo...");
        match self.cli.upload(&yaml_str) {
            Ok(_) => {
                let _ = self.journal.step_to_activating_runtime(Some(decision.current_yaml_hash), Some(&base_dir));
                let _ = self.journal.step_to_success(Some(&base_dir));
                self.save_active_profile();
                self.ui_state.set_status("Flashing hardware completato con SUCCESSO!", StatusLevel::Success);
                true
            }
            Err(e) => {
                error!("Flashing hardware fallito: {}", e);
                let _ = self.journal.step_to_failed(e.to_string(), Some(&base_dir));
                self.ui_state.set_status(format!("Flashing fallito: {}", e), StatusLevel::Error);
                false
            }
        }
    }
}
`

================================================================================
4. FILE: crates/app/src/tray.rs
================================================================================
`
ust
//! System Tray integration for Windows (with clean non-Windows stubs)
//!
//! Visualizza l'icona VortexDeck nella barra delle notifiche di sistema
//! e fornisce un menu contestuale per controllo rapido:
//! - "Apri VortexDeck"
//! - "Pausa Automazioni (ON/OFF)"
//! - "Visualizza HUD OSD"
//! - Separatore
//! - "Esci"

use crossbeam_channel::{unbounded, Receiver};
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    ShowWindow,
    TogglePause,
    ShowHud,
    Quit,
}

pub struct TrayHandle {
    pub receiver: Receiver<TrayAction>,
    #[cfg(target_os = "windows")]
    _thread: std::thread::JoinHandle<()>,
}

impl TrayHandle {
    /// Inizializza il System Tray su Windows in un thread dedicato
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();

        #[cfg(target_os = "windows")]
        let _thread = {
            let tx = sender.clone();
            std::thread::spawn(move || {
                windows_tray::run_tray_loop(tx);
            })
        };

        info!("System Tray inizializzato.");
        Self {
            receiver,
            #[cfg(target_os = "windows")]
            _thread,
        }
    }
}

impl Default for TrayHandle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "windows")]
mod windows_tray {
    use super::TrayAction;
    use crossbeam_channel::Sender;
    use windows::core::w;
    use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM};
    use windows::Win32::UI::Shell::{
        Shell_NotifyIconW, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW, NIF_ICON, NIF_MESSAGE, NIF_TIP,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DispatchMessageW,
        GetCursorPos, GetMessageW, LoadIconW, PostQuitMessage, RegisterClassW, SetForegroundWindow,
        TrackPopupMenu, IDI_APPLICATION, MF_SEPARATOR, MF_STRING, MSG, TPM_BOTTOMALIGN,
        TPM_LEFTALIGN, TPM_RIGHTBUTTON, WM_APP, WM_DESTROY, WM_LBUTTONDBLCLK, WM_LBUTTONUP,
        WM_RBUTTONUP, WNDCLASSW, WS_OVERLAPPEDWINDOW,
    };

    const WM_TRAY_CALLBACK: u32 = WM_APP + 101;
    const ID_TRAY_OPEN: usize = 1001;
    const ID_TRAY_PAUSE: usize = 1002;
    const ID_TRAY_HUD: usize = 1003;
    const ID_TRAY_EXIT: usize = 1004;

    static mut GLOBAL_TRAY_SENDER: Option<Sender<TrayAction>> = None;

    pub fn run_tray_loop(sender: Sender<TrayAction>) {
        unsafe {
            GLOBAL_TRAY_SENDER = Some(sender);

            let class_name = w!("VortexDeckTrayClass");
            let icon = LoadIconW(None, IDI_APPLICATION).unwrap_or_default();

            let wc = WNDCLASSW {
                lpfnWndProc: Some(tray_wnd_proc),
                hInstance: HINSTANCE::default(),
                lpszClassName: class_name,
                ..Default::default()
            };
            let _ = RegisterClassW(&wc);

            let hwnd = match CreateWindowExW(
                windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
                class_name,
                w!("VortexDeckTrayWindow"),
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                0,
                0,
                None,
                None,
                None,
                None,
            ) {
                Ok(h) => h,
                Err(_) => return,
            };

            let mut nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: hwnd,
                uID: 1,
                uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
                uCallbackMessage: WM_TRAY_CALLBACK,
                hIcon: icon,
                ..Default::default()
            };

            let tip = "VortexDeck — CH57x Studio\0".encode_utf16().collect::<Vec<u16>>();
            let len = tip.len().min(nid.szTip.len());
            nid.szTip[..len].copy_from_slice(&tip[..len]);

            let _ = Shell_NotifyIconW(NIM_ADD, &nid);

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = DispatchMessageW(&msg);
            }

            let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
        }
    }

    unsafe extern "system" fn tray_wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_TRAY_CALLBACK => {
                let event = lparam.0 as u32;
                if event == WM_RBUTTONUP {
                    let mut pt = POINT::default();
                    let _ = GetCursorPos(&mut pt);
                    let _ = SetForegroundWindow(hwnd);

                    let hmenu = CreatePopupMenu().unwrap_or_default();
                    if !hmenu.0.is_null() {
                        let _ = AppendMenuW(hmenu, MF_STRING, ID_TRAY_OPEN, w!("Apri VortexDeck"));
                        let _ = AppendMenuW(hmenu, MF_STRING, ID_TRAY_PAUSE, w!("Pausa Automazioni"));
                        let _ = AppendMenuW(hmenu, MF_STRING, ID_TRAY_HUD, w!("Visualizza HUD OSD"));
                        let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, None);
                        let _ = AppendMenuW(hmenu, MF_STRING, ID_TRAY_EXIT, w!("Esci"));

                        let cmd = TrackPopupMenu(
                            hmenu,
                            TPM_LEFTALIGN | TPM_BOTTOMALIGN | TPM_RIGHTBUTTON,
                            pt.x,
                            pt.y,
                            None,
                            hwnd,
                            None,
                        );

                        if cmd.as_bool() {
                            // Handled in WM_COMMAND
                        }
                        let _ = DestroyMenu(hmenu);
                    }
                } else if event == WM_LBUTTONUP || event == WM_LBUTTONDBLCLK {
                    if let Some(ref sender) = GLOBAL_TRAY_SENDER {
                        let _ = sender.send(TrayAction::ShowWindow);
                    }
                }
                LRESULT(0)
            }
            windows::Win32::UI::WindowsAndMessaging::WM_COMMAND => {
                let cmd_id = wparam.0 & 0xffff;
                if let Some(ref sender) = GLOBAL_TRAY_SENDER {
                    match cmd_id {
                        ID_TRAY_OPEN => {
                            let _ = sender.send(TrayAction::ShowWindow);
                        }
                        ID_TRAY_PAUSE => {
                            let _ = sender.send(TrayAction::TogglePause);
                        }
                        ID_TRAY_HUD => {
                            let _ = sender.send(TrayAction::ShowHud);
                        }
                        ID_TRAY_EXIT => {
                            let _ = sender.send(TrayAction::Quit);
                            PostQuitMessage(0);
                        }
                        _ => {}
                    }
                }
                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
`

================================================================================
5. FILE: crates/ui/src/lib.rs
================================================================================
`
ust
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
`

================================================================================
6. FILE: docs/agent_questions.md
================================================================================
`markdown
# CH57x Studio — Registro Domande Master Agent & Decisioni Utente

## Stato Corrente
- **Fase:** Wave 0, Wave 1, Wave 2, Wave 3 completate al 100%. Avvio Gate 4 (Test e Hardware Flash).
- **Prodotto:** VortexDeck v0.1.0 (eseguibile `vortexdeck.exe` in `dist/VortexDeck/`)
- **Data:** 19/10/2026

---

## Stato Risoluzioni Wave 0-3 (Ambiente, Architettura, Runtime, UI e Packaging)

### 1. Toolchain Rust & Risoluzione Linker GNU
- **Esito:** `rustc 1.99.1` e `cargo 1.99.1` operativi su target `x86_64-pc-windows-gnu`. Risolto symlink assembler MinGW creando wrapper `as.exe` e libreria di importazione `libshlwapi.a` (378 KB).
- **Stato:** [COMPLETO]

### 2. Upstream CLI `ch57x-keyboard-tool`
- **Esito:** `bin/ch57x-keyboard-tool.exe` integrato e testato. Validazione byte-exact con tutti i 54 slot confermata (`config is valid 👌`).
- **Stato:** [COMPLETO]

### 3. Repository Remota GitHub
- **Esito:** Repository privata `https://github.com/KojiroXor/ch57x-studio` sincronizzata con i commit delle Wave 0, Wave 1, Wave 2 e Wave 3 sul branch `main`.
- **Stato:** [COMPLETO]

### 4. Test Suite & Qualità Codice
- **Esito:** 73/73 test unitari, di integrazione e contract test passati (0 fallimenti). `cargo clippy --workspace -- -D warnings` a 0 avvisi.
- **Stato:** [COMPLETO]

### 5. Binario Release & Distribuzione
- **Esito:** Compilato `target/release/vortexdeck.exe` (17.1 MB). Assemblato bundle autonomo `dist/VortexDeck/` pronto all'uso con CLI upstream e mapping YAML.
- **Stato:** [COMPLETO]

---

## Gate 4: Test Hardware Flash & Installazione UsbDk (STOP & ASK)

### Domanda per l'Utente:
Per programmare fisicamente la memoria flash del Macropad (tramite `ch57x-keyboard-tool upload`):
1. Vuoi che proviamo ad avviare la GUI `vortexdeck.exe` per consentirti di esplorare l'interfaccia visuale, la griglia 4x3 e gli encoder?
2. Per la scrittura USB diretta (`upload`), è necessario installare il driver di sistema `UsbDk` (scaricabile dal repository ufficiale Daynix: https://github.com/daynix/UsbDk/releases). Vuoi procedere all'installazione o preferisci testare prima l'applicazione in modalità Host Runtime?
`
