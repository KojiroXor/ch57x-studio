//! Safe process execution wrapper for ch57x-keyboard-tool without shell
//!
//! Esegue le chiamate a validate, upload, show-keys direttamente tramite
//! `Command::new` con I/O via standard input/output. Su Windows imposta
//! `CREATE_NO_WINDOW` (0x08000000) per evitare finestre cmd popup.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use thiserror::Error;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Flag Win32 per evitare la creazione della finestra console su Windows
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("CLI binary not found at: {0}")]
    NotFound(PathBuf),

    #[error("CLI process failed with code {0:?}: {1}")]
    ProcessFailed(Option<i32>, String),

    #[error("IO error executing CLI: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct Ch57xCli {
    binary_path: PathBuf,
    vendor_id: Option<u16>,
    product_id: Option<u16>,
}

impl Ch57xCli {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            binary_path: path.as_ref().to_path_buf(),
            vendor_id: None,
            product_id: None,
        }
    }

    pub fn with_vendor_id(mut self, vid: u16) -> Self {
        self.vendor_id = Some(vid);
        self
    }

    pub fn with_product_id(mut self, pid: u16) -> Self {
        self.product_id = Some(pid);
        self
    }

    pub fn binary_path(&self) -> &Path {
        &self.binary_path
    }

    /// Rileva automaticamente la posizione del binario `ch57x-keyboard-tool`
    pub fn auto_detect() -> Result<Self, CliError> {
        let exe_name = if cfg!(target_os = "windows") {
            "ch57x-keyboard-tool.exe"
        } else {
            "ch57x-keyboard-tool"
        };

        // 1. Variabile d'ambiente esplicita
        if let Ok(env_path) = std::env::var("CH57X_TOOL_PATH") {
            let p = PathBuf::from(env_path);
            if p.exists() {
                return Ok(Self::new(p));
            }
        }

        // 2. Cartella bin/ nella directory corrente di lavoro
        let bin_folder = PathBuf::from("bin").join(exe_name);
        if bin_folder.exists() {
            return Ok(Self::new(bin_folder));
        }

        // 3. Directory dell'eseguibile corrente
        if let Ok(current_exe) = std::env::current_exe() {
            if let Some(parent) = current_exe.parent() {
                let candidate1 = parent.join(exe_name);
                if candidate1.exists() {
                    return Ok(Self::new(candidate1));
                }
                let candidate2 = parent.join("bin").join(exe_name);
                if candidate2.exists() {
                    return Ok(Self::new(candidate2));
                }
            }
        }

        // 4. PATH di sistema
        if let Ok(path_var) = std::env::var("PATH") {
            let sep = if cfg!(target_os = "windows") {
                ';'
            } else {
                ':'
            };
            for part in path_var.split(sep) {
                let candidate = PathBuf::from(part).join(exe_name);
                if candidate.exists() {
                    return Ok(Self::new(candidate));
                }
            }
        }

        Err(CliError::NotFound(bin_folder))
    }

    /// Prepara un Command con `CREATE_NO_WINDOW` su Windows e zero shell
    fn create_command(&self) -> Command {
        let mut cmd = Command::new(&self.binary_path);

        #[cfg(target_os = "windows")]
        {
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        if let Some(vid) = self.vendor_id {
            cmd.arg("--vendor-id").arg(vid.to_string());
        }
        if let Some(pid) = self.product_id {
            cmd.arg("--product-id").arg(pid.to_string());
        }

        cmd
    }

    /// Valida la configurazione YAML passata come stringa su stdin
    pub fn validate(&self, yaml_str: &str) -> Result<(), CliError> {
        if !self.binary_path.exists() {
            return Err(CliError::NotFound(self.binary_path.clone()));
        }

        let mut child = self
            .create_command()
            .arg("validate")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(yaml_str.as_bytes())?;
        }

        let output = child.wait_with_output()?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let msg = if !stderr.trim().is_empty() {
                stderr
            } else {
                stdout
            };
            Err(CliError::ProcessFailed(output.status.code(), msg))
        }
    }

    /// Carica la configurazione YAML sulla memoria flash della tastiera via stdin
    pub fn upload(&self, yaml_str: &str) -> Result<(), CliError> {
        if !self.binary_path.exists() {
            return Err(CliError::NotFound(self.binary_path.clone()));
        }

        let mut child = self
            .create_command()
            .arg("upload")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(yaml_str.as_bytes())?;
        }

        let output = child.wait_with_output()?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let msg = if !stderr.trim().is_empty() {
                stderr
            } else {
                stdout
            };
            Err(CliError::ProcessFailed(output.status.code(), msg))
        }
    }

    /// Interroga la CLI per ottenere l'elenco dei tasti e modificatori supportati
    pub fn show_keys(&self) -> Result<String, CliError> {
        if !self.binary_path.exists() {
            return Err(CliError::NotFound(self.binary_path.clone()));
        }

        let output = self
            .create_command()
            .arg("show-keys")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(CliError::ProcessFailed(output.status.code(), stderr))
        }
    }
}

/// Helper standalone per la validazione automatica
pub fn validate(yaml_str: &str) -> Result<(), CliError> {
    Ch57xCli::auto_detect()?.validate(yaml_str)
}

/// Helper standalone per l'upload
pub fn upload(yaml_str: &str) -> Result<(), CliError> {
    Ch57xCli::auto_detect()?.upload(yaml_str)
}

/// Helper standalone per l'elenco dei tasti
pub fn show_keys() -> Result<String, CliError> {
    Ch57xCli::auto_detect()?.show_keys()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find_test_binary() -> Option<PathBuf> {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let p = manifest_dir
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("bin")
            .join("ch57x-keyboard-tool.exe");
        if p.exists() {
            Some(p)
        } else {
            None
        }
    }

    #[test]
    fn test_show_keys() {
        if let Some(bin) = find_test_binary() {
            let cli = Ch57xCli::new(bin);
            let keys = cli.show_keys().expect("show_keys must succeed");
            assert!(keys.contains("Modifiers:"));
            assert!(keys.contains("ctrl"));
            assert!(keys.contains("f13"));
            assert!(keys.contains("f24"));
            assert!(keys.contains("volumeup"));
        }
    }

    #[test]
    fn test_validate_valid_yaml() {
        if let Some(bin) = find_test_binary() {
            let cli = Ch57xCli::new(bin);
            let yaml = r#"
orientation: normal
rows: 3
columns: 4
knobs: 2
layers:
  - buttons:
      - ["f13", "f14", "f15", "f16"]
      - ["f17", "f18", "f19", "f20"]
      - ["f21", "f22", "f23", "f24"]
    knobs:
      - ccw: "f24"
        press: "f24"
        cw: "f24"
      - ccw: "f24"
        press: "f24"
        cw: "f24"
  - buttons:
      - ["f24", "f24", "f24", "f24"]
      - ["f24", "f24", "f24", "f24"]
      - ["f24", "f24", "f24", "f24"]
    knobs:
      - ccw: "f24"
        press: "f24"
        cw: "f24"
      - ccw: "f24"
        press: "f24"
        cw: "f24"
  - buttons:
      - ["f24", "f24", "f24", "f24"]
      - ["f24", "f24", "f24", "f24"]
      - ["f24", "f24", "f24", "f24"]
    knobs:
      - ccw: "f24"
        press: "f24"
        cw: "f24"
      - ccw: "f24"
        press: "f24"
        cw: "f24"
"#;
            let result = cli.validate(yaml);
            assert!(result.is_ok(), "Validation should succeed: {:?}", result);
        }
    }

    #[test]
    fn test_validate_invalid_yaml() {
        if let Some(bin) = find_test_binary() {
            let cli = Ch57xCli::new(bin);
            let bad_yaml = "completely: invalid: yaml";
            let result = cli.validate(bad_yaml);
            assert!(result.is_err(), "Invalid YAML must fail");
        }
    }

    #[test]
    fn test_cli_binary_not_found() {
        let fake_cli = Ch57xCli::new("C:\\nonexistent_dir_12345\\binary.exe");
        let result = fake_cli.show_keys();
        assert!(matches!(result, Err(CliError::NotFound(_))));
    }
}
