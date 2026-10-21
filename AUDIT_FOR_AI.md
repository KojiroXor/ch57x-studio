# 🤖 VortexDeck (CH57x Studio) — Master Architectural Audit & AI Handover Dossier

> **Documento di Handover Tecnico e Audit Architetturale per Agenti AI e Ingegneri di Sistema.**
> Questo documento fornisce una radiografia completa dell'architettura del software, i contratti invarianti non negoziabili, i problemi storici già risolti e la roadmap dettagliata con aree di miglioramento prioritario.

---

## 📑 Sommario

1. [Visione Generale e Filosofia Dual-Engine](#1-visione-generale-e-filosofia-dual-engine)
2. [Mappa dei Crate e Dipendenze](#2-mappa-dei-crate-e-dipendenze)
3. [Invarianti di Sistema Non Negoziabili (Golden Rules)](#3-invarianti-di-sistema-non-negoziabili-golden-rules)
4. [Dettaglio Tecnico dei Crate](#4-dettaglio-tecnico-dei-crate)
5. [Trappole Tecniche Storiche Risolte (Lessons Learned)](#5-trappole-tecniche-storiche-risolte-lessons-learned)
6. [Stato dei Test e Pipeline CI/CD](#6-stato-dei-test-e-pipeline-cicd)
7. [Backlog di Miglioramento per la Prossima AI (Actionable Tasks)](#7-backlog-di-miglioramento-per-la-prossima-ai-actionable-tasks)

---

## 1. Visione Generale e Filosofia Dual-Engine

VortexDeck è progettato per superare i limiti intrinseci dei macropad programmabili basati su chip **WCH CH57x** (12 tasti meccanici disposti in griglia 4x3 + 2 encoder rotativi con push button su 3 layer hardware = **54 slot programmabili**).

### Il Principio Dual-Engine:
```
┌────────────────────────────────────────────────────────────────────────┐
│                        Macropad Fisico (CH57x)                         │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ Pressione Tasto / Rotazione
                                    ▼
                 ┌──────────────────────────────────────┐
                 │  Tipo di Binding nello Slot (Profilo)│
                 └───────┬──────────────────────┬───────┘
                         │                      │
       [SlotBinding::HidNative]          [SlotBinding::HostActionRef]
                         │                      │
                         ▼                      ▼
           ┌───────────────────────────┐   ┌────────────────────────────┐
           │     Flash Hardware USB    │   │  Trigger F13..F24 (Host)   │
           │(ch57x-keyboard-tool flash)│   │  (Intercettato da Win32)   │
           └─────────────┬─────────────┘   └─────────────┬──────────────┘
                         │                               │
                         ▼                               ▼
                 Esecuzione Standalone             Esecuzione Host
                 (Senza software attivo)           (VortexDeck Runtime)
                 - Tastiera USB                    - Tiling Finestre DWM
                 - Consumer Media                  - Restore Multi-Monitor
                 - Mouse Click                     - Macro Asincrone
                                                   - Luminosità WMI / Gamma
```

1. **Modalità Standalone (Hardware Flash)**: Genera un mapping YAML byte-exact per `ch57x-keyboard-tool` e lo scrive nella memoria flash del chip. Quando la tastiera è collegata via Bluetooth o a un altro computer senza VortexDeck installato, i comandi HID nativi continuano a funzionare autonomamente.
2. **Modalità Automazione Host (Runtime Engine)**: Assegna ai tasti speciali "PhantomTriggers" (combinazioni Win32 F13..F24 su 5 banchi modificatori). Un thread dedicato intercetta l'hotkey a livello di sistema operativo senza sottrarre il focus e delega l'azione agli attori runtime (spostamento finestre, restore workspace, sequenze macro, brightness WMI).

---

## 2. Mappa dei Crate e Dipendenze

La cartella di lavoro è organizzata come un workspace Cargo a 8 crate strettamente disaccoppiati:

```
crates/
├── domain/       # Pure Domain (Zero IO, zero platform). Tipi, 54 slot, validatore.
├── platform/     # Trait di astrazione OS + Implementazione Win32 e stub Linux.
├── hardware/     # Wrapper CLI per ch57x-keyboard-tool + Probe USB PnP VID 0x1189.
├── compiler/     # Allocatore deterministico trigger F13..F24 + Generatore YAML byte-exact.
├── persistence/  # Atomic file storage (.tmp -> rename), Journal transazionale, Trust store.
├── runtime/      # Dispatcher eventi, WindowActor, WorkspaceActor, MacroActor, ContextActor.
├── ui/           # Interfaccia egui Liquid Glass, visual canvas, inspector, preset 1-click.
└── app/          # Main binary (vortexdeck), CLI flags (--doctor, --background), System Tray.
```

### Grafo delle Dipendenze
- `ch57x-domain` non dipende da nessun crate interno.
- `ch57x-platform`, `ch57x-hardware`, `ch57x-compiler`, `ch57x-persistence` dipendono **solo** da `ch57x-domain`.
- `ch57x-runtime` dipende da `ch57x-domain` e `ch57x-platform`.
- `ch57x-ui` dipende da `ch57x-domain`, `ch57x-hardware`, `ch57x-platform`.
- `ch57x-studio` (app) coordina tutti i crate e gestisce il ciclo di vita.

---

## 3. Invarianti di Sistema Non Negoziabili (Golden Rules)

Quando modifichi o espandi il codice, **NON violare mai queste regole**:

1. **54 Slot Fisici Esatti**:
   - Un Layer ha 12 tasti e 2 encoder (ognuno con 3 eventi: CCW, Press, CW) = 18 slot.
   - 3 Layer = **54 slot complessivi**. Non aggiungere né eliminare indici slot al di fuori di `0..53`.
   - Contratto: [`crates/domain/src/hardware.rs`](crates/domain/src/hardware.rs).

2. **60 Candidati PhantomTrigger**:
   - F13..F24 distribuiti su 5 banchi:
     - Banco A (12 tasti): `F13`..`F24` (nessun modificatore)
     - Banco B (12 tasti): `Ctrl+F13`..`Ctrl+F24`
     - Banco C (12 tasti): `Shift+F13`..`Shift+F24`
     - Banco D (12 tasti): `Alt+F13`..`Alt+F24`
     - Banco E (12 tasti): `Ctrl+Shift+F13`..`Ctrl+Shift+F24`
   - Contratto: [`crates/domain/src/triggers.rs`](crates/domain/src/triggers.rs).

3. **Scrittura Atomica e Sicura su Disco**:
   - Non salvare mai direttamente sul file JSON di destinazione. Scrivi sempre prima su un file temporaneo (`.tmp`) nella stessa directory e usa `std::fs::rename` per garantire la sostituzione atomica a livello di file system.
   - Contratto: [`crates/persistence/src/storage.rs`](crates/persistence/src/storage.rs).

4. **Nessun Pop-up di Finestra Console su Windows**:
   - Qualsiasi esecuzione di comandi esterni (`ch57x-keyboard-tool`, powershell per WMI, comandi utente) deve specificare la flag Win32 `creation_flags(0x08000000)` (`CREATE_NO_WINDOW`) per evitare sfarfallii visivi della console sul desktop dell'utente.

5. **Compensazione Ombre DWM per Finestre**:
   - In Windows 10/11, `GetWindowRect` include le ombre trasparenti invisibili del compositore DWM (circa 7-8 pixel per lato). Per posizionare accuratamente una finestra, devi sempre interrogare `DwmGetWindowAttribute(hwnd, DWMWA_EXTENDED_FRAME_BOUNDS)` e applicare l'offset calcolato.
   - Contratto: [`crates/platform/src/windows/windows_api.rs`](crates/platform/src/windows/windows_api.rs).

---

## 4. Dettaglio Tecnico dei Crate

### `ch57x-domain`
- **Tipi Chiave**:
  - `SlotId`: coordinate layer (0..2), tipo di controllo (`Button { row, col }` o `Encoder { index, event }`).
  - `Profile`: struttura contenente metadati, `slots: HashMap<usize, SlotConfig>` e `actions: HashMap<ActionId, ActionDefinition>`.
  - `ActionDefinition`: enum polimorfico che supporta:
    - `LaunchOrFocus`: porta in primo piano o lancia un eseguibile con posizionamento/snap geometrico.
    - `RestoreWorkspace`: cattura e ripristina la disposizione di molteplici finestre su monitor diversi.
    - `MacroSequence`: esecuzione di sequenze di tasti/testo con focus guard e rollback clipboard.
    - `ExecProcess`: lancio processi in background con argomenti opzionali.
    - `ContextDispatch`: dispatch contestuale basato sull'eseguibile attivo in foreground.
    - `ShowHud`: overlay OSD su schermo.
    - `AdjustBrightness { delta: i32 }`: regolazione luminosità monitor.
- **Validazione**: `validate_profile(&Profile)` assicura l'integrità referenziale a tempo zero prima del salvataggio o deployment.

### `ch57x-platform`
- **Win32 Message Loop**: `hotkeys.rs` gira su un thread dedicato con un proprio ciclo `GetMessageW`, evitando qualsiasi interruzione all'event loop grafico di egui.
- **Controllo UIPI**: `input.rs` interroga il token di sicurezza (`TokenIntegrityLevel`) del processo target per avvisare se un'azione non può essere iniettata a causa della User Interface Privilege Isolation (es. task manager o finestre con privilegi di amministratore).
- **Motore Luminosità Nativo**: `brightness.rs` implementa cache atomica in memoria a 0 ms con debouncing e dispatch asincrono su WMI (per display integrati laptop/Surface) e fallback Gamma Ramp su GDI (per display desktop esterni).

### `ch57x-compiler`
- **Diffing Intelligente**: Prima di tentare la scrittura USB su flash (operazione lenta e delicata), `diff.rs` calcola l'hash SHA-256 del nuovo documento YAML e lo confronta con `last_uploaded_yaml_hash` salvato nel journal di persistenza. Se il documento non è cambiato, il flash hardware viene saltato istantaneamente.

### `ch57x-ui`
- **Liquid Glass Canvas**: Disegno vettoriale personalizzato con pipeline a strati:
  1. Base acrilica semi-trasparente (`rgba(18, 18, 22, 220)`).
  2. Bordo di illuminazione speculare superiore (`rgba(255, 255, 255, 38)`).
  3. Bordo scuro inferiore di contatto (`rgba(0, 0, 0, 70)`).
  4. Badge chip per tipologia binding (`HID`, `WIN`, `WSP`, `MAC`, `CMD`, `CTX`, `LUM`, `DIS`).
- **Drag & Drop Engine**: Permette lo scambio atomico tra qualsiasi coppia di slot della matrice con rendering di una finestra fantasma translucida che segue il cursore.
- **Libreria Preset 1-Click**: Modulo `presets.rs` contenente i preset standardizzati per encoder 3-in-1, chip di produttività per singoli tasti e template per interi layer.

---

## 5. Trappole Tecniche Storiche Risolte (Lessons Learned)

Se incontri problemi simili in futuro, ecco le cause radice già identificate e risolte:

| Problema Incontrato | Causa Radice | Soluzione Architetturale Adottata |
| :--- | :--- | :--- |
| **I tasti di luminosità non funzionavano su Windows** | `macbrightnessup/down` nel tool hardware sono alias per `Pause` e `ScrollLock`. Windows non controlla la retroilluminazione con quei tasti. | Creata l'azione `ActionDefinition::AdjustBrightness` pilotata via host tramite WMI (`WmiSetBrightness`) e GDI (`SetDeviceGammaRamp`). |
| **Crash linker MinGW locale (`dlltool` mancante)** | L'installazione locale di Rust MinGW non trovava gli eseguibili GNU standard nel PATH. | Configurato `rustup default stable-x86_64-pc-windows-gnu` e impostati i wrapper di build locali appropriati. |
| **GitHub Actions runner fallito su Linux** | Il file `rust-toolchain.toml` specificava un target tuple fisso per MinGW Windows che mandava in errore il runner Ubuntu. | Rimosso `rust-toolchain.toml` dal tracking Git e target-gated i warning specifici Linux via `#[cfg]`. |
| **Sfarfallio e finestre nere cmd popup** | `std::process::Command` senza flag specifiche apre una console Windows quando invocato da processi GUI. | Aggiunta sistematica di `creation_flags(0x08000000)` (`CREATE_NO_WINDOW`) su tutti i comandi di sistema. |
| **Finestra posizionata con 8px di errore** | `GetWindowRect` include le ombre DWM invisibili su Windows 10 e 11. | Utilizzo esclusivo di `DwmGetWindowAttribute(hwnd, DWMWA_EXTENDED_FRAME_BOUNDS)` per calcolare l'offset esatto. |

---

## 6. Stato dei Test e Pipeline CI/CD

- **Suite di Test**: 75+ test unitari e di integrazione passati al 100% in tutto il workspace:
  - `ch57x-domain`: 11 test passati.
  - `ch57x-platform`: 11 test passati.
  - `ch57x-hardware`: 6 test passati.
  - `ch57x-compiler`: 11 test passati (inclusa validazione YAML reale contro il binario `ch57x-keyboard-tool`).
  - `ch57x-persistence`: 9 test passati.
  - `ch57x-runtime`: 9 test passati.
  - `ch57x-ui`: 18 test passati.
- **Linter & Formatting**:
  - `cargo fmt --all -- --check`: 100% compliant.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warning su Windows e Linux.
- **Pipeline CI**: Workflow GitHub Actions `.github/workflows/ci.yml` con 5 job paralleli (lint, check, test suite cross-platform, build release e packaging dell'artefatto `VortexDeck-Windows-x64`).

---

## 7. Backlog di Miglioramento per la Prossima AI (Actionable Tasks)

Se vieni incaricato di espandere ulteriormente VortexDeck, ecco i compiti a più alto valore aggiunto, già analizzati tecnicamente:

### Task A: Telemetria Batteria Bluetooth BLE Diretta
- **Obiettivo**: Mostrare la percentuale esatta di carica della batteria e lo stato di ricarica nell'angolo superiore della GUI quando il dispositivo è connesso via Bluetooth.
- **Implementazione Suggerita**:
  - Implementare in `crates/platform/src/windows/` un modulo `battery_probe.rs` che usa le API Windows Runtime (`windows::Devices::Bluetooth::GenericAttributeProfile`).
  - Connettersi al GATT Standard Battery Service (UUID `0x180F`), leggere la Characteristic `Battery Level` (UUID `0x2A19`).
  - Iniettare il dato nello stato dell'interfaccia `UiState` ed esporlo con un indicatore a barra colorata nella toolbar principale.

### Task B: Snapshot e Backup Automatico del Firmware Flash
- **Obiettivo**: Prima di invocare `ch57x-keyboard-tool upload`, creare una copia di backup dell'attuale configurazione hardware per consentire il rollback in 1-click in caso di errore.
- **Implementazione Suggerita**:
  - Estendere `crates/hardware/src/cli_adapter.rs` per supportare il comando di lettura (dump) se supportato dall'endpoint o memorizzare la cronologia dei file YAML inviati con timestamp in `~/.vortexdeck/backups/`.
  - Aggiungere un pulsante "Ripristina Ultimo Firmware Funzionante" nel pannello Doctor.

### Task C: Rendering Icone Vettoriali sui Tasti (SVG Glyphs)
- **Obiettivo**: Consentire all'utente di associare icone grafiche stilizzate (es. icona terminale, forbici, play/pausa, browser, cartella) oltre alle semplici etichette di testo.
- **Implementazione Suggerita**:
  - Aggiungere in `crates/ui/src/` una libreria di icone vettoriali compatti (usando `egui_extras` con `svg` o font di icone incorporate tipo Phosphor / Feather).
  - In `pad_canvas.rs`, se uno slot ha un'icona assegnata, disegnarla al centro del keycap con blending proporzionale al colore scelto per il tasto.

### Task D: Sincronizzazione e Profili Multipli / Esportazione Portabile
- **Obiettivo**: Consentire di salvare, esportare e importare profili completi (es. "Profilo Montaggio Video Premiere", "Profilo Coding Rust/VSCode", "Profilo Gaming/Streaming") come singoli file `.vortexdeck-bundle` condivisibili.
- **Implementazione Suggerita**:
  - Estendere `crates/persistence/` con una funzione di import/export che include il mapping del profilo, le icone e i metadati in formato JSON compresso o formattato con checksum SHA-256 di convalida.
  - Aggiungere nella GUI un selettore a tendina di profili rapidi nella barra superiore.

---

## 📌 Indirizzario dei File Chiave per Navigazione Rapida

- **Modello Dominio Slot & Hardware**: [`crates/domain/src/hardware.rs`](crates/domain/src/hardware.rs)
- **Trigger Virtuali F13..F24**: [`crates/domain/src/triggers.rs`](crates/domain/src/triggers.rs)
- **Azioni Host & Tipi**: [`crates/domain/src/actions.rs`](crates/domain/src/actions.rs)
- **Regolazione Luminosità Windows**: [`crates/platform/src/windows/brightness.rs`](crates/platform/src/windows/brightness.rs)
- **Compensazione Finestre Win32 DWM**: [`crates/platform/src/windows/windows_api.rs`](crates/platform/src/windows/windows_api.rs)
- **Generatore YAML Byte-Exact**: [`crates/compiler/src/yaml_gen.rs`](crates/compiler/src/yaml_gen.rs)
- **Canvas Visuale & Drag-and-Drop**: [`crates/ui/src/pad_canvas.rs`](crates/ui/src/pad_canvas.rs)
- **Libreria Preset 1-Click**: [`crates/ui/src/presets.rs`](crates/ui/src/presets.rs)
- **Workflow CI/CD GitHub**: [`.github/workflows/ci.yml`](.github/workflows/ci.yml)


---

## ⚡ Aggiornamenti Architetturali Recenti (Sessione 15)

1. **Trigger BLE-Safe (`crates/domain/src/triggers.rs`)**:
   - I chip CH57x/CH58x troncano i keycode HID virtuali `> 0x65` quando operano in modalità Bluetooth Low Energy (BLE).
   - I 60 trigger sono stati riassegnati a codici standard `< 0x65` (F1..F11, 0..9, a..z, minus) con combinazioni `Ctrl+Alt+Shift` e `Ctrl+Alt+Win`.
   - `MOD_NOREPEAT` è stato rimosso per consentire rotazione continua fluida delle manopole.

2. **Start Menu Shortcut Synchronization (`crates/platform/src/windows/shortcut_sync.rs`)**:
   - Per le azioni `ExecProcess` (come `agy` o `wt.exe`), VortexDeck genera file `.lnk` in `%APPDATA%\Microsoft\Windows\Start Menu\Programs\VortexDeck` tramite COM `IShellLinkW`.
   - L'impostazione di `SetHotkey(0x0700 | VK)` delega la registrazione della scorciatoia a Windows Explorer (`explorer.exe`). In questo modo, premendo il tasto o ruotando l'encoder, l'azione host si avvia anche quando il processo `vortexdeck.exe` è completamente terminato.

3. **Motore Luminosità a 2 Stadi (`crates/platform/src/windows/brightness.rs`)**:
   - *Stadio 1*: `CreateFileW(w!("\\\\.\\LCD"))` con `DeviceIoControl(IOCTL_VIDEO_QUERY_DISPLAY_BRIGHTNESS)` e `IOCTL_VIDEO_SET_DISPLAY_BRIGHTNESS`. Risponde in < 1ms sui pannelli interni di Microsoft Surface e notebook.
   - *Stadio 2*: Fallback asincrono WMI debounced `WmiSetBrightness` e GDI `SetDeviceGammaRamp`.
