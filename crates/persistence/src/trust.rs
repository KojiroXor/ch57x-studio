//! Local trust store for executable hashes and paths

use crate::errors::PersistenceError;
use crate::storage::atomic_write_file;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

pub const TRUST_STORE_FILE_NAME: &str = "trust_store.json";

/// Computes the SHA-256 hash of a file on disk, formatted as lowercase hex string.
pub fn compute_file_sha256(path: &Path) -> Result<String, PersistenceError> {
    if !path.is_file() {
        return Err(PersistenceError::CorruptedData {
            path: path.to_path_buf(),
            message: "Target path does not exist or is not a regular file".into(),
        });
    }

    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

/// Computes the SHA-256 hash of in-memory bytes, formatted as lowercase hex string.
pub fn compute_bytes_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustStore {
    pub approved_sha256_hashes: HashSet<String>,
    pub path_to_hash: HashMap<PathBuf, String>,
    #[serde(skip)]
    pub storage_path: Option<PathBuf>,
}

impl TrustStore {
    pub fn default_store_path(storage_dir: &Path) -> PathBuf {
        storage_dir.join(TRUST_STORE_FILE_NAME)
    }

    /// Loads the trust store from the given storage directory, binding it to auto-save on change.
    pub fn open_or_default(storage_dir: &Path) -> Result<Self, PersistenceError> {
        let path = Self::default_store_path(storage_dir);
        let mut store = if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let s: Self = serde_json::from_str(&content)?;
            s
        } else {
            Self::default()
        };
        store.storage_path = Some(path);
        Ok(store)
    }

    /// Loads the trust store without binding a storage path.
    pub fn load_or_default(storage_dir: &Path) -> Result<Self, PersistenceError> {
        let path = Self::default_store_path(storage_dir);
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&path)?;
        let store: Self = serde_json::from_str(&content)?;
        Ok(store)
    }

    /// Atomically persists the trust store to the specified file path.
    pub fn save_to_file(&self, path: &Path) -> Result<(), PersistenceError> {
        let bytes = serde_json::to_vec_pretty(self)?;
        atomic_write_file(path, &bytes)?;
        Ok(())
    }

    /// Atomically persists the trust store to the bound storage path or the provided directory.
    pub fn save(&self, storage_dir: &Path) -> Result<(), PersistenceError> {
        let path = Self::default_store_path(storage_dir);
        self.save_to_file(&path)
    }

    fn persist_if_bound(&self) -> Result<(), PersistenceError> {
        if let Some(ref path) = self.storage_path {
            self.save_to_file(path)?;
        }
        Ok(())
    }

    /// Checks if an executable at `path` is trusted.
    ///
    /// The executable is trusted if and only if:
    /// 1. The file exists on disk.
    /// 2. Its current SHA-256 matches an entry in `approved_sha256_hashes`.
    ///
    /// If the file was replaced or tampered with on disk, its modified hash will not match
    /// and this returns `false`.
    pub fn is_trusted(&self, path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }

        match compute_file_sha256(path) {
            Ok(hash) => self.approved_sha256_hashes.contains(&hash.to_lowercase()),
            Err(_) => false,
        }
    }

    /// Checks if a raw SHA-256 hash string is approved in the store.
    pub fn is_hash_trusted(&self, hash: &str) -> bool {
        self.approved_sha256_hashes.contains(&hash.to_lowercase())
    }

    /// Approves an executable on disk by recording its SHA-256 hash and path.
    /// Persists changes immediately if the store was opened with a bound path.
    pub fn trust_path(&mut self, path: &Path) -> Result<(), PersistenceError> {
        let hash = compute_file_sha256(path)?.to_lowercase();
        self.approved_sha256_hashes.insert(hash.clone());

        let stored_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        self.path_to_hash.insert(stored_path.clone(), hash.clone());
        self.path_to_hash.insert(path.to_path_buf(), hash);

        self.persist_if_bound()?;
        Ok(())
    }

    /// Revokes trust for an executable path and its associated hash.
    /// Persists changes immediately if the store was opened with a bound path.
    pub fn revoke_path(&mut self, path: &Path) -> Result<(), PersistenceError> {
        if let Some(hash) = self.path_to_hash.remove(path) {
            self.approved_sha256_hashes.remove(&hash);
        }

        if let Ok(canon) = path.canonicalize() {
            if let Some(hash) = self.path_to_hash.remove(&canon) {
                self.approved_sha256_hashes.remove(&hash);
            }
        }

        if let Ok(hash) = compute_file_sha256(path) {
            self.approved_sha256_hashes.remove(&hash.to_lowercase());
        }

        self.persist_if_bound()?;
        Ok(())
    }

    /// Directly approves a SHA-256 hash.
    pub fn trust_hash(&mut self, hash: String) -> Result<(), PersistenceError> {
        self.approved_sha256_hashes.insert(hash.to_lowercase());
        self.persist_if_bound()?;
        Ok(())
    }

    /// Revokes trust for a raw SHA-256 hash.
    pub fn revoke_hash(&mut self, hash: &str) -> Result<(), PersistenceError> {
        self.approved_sha256_hashes.remove(&hash.to_lowercase());
        self.persist_if_bound()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use uuid::Uuid;

    #[test]
    fn test_compute_file_sha256_and_trust_verification() {
        let temp_dir = std::env::temp_dir().join(format!("ch57x_trust_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let exe_path = temp_dir.join("test_app.exe");

        // Write a mock binary file
        let mut f = File::create(&exe_path).unwrap();
        f.write_all(b"MZ\x90\x00MockExecutableBinaryContent12345")
            .unwrap();
        f.flush().unwrap();
        drop(f);

        let mut store = TrustStore::open_or_default(&temp_dir).unwrap();
        assert!(
            !store.is_trusted(&exe_path),
            "Should not be trusted initially"
        );

        // Trust the executable
        store.trust_path(&exe_path).expect("Failed to trust");
        assert!(
            store.is_trusted(&exe_path),
            "Must be trusted after approval"
        );

        let hash = compute_file_sha256(&exe_path).unwrap();
        assert!(store.is_hash_trusted(&hash));

        // Reload trust store from disk to verify auto-persistence
        let reloaded_store = TrustStore::open_or_default(&temp_dir).unwrap();
        assert!(reloaded_store.is_trusted(&exe_path));

        // Tamper with the executable (simulating modification or malware replacement)
        let mut f_tampered = File::create(&exe_path).unwrap();
        f_tampered
            .write_all(b"TamperedMaliciousExecutableBytes!")
            .unwrap();
        f_tampered.flush().unwrap();
        drop(f_tampered);

        // Tampered file MUST NOT be trusted anymore!
        assert!(
            !store.is_trusted(&exe_path),
            "Tampered executable must fail trust verification"
        );

        // Revoke trust for the path
        store.revoke_path(&exe_path).expect("Failed revoke");
        assert!(!store.is_hash_trusted(&hash));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_compute_bytes_sha256_known_vector() {
        // SHA-256 for empty string: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        let hash_empty = compute_bytes_sha256(b"");
        assert_eq!(
            hash_empty,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
