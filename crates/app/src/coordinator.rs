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
