//! Diff detection: Host-Only Apply vs Hardware Flash

use sha2::{Digest, Sha256};

pub fn compute_yaml_hash(yaml_str: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(yaml_str.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn requires_hardware_flash(last_uploaded_hash: Option<&str>, current_hash: &str) -> bool {
    match last_uploaded_hash {
        Some(last) => last != current_hash,
        None => true,
    }
}
