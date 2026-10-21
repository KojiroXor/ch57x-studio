use ch57x_domain::actions::{
    ForegroundAppInfo, WindowActionResult, WindowPlacement, WindowSelector,
};
use ch57x_domain::capabilities::CapabilityMatrix;
use ch57x_domain::errors::PlatformError;
use ch57x_domain::triggers::PhantomTrigger;
use std::path::Path;

pub trait PlatformBackend: Send + Sync {
    fn capability_matrix(&self) -> CapabilityMatrix;
    fn hotkeys(&self) -> &dyn HotkeyBackend;
    fn windows(&self) -> &dyn WindowManagerBackend;
    fn input(&self) -> &dyn InputInjectorBackend;
    fn autostart(&self) -> &dyn AutostartBackend;
    fn adjust_brightness(&self, _delta: i32) -> Result<u32, PlatformError> {
        Ok(50)
    }
}

pub trait HotkeyBackend: Send + Sync {
    /// Registra l'insieme di trigger richiesti dal profilo attivo.
    /// Restituisce la lista di eventuali trigger andati in conflitto col sistema operativo.
    fn register_triggers(
        &self,
        triggers: &[PhantomTrigger],
    ) -> Result<Vec<PhantomTrigger>, PlatformError>;
    fn unregister_all(&self) -> Result<(), PlatformError>;
    /// Sottoscrive il ricevitore per gli eventi trigger hardware/hotkey ricevuti in tempo reale.
    fn subscribe(&self) -> crossbeam_channel::Receiver<PhantomTrigger>;
}

pub trait WindowManagerBackend: Send + Sync {
    /// Legge la finestra attualmente in primo piano (escludendo il processo corrente se richiesto).
    fn capture_foreground_window(
        &self,
        exclude_pid: u32,
    ) -> Result<ForegroundAppInfo, PlatformError>;
    /// Enumera tutte le finestre visibili idonee per uno Snapshot del Workspace.
    fn list_visible_windows(
        &self,
        exclude_pid: u32,
    ) -> Result<Vec<ForegroundAppInfo>, PlatformError>;
    /// Esegue la logica completa: trova finestra o avvia processo, posiziona (compensando DWM) e richiede focus.
    fn launch_or_place_window(
        &self,
        selector: &WindowSelector,
        placement: &WindowPlacement,
        launch_cmd: Option<(&Path, &[String], Option<&Path>)>,
        timeout_ms: u64,
    ) -> Result<WindowActionResult, PlatformError>;
}

pub trait InputInjectorBackend: Send + Sync {
    /// Verifica se la finestra target in primo piano ha un livello di integrità (UIPI) superiore al nostro.
    fn can_inject_into_foreground(&self) -> bool;
    fn tap_key(&self, key_token: &str) -> Result<(), PlatformError>;
    fn chord_keys(&self, modifiers: &[String], key_token: &str) -> Result<(), PlatformError>;
    fn type_unicode_text(&self, text: &str) -> Result<(), PlatformError>;
}

pub trait AutostartBackend: Send + Sync {
    fn is_enabled(&self) -> Result<bool, PlatformError>;
    fn set_enabled(&self, enabled: bool, exe_path: &Path) -> Result<(), PlatformError>;
}
