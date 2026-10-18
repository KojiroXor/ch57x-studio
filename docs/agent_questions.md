# CH57x Studio — Registro Domande Master Agent & Decisioni Utente

## Stato Corrente
- **Fase:** Wave 0 (Discovery & Setup Workspace)
- **Data Inizio:** 18/10/2026

---

## Domande Bloccanti Wave 0 (Ambiente & Hardware)

### 1. Toolchain Rust non trovata su PATH
- **Rilevamento:** `rustc` e `cargo` non risultano installati o non sono presenti nel `PATH` utente/sistema.
- **Opzioni:**
  - A: Installare Rust tramite `winget install Rustlang.Rustup` (o installer ufficiale `rustup-init.exe`).
  - B: Se Rust è già presente in un percorso custom, indicare il percorso esatto della directory `bin`.
- **Stato:** [APERTA]

### 2. Binario `ch57x-keyboard-tool`
- **Rilevamento:** Il binario `ch57x-keyboard-tool.exe` non è presente nel PATH né nella cartella di progetto.
- **Opzioni:**
  - A: L'utente fornisce il binario inserendolo in `bin/ch57x-keyboard-tool.exe`.
  - B: Compilare / scaricare il binario dalle release upstream.
- **Stato:** [APERTA]

### 3. Driver USB / UsbDk su Windows
- **Rilevamento:** Servizio `UsbDk` non rilevato attivo.
- **Opzioni:**
  - Per il solo sviluppo software / GUI / simulazione e gestione finestre Windows (Wave 1-3), UsbDk non è necessario.
  - Per il flash USB reale (Wave 4 - Gate 4), UsbDk sarà richiesto da `ch57x-keyboard-tool`.
- **Stato:** [APERTA]

### 4. Macropad fisico e VID/PID
- **Rilevamento:**
  - Dispositivo USB registrato in cronologia PnP: `VID_1189&PID_8890` (confermato modello CH57x standard, attualmente `Present: False`, cavo USB scollegato).
  - Dispositivo Bluetooth Low Energy attualmente attivo e connesso: `BTHLE\DEV_F83B26262312` (`Status: OK, Present: True`).
- **Stato:** [RISOLTO / RILEVATO] Modalità Host-Only / Bluetooth verificabile; per flash USB servirà collegare il cavo.

