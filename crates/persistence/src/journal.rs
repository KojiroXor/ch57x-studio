//! Deployment journal state machine and crash recovery

use crate::errors::PersistenceError;
use crate::storage::atomic_write_file;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub const JOURNAL_FILE_NAME: &str = "deployment_journal.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeploymentState {
    Idle,
    ValidatingYaml,
    WritingHardwareFlash,
    ActivatingHostRuntime,
    Success,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeploymentJournal {
    pub state: DeploymentState,
    pub last_uploaded_yaml_hash: Option<String>,
    pub active_deployment_id: Option<Uuid>,
    pub updated_at_epoch_ms: u64,
}

fn current_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

impl Default for DeploymentJournal {
    fn default() -> Self {
        Self {
            state: DeploymentState::Idle,
            last_uploaded_yaml_hash: None,
            active_deployment_id: None,
            updated_at_epoch_ms: current_epoch_ms(),
        }
    }
}

impl DeploymentJournal {
    pub fn journal_path(storage_dir: &Path) -> PathBuf {
        storage_dir.join(JOURNAL_FILE_NAME)
    }

    /// Loads the journal from `storage_dir`, recovering from any in-flight crash or sudden disconnection.
    pub fn load_or_recover(storage_dir: &Path) -> Result<Self, PersistenceError> {
        let path = Self::journal_path(storage_dir);
        if !path.exists() {
            let journal = Self::default();
            journal.save(storage_dir)?;
            return Ok(journal);
        }

        let content = std::fs::read_to_string(&path)?;
        let mut journal: Self = serde_json::from_str(&content)?;

        // Crash recovery check: if an operation was interrupted mid-flight
        let needs_recovery_save = match &journal.state {
            DeploymentState::ValidatingYaml => {
                journal.state = DeploymentState::Failed(
                    "Crash or sudden exit detected during YAML validation".into(),
                );
                journal.active_deployment_id = None;
                journal.updated_at_epoch_ms = current_epoch_ms();
                true
            }
            DeploymentState::WritingHardwareFlash => {
                journal.state = DeploymentState::Failed(
                    "Crash or USB disconnection detected during hardware flash write".into(),
                );
                journal.active_deployment_id = None;
                journal.updated_at_epoch_ms = current_epoch_ms();
                true
            }
            DeploymentState::ActivatingHostRuntime => {
                journal.state = DeploymentState::Failed(
                    "Crash or sudden exit detected during host runtime activation".into(),
                );
                journal.active_deployment_id = None;
                journal.updated_at_epoch_ms = current_epoch_ms();
                true
            }
            DeploymentState::Idle | DeploymentState::Success | DeploymentState::Failed(_) => false,
        };

        if needs_recovery_save {
            journal.save(storage_dir)?;
        }

        Ok(journal)
    }

    /// Atomically persists the journal to disk.
    pub fn save(&self, storage_dir: &Path) -> Result<(), PersistenceError> {
        let path = Self::journal_path(storage_dir);
        let bytes = serde_json::to_vec_pretty(self)?;
        atomic_write_file(&path, &bytes)?;
        Ok(())
    }

    /// Transitions to a new state if valid according to the state machine contract.
    pub fn transition_to(
        &mut self,
        next: DeploymentState,
        storage_dir: Option<&Path>,
    ) -> Result<(), PersistenceError> {
        let is_valid = match (&self.state, &next) {
            // From Idle, start deployment
            (DeploymentState::Idle, DeploymentState::ValidatingYaml) => true,

            // From ValidatingYaml, proceed to Hardware Flash or Host-Only activation, or Fail
            (DeploymentState::ValidatingYaml, DeploymentState::WritingHardwareFlash) => true,
            (DeploymentState::ValidatingYaml, DeploymentState::ActivatingHostRuntime) => true,
            (DeploymentState::ValidatingYaml, DeploymentState::Failed(_)) => true,

            // From WritingHardwareFlash, proceed to Host Runtime activation or Fail
            (DeploymentState::WritingHardwareFlash, DeploymentState::ActivatingHostRuntime) => true,
            (DeploymentState::WritingHardwareFlash, DeploymentState::Failed(_)) => true,

            // From ActivatingHostRuntime, proceed to Success or Fail
            (DeploymentState::ActivatingHostRuntime, DeploymentState::Success) => true,
            (DeploymentState::ActivatingHostRuntime, DeploymentState::Failed(_)) => true,

            // From Success, can reset to Idle or start another deployment directly
            (DeploymentState::Success, DeploymentState::Idle) => true,
            (DeploymentState::Success, DeploymentState::ValidatingYaml) => true,

            // From Failed, can reset to Idle or retry
            (DeploymentState::Failed(_), DeploymentState::Idle) => true,
            (DeploymentState::Failed(_), DeploymentState::ValidatingYaml) => true,

            _ => false,
        };

        if !is_valid {
            return Err(PersistenceError::InvalidStateTransition {
                from: self.state.clone(),
                to: next,
            });
        }

        self.state = next;
        self.updated_at_epoch_ms = current_epoch_ms();

        if let Some(dir) = storage_dir {
            self.save(dir)?;
        }

        Ok(())
    }

    /// Begins a new deployment transaction.
    pub fn start_deployment(
        &mut self,
        storage_dir: Option<&Path>,
    ) -> Result<Uuid, PersistenceError> {
        let deployment_id = Uuid::new_v4();
        self.active_deployment_id = Some(deployment_id);
        self.transition_to(DeploymentState::ValidatingYaml, storage_dir)?;
        Ok(deployment_id)
    }

    /// Advances to WritingHardwareFlash.
    pub fn step_to_hardware_flash(
        &mut self,
        storage_dir: Option<&Path>,
    ) -> Result<(), PersistenceError> {
        self.transition_to(DeploymentState::WritingHardwareFlash, storage_dir)
    }

    /// Advances to ActivatingHostRuntime, recording the newly uploaded YAML hash if a flash occurred.
    pub fn step_to_activating_runtime(
        &mut self,
        uploaded_yaml_hash: Option<String>,
        storage_dir: Option<&Path>,
    ) -> Result<(), PersistenceError> {
        if let Some(hash) = uploaded_yaml_hash {
            self.last_uploaded_yaml_hash = Some(hash);
        }
        self.transition_to(DeploymentState::ActivatingHostRuntime, storage_dir)
    }

    /// Marks deployment as successful and clears active deployment id.
    pub fn step_to_success(&mut self, storage_dir: Option<&Path>) -> Result<(), PersistenceError> {
        self.active_deployment_id = None;
        self.transition_to(DeploymentState::Success, storage_dir)
    }

    /// Marks deployment as failed with error message and clears active deployment id.
    pub fn step_to_failed(
        &mut self,
        error_message: String,
        storage_dir: Option<&Path>,
    ) -> Result<(), PersistenceError> {
        self.active_deployment_id = None;
        self.transition_to(DeploymentState::Failed(error_message), storage_dir)
    }

    /// Resets the journal back to Idle state.
    pub fn reset_to_idle(&mut self, storage_dir: Option<&Path>) -> Result<(), PersistenceError> {
        self.active_deployment_id = None;
        self.transition_to(DeploymentState::Idle, storage_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_journal_full_hardware_deployment_lifecycle() {
        let temp_dir = std::env::temp_dir().join(format!("ch57x_journal_hw_{}", Uuid::new_v4()));
        let mut journal =
            DeploymentJournal::load_or_recover(&temp_dir).expect("Failed to initialize journal");
        assert_eq!(journal.state, DeploymentState::Idle);

        // 1. Start deployment
        let dep_id = journal
            .start_deployment(Some(&temp_dir))
            .expect("Failed to start");
        assert_eq!(journal.state, DeploymentState::ValidatingYaml);
        assert_eq!(journal.active_deployment_id, Some(dep_id));

        // 2. Hardware flash
        journal
            .step_to_hardware_flash(Some(&temp_dir))
            .expect("Failed step flash");
        assert_eq!(journal.state, DeploymentState::WritingHardwareFlash);

        // 3. Activating host runtime with yaml hash
        let hash = "a1b2c3d4e5f6".to_string();
        journal
            .step_to_activating_runtime(Some(hash.clone()), Some(&temp_dir))
            .expect("Failed step runtime");
        assert_eq!(journal.state, DeploymentState::ActivatingHostRuntime);
        assert_eq!(journal.last_uploaded_yaml_hash, Some(hash));

        // 4. Success
        journal
            .step_to_success(Some(&temp_dir))
            .expect("Failed step success");
        assert_eq!(journal.state, DeploymentState::Success);
        assert_eq!(journal.active_deployment_id, None);

        // Reload from disk to verify persistence
        let reloaded = DeploymentJournal::load_or_recover(&temp_dir).expect("Failed reload");
        assert_eq!(reloaded.state, DeploymentState::Success);
        assert_eq!(
            reloaded.last_uploaded_yaml_hash,
            Some("a1b2c3d4e5f6".into())
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_journal_host_only_deployment_lifecycle() {
        let temp_dir = std::env::temp_dir().join(format!("ch57x_journal_host_{}", Uuid::new_v4()));
        let mut journal = DeploymentJournal::load_or_recover(&temp_dir).expect("Failed to init");

        journal
            .start_deployment(Some(&temp_dir))
            .expect("Failed to start");
        assert_eq!(journal.state, DeploymentState::ValidatingYaml);

        // Host-only jumps directly to ActivatingHostRuntime without writing hardware flash
        journal
            .step_to_activating_runtime(None, Some(&temp_dir))
            .expect("Failed host-only runtime");
        assert_eq!(journal.state, DeploymentState::ActivatingHostRuntime);

        journal
            .step_to_success(Some(&temp_dir))
            .expect("Failed success");
        assert_eq!(journal.state, DeploymentState::Success);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_invalid_state_transition_is_blocked() {
        let mut journal = DeploymentJournal::default();
        // Cannot jump directly from Idle to Success
        let err = journal
            .transition_to(DeploymentState::Success, None)
            .unwrap_err();
        match err {
            PersistenceError::InvalidStateTransition { from, to } => {
                assert_eq!(from, DeploymentState::Idle);
                assert_eq!(to, DeploymentState::Success);
            }
            other => panic!("Expected InvalidStateTransition, got {:?}", other),
        }
    }

    #[test]
    fn test_crash_recovery_during_hardware_flash() {
        let temp_dir = std::env::temp_dir().join(format!("ch57x_journal_crash_{}", Uuid::new_v4()));
        let mut journal = DeploymentJournal::load_or_recover(&temp_dir).expect("Failed to init");

        journal
            .start_deployment(Some(&temp_dir))
            .expect("Failed start");
        journal
            .step_to_hardware_flash(Some(&temp_dir))
            .expect("Failed step flash");

        // Simulate crash: process abruptly terminated while in WritingHardwareFlash
        // Next process launch loads the journal:
        let recovered = DeploymentJournal::load_or_recover(&temp_dir).expect("Failed recovery");
        match recovered.state {
            DeploymentState::Failed(msg) => {
                assert!(msg.contains("Crash or USB disconnection detected"));
            }
            other => panic!("Expected Failed state after crash, got {:?}", other),
        }
        assert_eq!(recovered.active_deployment_id, None);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
