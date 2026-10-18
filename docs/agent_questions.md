# CH57x Studio — Registro Domande Master Agent & Decisioni Utente

## Stato Corrente
- **Fase:** Wave 0 (Discovery & Setup Workspace) completata / Avvio Wave 1
- **Data Inizio:** 18/10/2026

---

## Stato Risoluzioni Wave 0 (Ambiente, Toolchain & Hardware)

### 1. Toolchain Rust
- **Esito:** Installato `rustup` via `winget`. `rustc 1.99.1` e `cargo 1.99.1` funzionanti.
- **Stato:** [RISOLTO]

### 2. Binario `ch57x-keyboard-tool`
- **Esito:** Scaricata release ufficiale v1.7.0 `kriomant/ch57x-keyboard-tool` (x86_64-pc-windows-msvc) in `bin/ch57x-keyboard-tool.exe`. Test `show-keys` e `validate` superati.
- **Stato:** [RISOLTO]

### 3. Repository Remota GitHub
- **Esito:** Creata repo privata `https://github.com/KojiroXor/ch57x-studio` e primo commit effettuato su branch `main`.
- **Stato:** [RISOLTO]

### 4. Macropad fisico e VID/PID
- **Rilevamento:**
  - Cavo USB collegato: `USB\VID_1189&PID_8890` (`Status: OK, Present: True`).
  - Bluetooth Low Energy attivo: `BTHLE\DEV_F83B26262312` (`Status: OK, Present: True`).
- **Stato:** [RISOLTO / RILEVATO]

### 5. Driver USB / UsbDk su Windows
- **Rilevamento:** Servizio `UsbDk` non ancora installato.
- **Decisione Utente:** Procedere allo sviluppo software, contratti, compilatore e runtime (Wave 1-3). Il flash USB reale e l'eventuale installazione UsbDk sono rimandati a Wave 4 (Gate 4).
- **Stato:** [ACCORDATO]
