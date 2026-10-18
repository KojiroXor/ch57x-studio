//! Diff detection: Host-Only Apply vs Hardware Flash
//!
//! Calcola SHA-256 del YAML generato e confronta con last_uploaded_yaml_hash
//! per stabilire requires_hardware_flash (Pilastro 5 & Sezione 5.2).

use sha2::{Digest, Sha256};

/// Calcola l'hash esadecimale SHA-256 della stringa YAML
pub fn compute_yaml_hash(yaml_str: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(yaml_str.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Determina se le modifiche richiedono una riscrittura USB della flash hardware
/// o se possono essere applicate immediatamente solo a livello Host (Host-Only Apply).
///
/// Se `last_uploaded_hash` e' None, significa che la tastiera non e' mai stata flashata
/// in questa sessione/macchina, quindi richiede il flash.
/// Se l'hash e' invariato, i binding HID e i trigger fisici sono identici:
/// qualsiasi modifica a percorsi eseguibili, finestre, macro o dispatch viene
/// aggiornata istantaneamente nel demone senza toccare la tastiera fisica.
pub fn requires_hardware_flash(last_uploaded_hash: Option<&str>, current_hash: &str) -> bool {
    match last_uploaded_hash {
        Some(last) => last != current_hash,
        None => true,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeploymentDecision {
    pub current_yaml_hash: String,
    pub last_uploaded_hash: Option<String>,
    pub requires_hardware_flash: bool,
}

impl DeploymentDecision {
    pub fn new(current_yaml: &str, last_uploaded_hash: Option<&str>) -> Self {
        let current_hash = compute_yaml_hash(current_yaml);
        let flash = requires_hardware_flash(last_uploaded_hash, &current_hash);
        Self {
            current_yaml_hash: current_hash,
            last_uploaded_hash: last_uploaded_hash.map(|s| s.to_string()),
            requires_hardware_flash: flash,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaml_hash_deterministic() {
        let yaml_a = "orientation: normal\nrows: 3\ncolumns: 4\nknobs: 2\n";
        let yaml_b = "orientation: normal\nrows: 3\ncolumns: 4\nknobs: 2\n";
        let yaml_c = "orientation: normal\nrows: 3\ncolumns: 4\nknobs: 1\n";

        let hash_a = compute_yaml_hash(yaml_a);
        let hash_b = compute_yaml_hash(yaml_b);
        let hash_c = compute_yaml_hash(yaml_c);

        assert_eq!(hash_a, hash_b);
        assert_ne!(hash_a, hash_c);
        assert_eq!(hash_a.len(), 64); // SHA-256 in hex
    }

    #[test]
    fn test_requires_hardware_flash_logic() {
        let current_yaml = "orientation: normal\n";
        let hash = compute_yaml_hash(current_yaml);

        // Nessun upload precedente -> richiede flash
        assert!(requires_hardware_flash(None, &hash));

        // Upload precedente diverso -> richiede flash
        assert!(requires_hardware_flash(Some("abcdef0123456789"), &hash));

        // Upload precedente identico -> NON richiede flash (Host-Only Apply)
        assert!(!requires_hardware_flash(Some(&hash), &hash));
    }

    #[test]
    fn test_deployment_decision() {
        let yaml = "test yaml config";
        let hash = compute_yaml_hash(yaml);

        let decision_first = DeploymentDecision::new(yaml, None);
        assert!(decision_first.requires_hardware_flash);
        assert_eq!(decision_first.current_yaml_hash, hash);

        let decision_cached = DeploymentDecision::new(yaml, Some(&hash));
        assert!(!decision_cached.requires_hardware_flash);
    }
}
