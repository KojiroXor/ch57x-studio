//! OS standard directories and atomic file writing

use crate::errors::PersistenceError;
use ch57x_domain::Profile;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Returns the base directory for app data.
/// Uses OS standard (%APPDATA%\ch57x-studio on Windows, $XDG_DATA_HOME/ch57x-studio on Linux),
/// with fallback to local `./data` if standard directory is not resolvable.
pub fn get_app_data_dir() -> PathBuf {
    dirs::data_dir()
        .map(|p| p.join("ch57x-studio"))
        .unwrap_or_else(|| PathBuf::from("./data"))
}

/// Returns the profiles directory inside the app data directory or custom directory.
pub fn get_profiles_dir(custom_dir: Option<&Path>) -> PathBuf {
    match custom_dir {
        Some(d) => d.join("profiles"),
        None => get_app_data_dir().join("profiles"),
    }
}

/// Atomically writes content to `target_path`.
///
/// 1. Ensures parent directory exists.
/// 2. Writes to a temporary file (`.tmp`) in the exact same directory (ensures same volume).
/// 3. Flushes and syncs data to disk.
/// 4. Atomically replaces `target_path` via `std::fs::rename`.
pub fn atomic_write_file(target_path: &Path, content: &[u8]) -> Result<(), PersistenceError> {
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file_stem = target_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");
    let tmp_filename = format!("{}.{}.tmp", file_stem, Uuid::new_v4());
    let tmp_path = target_path.with_file_name(tmp_filename);

    let write_res = (|| -> Result<(), std::io::Error> {
        let mut file = File::create(&tmp_path)?;
        file.write_all(content)?;
        file.sync_all()?;
        Ok(())
    })();

    if let Err(e) = write_res {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(PersistenceError::Io(e));
    }

    if let Err(e) = std::fs::rename(&tmp_path, target_path) {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(PersistenceError::Io(e));
    }

    Ok(())
}

/// Validates and atomically saves a `Profile` to the specified file path.
pub fn save_profile_to_path(profile: &Profile, target_path: &Path) -> Result<(), PersistenceError> {
    ch57x_domain::validate_profile(profile)?;
    let json_bytes = serde_json::to_vec_pretty(profile)?;
    atomic_write_file(target_path, &json_bytes)?;
    Ok(())
}

/// Validates and atomically saves a `Profile` in the standard profiles folder (`<id>.json`).
pub fn save_profile(
    profile: &Profile,
    custom_dir: Option<&Path>,
) -> Result<PathBuf, PersistenceError> {
    let profiles_dir = get_profiles_dir(custom_dir);
    let target_path = profiles_dir.join(format!("{}.json", profile.id));
    save_profile_to_path(profile, &target_path)?;
    Ok(target_path)
}

/// Loads and validates a `Profile` from a file path.
pub fn load_profile_from_path(target_path: &Path) -> Result<Profile, PersistenceError> {
    if !target_path.exists() {
        return Err(PersistenceError::ProfileNotFound(
            target_path.display().to_string(),
        ));
    }

    let content = std::fs::read_to_string(target_path)?;
    let profile: Profile = serde_json::from_str(&content)?;
    ch57x_domain::validate_profile(&profile)?;
    Ok(profile)
}

/// Loads a `Profile` by ID from the standard profiles folder.
pub fn load_profile(id: &Uuid, custom_dir: Option<&Path>) -> Result<Profile, PersistenceError> {
    let profiles_dir = get_profiles_dir(custom_dir);
    let target_path = profiles_dir.join(format!("{}.json", id));
    load_profile_from_path(&target_path)
}

/// Lists all valid profiles in the profiles directory.
pub fn list_profiles(custom_dir: Option<&Path>) -> Result<Vec<Profile>, PersistenceError> {
    let profiles_dir = get_profiles_dir(custom_dir);
    if !profiles_dir.exists() {
        return Ok(Vec::new());
    }

    let mut profiles = Vec::new();
    for entry in std::fs::read_dir(profiles_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            if let Ok(profile) = load_profile_from_path(&path) {
                profiles.push(profile);
            }
        }
    }

    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(profiles)
}

/// Deletes a `Profile` by ID from the profiles directory.
pub fn delete_profile(id: &Uuid, custom_dir: Option<&Path>) -> Result<(), PersistenceError> {
    let profiles_dir = get_profiles_dir(custom_dir);
    let target_path = profiles_dir.join(format!("{}.json", id));
    if target_path.exists() {
        std::fs::remove_file(target_path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ch57x_domain::Profile;

    #[test]
    fn test_atomic_write_and_load_profile_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!("ch57x_test_{}", Uuid::new_v4()));
        let profile = Profile::default();
        let profile_id = profile.id;

        // Save profile
        let saved_path = save_profile(&profile, Some(&temp_dir)).expect("Failed to save profile");
        assert!(saved_path.exists(), "Profile file must exist on disk");

        // Load profile by path
        let loaded_from_path = load_profile_from_path(&saved_path).expect("Failed to load by path");
        assert_eq!(loaded_from_path.id, profile_id);
        assert_eq!(loaded_from_path.name, profile.name);

        // Load profile by ID
        let loaded_by_id =
            load_profile(&profile_id, Some(&temp_dir)).expect("Failed to load by id");
        assert_eq!(loaded_by_id.id, profile_id);

        // List profiles
        let profiles = list_profiles(Some(&temp_dir)).expect("Failed to list profiles");
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].id, profile_id);

        // Delete profile
        delete_profile(&profile_id, Some(&temp_dir)).expect("Failed to delete profile");
        assert!(!saved_path.exists(), "Profile file must be deleted");

        // Clean up temp dir
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_atomic_overwrite_preserves_integrity() {
        let temp_dir = std::env::temp_dir().join(format!("ch57x_test_ow_{}", Uuid::new_v4()));
        let mut profile = Profile {
            name: "Version 1".into(),
            ..Default::default()
        };

        let path = save_profile(&profile, Some(&temp_dir)).expect("Failed to save V1");
        let v1 = load_profile_from_path(&path).expect("Failed to load V1");
        assert_eq!(v1.name, "Version 1");

        // Overwrite with Version 2
        profile.name = "Version 2".into();
        save_profile(&profile, Some(&temp_dir)).expect("Failed to save V2");

        let v2 = load_profile_from_path(&path).expect("Failed to load V2");
        assert_eq!(v2.name, "Version 2");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_saving_invalid_profile_fails() {
        let temp_dir = std::env::temp_dir().join(format!("ch57x_test_inv_{}", Uuid::new_v4()));
        let mut profile = Profile::default();
        profile.slots.remove(&0); // Invalidate profile

        let result = save_profile(&profile, Some(&temp_dir));
        assert!(result.is_err(), "Saving an invalid profile must fail");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
