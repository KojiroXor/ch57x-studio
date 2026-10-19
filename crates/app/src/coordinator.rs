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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use crossbeam_channel::{unbounded, Receiver, Sender};
use tracing::{error, info, warn};

use ch57x_compiler::allocator::allocate_triggers;
use ch57x_compiler::diff::DeploymentDecision;
use ch57x_compiler::yaml_gen::generate_yaml;
use ch57x_domain::profile::Profile;
use ch57x_domain::triggers::PhantomTrigger;
use ch57x_domain::validation::validate_profile;
use ch57x_hardware::cli_adapter::Ch57xCli;
use ch57x_hardware::usb_probe::{probe_usb_status, UsbStatus};
use ch57x_persistence::journal::DeploymentJournal;
use ch57x_persistence::storage::{get_app_data_dir, get_profiles_dir, load_profile_from_path, save_profile_to_path};
use ch57x_persistence::trust::TrustStore;
use ch57x_platform::get_platform;
use ch57x_platform::traits::PlatformBackend;
use ch57x_runtime::dispatcher::RuntimeDispatcher;
use ch57x_ui::state::{StatusLevel, UiState};

#[derive(Debug, Clone)]
pub struct HardwareFlashPreview {
    pub yaml_str: String,
    pub current_hash: String,
    #[allow(dead_code)]
    pub last_hash: Option<String>,
    pub requires_hardware_flash: bool,
    pub modified_slots_count: usize,
}

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
    FlashSuccess,
    FlashFailed(String),
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
    pub cached_usb_status: UsbStatus,
    pub last_usb_poll: Instant,
    pub is_flashing: Arc<AtomicBool>,
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
            if let Err(e) = allocate_triggers(&mut def) {
                warn!("Errore prima allocazione trigger default: {:?}", e);
            }
            let _ = save_profile_to_path(&def, &profile_path);
            def
        };

        let cli = Ch57xCli::auto_detect()
            .unwrap_or_else(|_| Ch57xCli::new("bin/ch57x-keyboard-tool.exe"))
            .with_vendor_id(0x1189)
            .with_product_id(0x8890);

        // Singola istanza condivisa di Platform
        let platform_inst = Arc::new(get_platform());
        let platform: Arc<dyn PlatformBackend> = platform_inst.clone();

        let dispatcher = Arc::new(RuntimeDispatcher::new(
            profile.clone(),
            platform_inst.clone(),
            platform_inst,
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

        // Thread dedicato background per l'inoltro diretto delle hotkey a RuntimeDispatcher (Bug 1 & 2)
        let hotkey_rx = platform.hotkeys().subscribe();
        let disp_bg = dispatcher.clone();
        let cmd_tx_bg = command_sender.clone();

        std::thread::Builder::new()
            .name("vortexdeck-hotkey-forwarder".into())
            .spawn(move || {
                while let Ok(trigger) = hotkey_rx.recv() {
                    // 1. Inoltro immediato a RuntimeDispatcher sul thread background
                    let _ = disp_bg.trigger_received(trigger);
                    // 2. Notifica alla UI per evidenziare il controllo o visualizzare l'OSD
                    let _ = cmd_tx_bg.send(CoordinatorCommand::TriggerEvent(trigger));
                    // 3. Risveglio immediato del renderer egui/eframe
                    crate::tray::notify_repaint();
                }
            })
            .expect("Failed to spawn hotkey forwarder thread");

        let cached_usb_status = probe_usb_status();
        let last_usb_poll = Instant::now();
        let is_flashing = Arc::new(AtomicBool::new(false));

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
            cached_usb_status,
            last_usb_poll,
            is_flashing,
        })
    }

    /// Rilevamento USB con cache a 2 secondi (Bug 3)
    pub fn get_usb_status(&mut self) -> UsbStatus {
        if self.last_usb_poll.elapsed() >= Duration::from_secs(2) {
            self.cached_usb_status = probe_usb_status();
            if let UsbStatus::Ready { vid, pid } = self.cached_usb_status {
                self.cli = self.cli.clone().with_vendor_id(vid).with_product_id(pid);
            }
            self.last_usb_poll = Instant::now();
        }
        self.cached_usb_status.clone()
    }

    pub fn is_flashing(&self) -> bool {
        self.is_flashing.load(Ordering::SeqCst)
    }

    /// Processa tutti i comandi in coda non-bloccanti
    pub fn process_pending_commands(&mut self) {
        while let Ok(cmd) = self.command_receiver.try_recv() {
            match cmd {
                CoordinatorCommand::TriggerEvent(trigger) => {
                    info!("Ricevuto trigger hardware/virtuale: {}", trigger.yaml_token());
                    // Visual feedback nella UI: seleziona lo slot corrispondente se presente
                    if let Some((&slot_idx, _)) = self.ui_state.profile.trigger_allocation.iter().find(|(_, &t)| t == trigger) {
                        if let Some(slot_cfg) = self.ui_state.profile.slots.get(&slot_idx) {
                            self.ui_state.selected_slot = Some(slot_cfg.slot);
                        }
                    }
                }
                CoordinatorCommand::SaveProfile => {
                    self.save_active_profile();
                }
                CoordinatorCommand::ValidateYaml => {
                    let _ = self.validate_configuration();
                }
                CoordinatorCommand::UploadToDevice => {
                    // Avviato tramite modal di conferma
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
                    // Chiusura pulita gestita dal loop dell'app senza exit(0)
                }
                CoordinatorCommand::FlashSuccess => {
                    self.save_active_profile();
                    self.journal = DeploymentJournal::load_or_recover(&self.base_dir).unwrap_or_default();
                    self.ui_state.set_status("Flashing hardware completato con SUCCESSO!", StatusLevel::Success);
                }
                CoordinatorCommand::FlashFailed(err) => {
                    self.journal = DeploymentJournal::load_or_recover(&self.base_dir).unwrap_or_default();
                    self.ui_state.set_status(format!("Flashing fallito: {err}"), StatusLevel::Error);
                }
            }
        }
    }

    /// Salva atomicamente il profilo su disco e aggiorna il dispatcher
    pub fn save_active_profile(&mut self) -> bool {
        self.profile = self.ui_state.profile.clone();

        // Alloca eventuali trigger mancanti con gestione errore (Bug 8)
        if let Err(e) = allocate_triggers(&mut self.profile) {
            self.ui_state.set_status(format!("Allocazione trigger fallita: {}", e), StatusLevel::Error);
            return false;
        }

        // Verifica semantica prima del salvataggio
        if let Err(e) = validate_profile(&self.profile) {
            self.ui_state.set_status(format!("Validazione fallita: {}", e), StatusLevel::Error);
            return false;
        }

        self.ui_state.profile = self.profile.clone();

        match save_profile_to_path(&self.profile, &self.profile_path) {
            Ok(_) => {
                self.ui_state.clear_dirty();
                self.dispatcher.set_profile(self.profile.clone());
                self.ui_state.set_status("Profilo salvato e runtime host aggiornato.", StatusLevel::Success);

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
        if let Err(e) = allocate_triggers(&mut profile) {
            self.ui_state.set_status(format!("Errore allocazione trigger: {:?}", e), StatusLevel::Error);
            return None;
        }

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

    /// Genera la preview per il modal di conferma flashing hardware (Bug 4)
    pub fn generate_flash_preview(&mut self) -> Result<HardwareFlashPreview, String> {
        let mut profile = self.ui_state.profile.clone();
        allocate_triggers(&mut profile).map_err(|e| format!("Allocazione trigger fallita: {e}"))?;

        let yaml_str = generate_yaml(&profile).map_err(|e| format!("Generazione YAML fallita: {e}"))?;
        self.cli.validate(&yaml_str).map_err(|e| format!("Validazione hardware CLI fallita: {e}"))?;

        let last_hash = self.journal.last_uploaded_yaml_hash.clone();
        let decision = DeploymentDecision::new(&yaml_str, last_hash.as_deref());

        let modified_slots = profile.slots.iter().filter(|(_, s)| !matches!(s.binding, ch57x_domain::actions::SlotBinding::Disabled)).count();

        Ok(HardwareFlashPreview {
            yaml_str,
            current_hash: decision.current_yaml_hash,
            last_hash,
            requires_hardware_flash: decision.requires_hardware_flash,
            modified_slots_count: modified_slots,
        })
    }

    /// Esegue il flashing hardware in un thread background non-bloccante (Bug 4)
    pub fn start_hardware_flash(&mut self, yaml_str: String, current_hash: String) -> Result<(), String> {
        if self.is_flashing() {
            return Err("Operazione di flashing già in corso!".into());
        }

        let is_flashing = self.is_flashing.clone();
        let cli = self.cli.clone();
        let mut journal = self.journal.clone();
        let base_dir = self.base_dir.clone();
        let cmd_tx = self.command_sender.clone();

        is_flashing.store(true, Ordering::SeqCst);
        self.ui_state.set_status("Scrittura memoria flash hardware in corso...", StatusLevel::Warning);

        std::thread::Builder::new()
            .name("vortexdeck-hardware-flasher".into())
            .spawn(move || {
                let outcome = (|| -> Result<(), String> {
                    let _ = journal.start_deployment(Some(&base_dir));
                    let _ = journal.step_to_hardware_flash(Some(&base_dir));

                    cli.upload(&yaml_str).map_err(|e| e.to_string())?;

                    let _ = journal.step_to_activating_runtime(Some(current_hash), Some(&base_dir));
                    let _ = journal.step_to_success(Some(&base_dir));
                    Ok(())
                })();

                is_flashing.store(false, Ordering::SeqCst);

                match outcome {
                    Ok(_) => {
                        let _ = cmd_tx.send(CoordinatorCommand::FlashSuccess);
                    }
                    Err(e) => {
                        error!("Flashing hardware fallito: {}", e);
                        let _ = journal.step_to_failed(e.clone(), Some(&base_dir));
                        let _ = cmd_tx.send(CoordinatorCommand::FlashFailed(e));
                    }
                }

                crate::tray::notify_repaint();
            })
            .map_err(|e| format!("Impossibile avviare il thread di flash: {e}"))?;

        Ok(())
    }
}
