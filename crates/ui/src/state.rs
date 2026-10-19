//! Reactive application UI state (`UiState`)
//!
//! Gestisce layer di editing (1, 2, 3), selezione slot, dirty tracking,
//! notifiche di status bar, countdown di cattura finestra e wizard di calibrazione.

use ch57x_domain::actions::{
    ActionDefinition, ActionId, ContextDispatchAction, ExecProcessAction, LaunchOrFocusAction,
    MacroConcurrencyPolicy, MacroSequenceAction, RestoreWorkspaceAction, SlotBinding,
    WindowSelector,
};
use ch57x_domain::hardware::{
    EncoderEvent, LayerIndex, SlotId, BUTTONS_PER_LAYER, ENCODERS_PER_LAYER, LAYER_COUNT,
    TOTAL_SLOTS,
};
use ch57x_domain::profile::{Profile, SlotConfig};
use ch57x_domain::triggers::PhantomTrigger;
use ch57x_hardware::cli_adapter::Ch57xCli;
use ch57x_hardware::usb_probe::{is_usbdk_installed, probe_usb_status, UsbStatus};
use ch57x_platform::PlatformBackend;
use std::path::PathBuf;
use std::time::Instant;

/// Livello di gravità per i messaggi di stato
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusLevel {
    Info,
    Success,
    Warning,
    Error,
}

/// Messaggio di notifica per la status bar
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusAlert {
    pub text: String,
    pub level: StatusLevel,
}

/// Progresso non-bloccante del countdown per cattura finestra
#[derive(Debug, Clone, PartialEq)]
pub struct WindowCaptureProgress {
    pub slot_id: SlotId,
    pub remaining_seconds: f32,
    pub total_seconds: f32,
}

/// Stato del wizard di calibrazione fisica
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalibrationState {
    pub tested_buttons: [bool; BUTTONS_PER_LAYER], // 12
    pub tested_encoders: [[bool; 3]; ENCODERS_PER_LAYER], // 2 encoders * 3 eventi (CCW, Press, CW)
    pub swap_order: bool,
}

impl Default for CalibrationState {
    fn default() -> Self {
        Self {
            tested_buttons: [false; BUTTONS_PER_LAYER],
            tested_encoders: [[false; 3]; ENCODERS_PER_LAYER],
            swap_order: false,
        }
    }
}

impl CalibrationState {
    pub fn press_button(&mut self, idx: usize) {
        if idx < BUTTONS_PER_LAYER {
            self.tested_buttons[idx] = true;
        }
    }

    pub fn press_encoder(&mut self, enc_idx: usize, event: EncoderEvent) {
        if enc_idx < ENCODERS_PER_LAYER {
            let ev_idx = match event {
                EncoderEvent::Ccw => 0,
                EncoderEvent::Press => 1,
                EncoderEvent::Cw => 2,
            };
            self.tested_encoders[enc_idx][ev_idx] = true;
        }
    }

    pub fn toggle_swap(&mut self) {
        self.swap_order = !self.swap_order;
    }

    pub fn tested_count(&self) -> usize {
        let btn_count = self.tested_buttons.iter().filter(|&&b| b).count();
        let enc_count = self
            .tested_encoders
            .iter()
            .flat_map(|enc| enc.iter())
            .filter(|&&b| b)
            .count();
        btn_count + enc_count
    }

    pub fn total_controls(&self) -> usize {
        BUTTONS_PER_LAYER + (ENCODERS_PER_LAYER * 3) // 18
    }

    pub fn is_complete(&self) -> bool {
        self.tested_count() == self.total_controls()
    }

    pub fn reset(&mut self) {
        self.tested_buttons = [false; BUTTONS_PER_LAYER];
        self.tested_encoders = [[false; 3]; ENCODERS_PER_LAYER];
    }
}

/// Cache delle verifiche diagnostiche per il pannello Doctor
#[derive(Debug, Clone)]
pub struct DoctorCache {
    pub cli_path: Option<PathBuf>,
    pub cli_valid: bool,
    pub cli_error: Option<String>,
    pub usb_status: UsbStatus,
    pub usbdk_installed: bool,
    pub hotkey_conflicts: Vec<PhantomTrigger>,
    pub hotkeys_tested: bool,
    pub last_checked: Option<Instant>,
}

impl Default for DoctorCache {
    fn default() -> Self {
        Self {
            cli_path: None,
            cli_valid: false,
            cli_error: None,
            usb_status: UsbStatus::Disconnected,
            usbdk_installed: false,
            hotkey_conflicts: Vec::new(),
            hotkeys_tested: false,
            last_checked: None,
        }
    }
}

impl DoctorCache {
    /// Esegue il refresh di tutte le metriche diagnostiche
    pub fn refresh(&mut self, platform: Option<&dyn PlatformBackend>) {
        // 1. CLI tool
        match Ch57xCli::auto_detect() {
            Ok(cli) => {
                self.cli_path = Some(cli.binary_path().to_path_buf());
                match cli.show_keys() {
                    Ok(_) => {
                        self.cli_valid = true;
                        self.cli_error = None;
                    }
                    Err(e) => {
                        self.cli_valid = false;
                        self.cli_error = Some(e.to_string());
                    }
                }
            }
            Err(e) => {
                self.cli_path = None;
                self.cli_valid = false;
                self.cli_error = Some(e.to_string());
            }
        }

        // 2. USB Status
        self.usb_status = probe_usb_status();

        // 3. UsbDk
        self.usbdk_installed = is_usbdk_installed();

        // 4. Hotkeys test (se platform fornito)
        if let Some(p) = platform {
            let candidates: Vec<PhantomTrigger> = (13..=24)
                .filter_map(|k| PhantomTrigger::new(ch57x_domain::triggers::TriggerBank::BankA, k))
                .collect();
            match p.hotkeys().register_triggers(&candidates) {
                Ok(conflicts) => {
                    self.hotkey_conflicts = conflicts;
                    self.hotkeys_tested = true;
                    let _ = p.hotkeys().unregister_all();
                }
                Err(_) => {
                    self.hotkeys_tested = false;
                }
            }
        }

        self.last_checked = Some(Instant::now());
    }
}

/// Vista principale attiva nell'interfaccia
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveView {
    Editor,
    Doctor,
    Hud,
}

/// Stato reattivo globale della UI
pub struct UiState {
    /// Layer attualmente modificato (Layer 1..=3 -> LayerIndex(0..=2))
    pub editing_layer: LayerIndex,
    /// Slot attualmente selezionato per la modifica nell'Inspector
    pub selected_slot: Option<SlotId>,
    /// Profilo di configurazione caricato
    pub profile: Profile,
    /// Flag modifiche non salvate
    pub is_dirty: bool,
    /// Notifica/messaggio di stato corrente
    pub status_message: Option<StatusAlert>,
    /// Stato del countdown di cattura finestra attiva
    pub window_capture: Option<WindowCaptureProgress>,
    /// Stato del wizard di calibrazione fisica
    pub calibration: CalibrationState,
    /// Cache dei controlli diagnostici Doctor
    pub doctor_cache: DoctorCache,
    /// Vista principale attiva (Editor canvas, Doctor panel o HUD)
    pub active_view: ActiveView,
    /// Feedback visivo per slot recentemente cliccato (slot, timestamp)
    pub click_feedback: Option<(SlotId, Instant)>,
}

impl Default for UiState {
    fn default() -> Self {
        let profile = Profile::default();
        let swap = profile.device.encoder_order_swapped;
        Self {
            editing_layer: LayerIndex(0),
            selected_slot: None,
            profile,
            is_dirty: false,
            status_message: None,
            window_capture: None,
            calibration: CalibrationState {
                swap_order: swap,
                ..Default::default()
            },
            doctor_cache: DoctorCache::default(),
            active_view: ActiveView::Editor,
            click_feedback: None,
        }
    }
}

impl UiState {
    /// Crea un nuovo UiState con un profilo specificato
    pub fn new(profile: Profile) -> Self {
        let swap = profile.device.encoder_order_swapped;
        Self {
            editing_layer: LayerIndex(0),
            selected_slot: None,
            profile,
            is_dirty: false,
            status_message: None,
            window_capture: None,
            calibration: CalibrationState {
                swap_order: swap,
                ..Default::default()
            },
            doctor_cache: DoctorCache::default(),
            active_view: ActiveView::Editor,
            click_feedback: None,
        }
    }

    /// Imposta il layer corrente di modifica (0..=2)
    pub fn set_editing_layer(&mut self, layer: LayerIndex) {
        if (layer.0 as usize) < LAYER_COUNT {
            self.editing_layer = layer;
            // Se lo slot selezionato appartiene a un altro layer, ri-mappa sul medesimo controllo del nuovo layer
            if let Some(slot) = self.selected_slot {
                self.selected_slot = Some(SlotId {
                    layer,
                    control: slot.control,
                });
            }
        }
    }

    /// Seleziona uno slot tramite SlotId
    pub fn select_slot(&mut self, slot: SlotId) {
        self.selected_slot = Some(slot);
        self.click_feedback = Some((slot, Instant::now()));
    }

    /// Seleziona uno slot tramite indice globale (0..53)
    pub fn select_slot_by_index(&mut self, global_index: usize) {
        if global_index < TOTAL_SLOTS {
            for slot in SlotId::all_slots() {
                if slot.global_index() == global_index {
                    self.select_slot(slot);
                    self.editing_layer = slot.layer;
                    break;
                }
            }
        }
    }

    /// Deseleziona lo slot attivo
    pub fn clear_selection(&mut self) {
        self.selected_slot = None;
    }

    /// Restituisce l'indice globale 0..53 dello slot selezionato
    pub fn selected_global_index(&self) -> Option<usize> {
        self.selected_slot.map(|s| s.global_index())
    }

    /// Restituisce la configurazione dello slot selezionato
    pub fn selected_slot_config(&self) -> Option<&SlotConfig> {
        let idx = self.selected_global_index()?;
        self.profile.slots.get(&idx)
    }

    /// Restituisce un riferimento mutabile alla configurazione dello slot selezionato
    pub fn selected_slot_config_mut(&mut self) -> Option<&mut SlotConfig> {
        let idx = self.selected_global_index()?;
        self.profile.slots.get_mut(&idx)
    }

    /// Restituisce la configurazione per un qualsiasi SlotId
    pub fn get_slot_config(&self, slot: SlotId) -> Option<&SlotConfig> {
        self.profile.slots.get(&slot.global_index())
    }

    /// Restituisce un riferimento mutabile alla configurazione per un qualsiasi SlotId
    pub fn get_slot_config_mut(&mut self, slot: SlotId) -> Option<&mut SlotConfig> {
        self.profile.slots.get_mut(&slot.global_index())
    }

    /// Segna lo stato come modificato (dirty)
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    /// Ripristina il flag dirty dopo salvataggio
    pub fn clear_dirty(&mut self) {
        self.is_dirty = false;
    }

    /// Imposta una notifica per la barra di stato
    pub fn set_status(&mut self, text: impl Into<String>, level: StatusLevel) {
        self.status_message = Some(StatusAlert {
            text: text.into(),
            level,
        });
    }

    /// Pulisce il messaggio di stato
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Avvia il countdown non-bloccante di cattura finestra
    pub fn start_window_capture(&mut self, slot_id: SlotId, duration_secs: f32) {
        self.window_capture = Some(WindowCaptureProgress {
            slot_id,
            remaining_seconds: duration_secs,
            total_seconds: duration_secs,
        });
        self.set_status(
            format!(
                "Porta in primo piano la finestra desiderata... ({:.0}s)",
                duration_secs
            ),
            StatusLevel::Info,
        );
    }

    /// Annulla il countdown di cattura finestra
    pub fn cancel_window_capture(&mut self) {
        self.window_capture = None;
        self.set_status("Cattura finestra annullata.", StatusLevel::Warning);
    }

    /// Avanza il timer di cattura finestra di `delta_secs`.
    /// Ritorna `Some(SlotId)` se il countdown è terminato e la cattura deve scattare ora.
    pub fn tick_window_capture(&mut self, delta_secs: f32) -> Option<SlotId> {
        if let Some(progress) = &mut self.window_capture {
            progress.remaining_seconds -= delta_secs;
            if progress.remaining_seconds <= 0.0 {
                let target_slot = progress.slot_id;
                self.window_capture = None;
                return Some(target_slot);
            }
        }
        None
    }

    /// Calcola il badge compatto per lo slot specificato:
    /// `HID`, `WIN`, `WSP`, `MAC`, `CMD`, `CTX`, `DIS`
    pub fn get_badge_for_slot(&self, slot: SlotId) -> &'static str {
        if let Some(config) = self.get_slot_config(slot) {
            match &config.binding {
                SlotBinding::Disabled => "DIS",
                SlotBinding::HidNative(_) => "HID",
                SlotBinding::HostActionRef(action_id) => {
                    if let Some(action) = self.profile.actions.get(action_id) {
                        match action {
                            ActionDefinition::LaunchOrFocus(_) => "WIN",
                            ActionDefinition::RestoreWorkspace(_) => "WSP",
                            ActionDefinition::MacroSequence(_) => "MAC",
                            ActionDefinition::ExecProcess(_) => "CMD",
                            ActionDefinition::ContextDispatch(_) => "CTX",
                            ActionDefinition::ShowHud { .. } => "HUD",
                        }
                    } else {
                        "OFF"
                    }
                }
            }
        } else {
            "DIS"
        }
    }

    /// Restituisce la definizione dell'azione associata a uno slot, se HostActionRef
    pub fn get_action_for_slot(&self, slot: SlotId) -> Option<&ActionDefinition> {
        let config = self.get_slot_config(slot)?;
        if let SlotBinding::HostActionRef(action_id) = &config.binding {
            self.profile.actions.get(action_id)
        } else {
            None
        }
    }

    /// Restituisce un riferimento mutabile all'azione associata a uno slot, se HostActionRef
    pub fn get_action_for_slot_mut(&mut self, slot: SlotId) -> Option<&mut ActionDefinition> {
        let idx = slot.global_index();
        let config = self.profile.slots.get(&idx)?;
        if let SlotBinding::HostActionRef(action_id) = &config.binding {
            let id = *action_id;
            self.profile.actions.get_mut(&id)
        } else {
            None
        }
    }

    /// Restituisce il token trigger phantom se assegnato allo slot (es. "f13", "ctrl-f14")
    pub fn get_trigger_token_for_slot(&self, slot: SlotId) -> Option<String> {
        let idx = slot.global_index();
        self.profile
            .trigger_allocation
            .get(&idx)
            .map(|t| t.yaml_token())
    }

    /// Assegna una nuova o esistente `ActionDefinition` allo slot selezionato e ne aggiorna il binding
    pub fn assign_action_to_selected_slot(&mut self, action: ActionDefinition) -> Option<ActionId> {
        let slot = self.selected_slot?;
        let action_id = ActionId::new_v4();
        self.profile.actions.insert(action_id, action);

        let idx = slot.global_index();
        if let Some(config) = self.profile.slots.get_mut(&idx) {
            config.binding = SlotBinding::HostActionRef(action_id);
            self.mark_dirty();
            Some(action_id)
        } else {
            None
        }
    }

    /// Cambia il tipo di binding per lo slot selezionato garantendo la corretta istanziazione di default
    pub fn set_selected_binding_type(&mut self, target_type: &str) {
        let slot = match self.selected_slot {
            Some(s) => s,
            None => return,
        };

        match target_type {
            "Disabled" => {
                if let Some(cfg) = self.get_slot_config_mut(slot) {
                    cfg.binding = SlotBinding::Disabled;
                    self.mark_dirty();
                }
            }
            "HidNative" => {
                if let Some(cfg) = self.get_slot_config_mut(slot) {
                    cfg.binding =
                        SlotBinding::HidNative(ch57x_domain::actions::HidAction::ConsumerMedia(
                            ch57x_domain::actions::MediaToken::PlayPause,
                        ));
                    self.mark_dirty();
                }
            }
            "LaunchOrFocus" => {
                let action = ActionDefinition::LaunchOrFocus(LaunchOrFocusAction {
                    selector: WindowSelector {
                        exe_name: Some("notepad.exe".into()),
                        title_contains: None,
                        class_name: None,
                    },
                    placement: None,
                    launch: None,
                    timeout_ms: 3000,
                });
                self.assign_action_to_selected_slot(action);
            }
            "RestoreWorkspace" => {
                let action = ActionDefinition::RestoreWorkspace(RestoreWorkspaceAction {
                    targets: Vec::new(),
                    post_delay_ms: 100,
                });
                self.assign_action_to_selected_slot(action);
            }
            "MacroSequence" => {
                let action = ActionDefinition::MacroSequence(MacroSequenceAction {
                    steps: Vec::new(),
                    concurrency: MacroConcurrencyPolicy::IgnoreIfRunning,
                    require_initial_foreground_match: false,
                });
                self.assign_action_to_selected_slot(action);
            }
            "ExecProcess" => {
                let action = ActionDefinition::ExecProcess(ExecProcessAction {
                    program: PathBuf::from("cmd.exe"),
                    args: vec!["/c".into(), "echo Hello".into()],
                    working_dir: None,
                    timeout_ms: 5000,
                    hide_console_window: true,
                });
                self.assign_action_to_selected_slot(action);
            }
            "ContextDispatch" => {
                let fallback = ActionId::new_v4();
                self.profile
                    .actions
                    .insert(fallback, ActionDefinition::ShowHud { duration_ms: 1500 });
                let action = ActionDefinition::ContextDispatch(ContextDispatchAction {
                    rules: Vec::new(),
                    fallback_action_id: fallback,
                });
                self.assign_action_to_selected_slot(action);
            }
            _ => {}
        }
    }

    /// Verifica se uno slot è stato cliccato recentemente (< 150ms) per feedback visivo
    pub fn is_slot_recently_clicked(&self, slot: SlotId) -> bool {
        if let Some((clicked_slot, time)) = self.click_feedback {
            if clicked_slot == slot && time.elapsed().as_millis() < 150 {
                return true;
            }
        }
        false
    }
}

/// Helper per convertire codice esadecimale (#RRGGBB o #RRGGBBAA) in `egui::Color32`
pub fn hex_to_color32(hex: &str) -> Option<egui::Color32> {
    let s = hex.trim().trim_start_matches('#');
    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some(egui::Color32::from_rgb(r, g, b))
    } else if s.len() == 8 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        let a = u8::from_str_radix(&s[6..8], 16).ok()?;
        Some(egui::Color32::from_rgba_unmultiplied(r, g, b, a))
    } else {
        None
    }
}

/// Helper per convertire `egui::Color32` in codice esadecimale #RRGGBB
pub fn color32_to_hex(color: egui::Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ch57x_domain::hardware::ControlId;

    #[test]
    fn test_ui_state_initialization() {
        let state = UiState::default();
        assert_eq!(state.editing_layer.0, 0);
        assert_eq!(state.selected_slot, None);
        assert!(!state.is_dirty);
        assert_eq!(state.status_message, None);
        assert_eq!(state.window_capture, None);
        assert_eq!(state.active_view, ActiveView::Editor);
    }

    #[test]
    fn test_layer_switch_and_slot_selection() {
        let mut state = UiState::default();

        let slot = SlotId::new_button(0, 1, 2).unwrap();
        state.select_slot(slot);
        assert_eq!(state.selected_slot, Some(slot));
        assert_eq!(state.selected_global_index(), Some(slot.global_index()));

        // Switch to Layer 2 (index 1)
        state.set_editing_layer(LayerIndex(1));
        assert_eq!(state.editing_layer, LayerIndex(1));
        // Selected slot must track to new layer with same control
        assert_eq!(
            state.selected_slot,
            Some(SlotId {
                layer: LayerIndex(1),
                control: ControlId::Button { row: 1, col: 2 }
            })
        );
    }

    #[test]
    fn test_select_slot_by_index() {
        let mut state = UiState::default();
        state.select_slot_by_index(15); // Layer 0, encoder 1 CCW
        assert!(state.selected_slot.is_some());
        assert_eq!(state.selected_global_index(), Some(15));
    }

    #[test]
    fn test_dirty_tracking() {
        let mut state = UiState::default();
        assert!(!state.is_dirty);
        state.mark_dirty();
        assert!(state.is_dirty);
        state.clear_dirty();
        assert!(!state.is_dirty);
    }

    #[test]
    fn test_status_alert() {
        let mut state = UiState::default();
        state.set_status("Test alert", StatusLevel::Success);
        assert_eq!(
            state.status_message,
            Some(StatusAlert {
                text: "Test alert".into(),
                level: StatusLevel::Success,
            })
        );
        state.clear_status();
        assert_eq!(state.status_message, None);
    }

    #[test]
    fn test_window_capture_countdown() {
        let mut state = UiState::default();
        let slot = SlotId::new_button(0, 0, 0).unwrap();
        state.start_window_capture(slot, 3.0);
        assert!(state.window_capture.is_some());

        // Tick 1 second
        let completed = state.tick_window_capture(1.0);
        assert_eq!(completed, None);
        assert!(state.window_capture.is_some());

        // Tick 2.5 seconds (exceeding 3.0)
        let completed2 = state.tick_window_capture(2.5);
        assert_eq!(completed2, Some(slot));
        assert_eq!(state.window_capture, None);
    }

    #[test]
    fn test_calibration_state() {
        let mut cal = CalibrationState::default();
        assert_eq!(cal.tested_count(), 0);
        assert!(!cal.is_complete());

        cal.press_button(0);
        cal.press_button(11);
        assert_eq!(cal.tested_count(), 2);

        cal.press_encoder(0, EncoderEvent::Ccw);
        cal.press_encoder(0, EncoderEvent::Press);
        cal.press_encoder(0, EncoderEvent::Cw);
        assert_eq!(cal.tested_count(), 5);

        cal.toggle_swap();
        assert!(cal.swap_order);

        cal.reset();
        assert_eq!(cal.tested_count(), 0);
    }

    #[test]
    fn test_hex_conversion() {
        let col = egui::Color32::from_rgb(255, 128, 0);
        let hex = color32_to_hex(col);
        assert_eq!(hex, "#FF8000");

        let parsed = hex_to_color32(&hex).unwrap();
        assert_eq!(parsed.r(), 255);
        assert_eq!(parsed.g(), 128);
        assert_eq!(parsed.b(), 0);

        assert_eq!(hex_to_color32("invalid"), None);
    }

    #[test]
    fn test_badge_resolution() {
        let mut state = UiState::default();
        let slot = SlotId::new_button(0, 0, 0).unwrap();
        state.select_slot(slot);

        assert_eq!(state.get_badge_for_slot(slot), "DIS");

        state.set_selected_binding_type("HidNative");
        assert_eq!(state.get_badge_for_slot(slot), "HID");

        state.set_selected_binding_type("LaunchOrFocus");
        assert_eq!(state.get_badge_for_slot(slot), "WIN");

        state.set_selected_binding_type("RestoreWorkspace");
        assert_eq!(state.get_badge_for_slot(slot), "WSP");

        state.set_selected_binding_type("MacroSequence");
        assert_eq!(state.get_badge_for_slot(slot), "MAC");

        state.set_selected_binding_type("ExecProcess");
        assert_eq!(state.get_badge_for_slot(slot), "CMD");

        state.set_selected_binding_type("ContextDispatch");
        assert_eq!(state.get_badge_for_slot(slot), "CTX");

        state.set_selected_binding_type("Disabled");
        assert_eq!(state.get_badge_for_slot(slot), "DIS");
    }
}
