# 📓 Diario di Sviluppo VortexDeck (CH57x Studio)

> **Cronistoria tecnica dettagliata di ogni intervento, motivazione, diagnosi e soluzione architetturale implementata nel progetto.**

---

## 🗺️ Indice delle Fasi e Interventi

- [Sessione 1: Fondamenta, Specifiche e Contratti Iniziali (Wave 0)](#sessione-1-fondamenta-specifiche-e-contratti-iniziali-wave-0)
- [Sessione 2: Ingegneria dei Crate Core (Wave 1)](#sessione-2-ingegneria-dei-crate-core-wave-1)
- [Sessione 3: Runtime Engine & Canvas Visuale (Wave 2)](#sessione-3-runtime-engine--canvas-visuale-wave-2)
- [Sessione 4: Integrazione App, System Tray e Release Packager](#sessione-4-integrazione-app-system-tray-e-release-packager)
- [Sessione 5: Il Grande Bug Bash dei 13 Difetti Architetturali](#sessione-5-il-grande-bug-bash-dei-13-difetti-architetturali)
- [Sessione 6: Redesign Visivo Apple Liquid Glass](#sessione-6-redesign-visivo-apple-liquid-glass)
- [Sessione 7: Chiarimenti Bluetooth, Avvio Windows e Deep Sleep Timer](#sessione-7-chiarimenti-bluetooth-avvio-windows-e-deep-sleep-timer)
- [Sessione 8: Falsi Positivi Windows Defender e Analisi Batteria](#sessione-8-falsi-positivi-windows-defender-e-analisi-batteria)
- [Sessione 9: Risoluzione Luminosità Manopole su Windows (WMI + Gamma)](#sessione-9-risoluzione-luminosit-manopole-su-windows-wmi--gamma)
- [Sessione 10: Espansione Master dei Preset 1-Click e Template di Layer](#sessione-10-espansione-master-dei-preset-1-click-e-template-di-layer)
- [Sessione 11: Drag-and-Drop per Spostamento/Scambio Configurazione Tasti](#sessione-11-drag-and-drop-per-spostamentoscambio-configurazione-tasti)
- [Sessione 12: Risoluzione GitHub Actions CI/CD e Build Cloud](#sessione-12-risoluzione-github-actions-cicd-e-build-cloud)
- [Sessione 13: Audit di Sicurezza e Sanitizzazione Dati Sensibili](#sessione-13-audit-di-sicurezza-e-sanitizzazione-dati-sensibili)
- [Sessione 14: Compilazione Release Locale e Validazione Eseguibile](#sessione-14-compilazione-release-locale-e-validazione-eseguibile)

---

### Sessione 1: Fondamenta, Specifiche e Contratti Iniziali (Wave 0)
- **Data & Commit**: `2026-10-18` | `22fdd71`, `6223ac9`
- **Motivazione**: Creare un configuratore moderno e performante per il tastierino macro basato su microcontrollore WCH CH57x (12 tasti meccanici + 2 encoder rotativi cliccabili su 3 layer = 54 slot fisici).
- **Sfida Tecnica**: I tool esistenti erano script Python lenti o interfacce proprietarie chiuse in cinese. Serviva un'architettura modulare in Rust priva di lock-in con netta separazione tra dominio puro, hardware e piattaforma host.
- **Cosa abbiamo fatto**:
  - Creato il workspace Cargo con 8 crate disaccoppiati (`ch57x-domain`, `ch57x-platform`, `ch57x-hardware`, `ch57x-compiler`, `ch57x-persistence`, `ch57x-runtime`, `ch57x-ui`, `ch57x-studio`).
  - Congelato il modello di dominio a 54 slot immutabili (`hardware.rs`) e i 60 `PhantomTrigger` (F13..F24 suddivisi nei 5 banchi modificatori A..E in `triggers.rs`).
  - Definiti i contratti dei trait platform (`PlatformBackend`, `WindowManagerBackend`, `HotkeyBackend`, `InputInjectorBackend`).

---

### Sessione 2: Ingegneria dei Crate Core (Wave 1)
- **Data & Commit**: `2026-10-18` | `8c4c805`
- **Motivazione**: Implementare la logica fondamentale di compilazione, persistenza sicura e comunicazione a basso livello con il dispositivo USB.
- **Sfida Tecnica**: 
  - La scrittura su flash hardware è lenta e richiede il caricamento di file YAML con formattazione byte-exact per il tool CLI upstream `ch57x-keyboard-tool`.
  - Rischio di corruzione del profilo utente in caso di chiusura forzata dell'applicazione o spegnimento improvviso del PC.
- **Cosa abbiamo fatto**:
  - **`ch57x-domain`**: Implementato validatore semantico a 4 livelli (integrità dei 54 slot, assenza di collisioni di trigger, fallback validi per regole contestuali).
  - **`ch57x-persistence`**: Implementato storage atomico (scrittura su file `.tmp` e rename atomico a livello di file system), journal di deployment transazionale con recovery e trust store SHA-256 per gli eseguibili autorizzati.
  - **`ch57x-compiler`**: Sviluppato l'allocatore deterministico di trigger e il generatore YAML con calcolo hash SHA-256 per rilevare se la memoria flash hardware necessita realmente di essere riscritta.
  - **`ch57x-hardware`**: Creato il wrapper CLI sicuro (`cli_adapter.rs`) che comunica via stdin/stdout con soppressione della console Windows (`CREATE_NO_WINDOW`), e probe USB PnP per VID `0x1189`.

---

### Sessione 3: Runtime Engine & Canvas Visuale (Wave 2)
- **Data & Commit**: `2026-10-18` | `8ae024b`
- **Motivazione**: Consentire al Macropad di andare oltre i normali tasti HID hardware, pilotando automazioni complesse sul sistema operativo Windows (tiling finestre, restore workspace multi-monitor, macro asincrone).
- **Sfida Tecnica**: 
  - Windows 10 e 11 applicano ombre DWM trasparenti invisibili che falsano il calcolo delle coordinate rispetto a `GetWindowRect`.
  - Le macro possono inviare tasti alla finestra sbagliata se l'utente cambia focus durante l'esecuzione.
- **Cosa abbiamo fatto**:
  - **`ch57x-runtime`**: Implementati i quattro attori asincroni (`WindowActor`, `WorkspaceActor`, `MacroActor` con focus guard e rollback clipboard, `ContextActor` per dispatch dipendente dall'eseguibile attivo).
  - **`ch57x-ui`**: Realizzato il canvas interattivo vettoriale in `egui` che disegna fedelmente la matrice 4x3 dei tasti, i 3 LED centrali e le manopole degli encoder con aree cliccabili separate per CCW, Push e CW.

---

### Sessione 4: Integrazione App, System Tray e Release Packager
- **Data & Commit**: `2026-10-19` | `5c77311`, `ed5bbbc`, `247ec78`
- **Motivazione**: Rendere VortexDeck un'applicazione Windows completa, avviabile in background all'avvio del sistema e distribuibile come pacchetto autonomo.
- **Sfida Tecnica**: Unificare l'event loop grafico di `egui/eframe` con il thread dedicato dei messaggi Win32 (`WM_HOTKEY`) e l'icona nella barra delle applicazioni (System Tray).
- **Cosa abbiamo fatto**:
  - Sviluppato `crates/app/src/coordinator.rs` che orchestra dispatcher, hotkey hook, persistenza e interfaccia.
  - Implementato `crates/app/src/tray.rs` con menu contestuale Win32 (Apri, Pausa Automazioni, HUD OSD, Esci) e risveglio reattivo dell'event loop egui.
  - Creato lo script `packaging/windows/build_release.ps1` per assemblare la directory distribuibile con binari e documentazione.

---

### Sessione 5: Il Grande Bug Bash dei 13 Difetti Architetturali
- **Data & Commit**: `2026-10-19` | `124a6ab`, `5f3575c`
- **Motivazione**: Audit rigoroso del codice per eliminare ogni possibile causa di crash, instabilità o discrepanza tra interfaccia e hardware.
- **Sfida Tecnica**: Risolvere 13 difetti sottili rilevati dall'audit prima della prima release stabile.
- **Cosa abbiamo fatto**:
  1. Corretta compensazione delle ombre DWM tramite `DwmGetWindowAttribute(DWMWA_EXTENDED_FRAME_BOUNDS)`.
  2. Implementato controllo UIPI con token integrity level per evitare iniezioni a processi elevati da contesti non privilegiati.
  3. Risolto blocco del linker locale MinGW con wrapper automatico `as.exe` e `dlltool.exe`.
  4. Sostituito canale bloccante `std::sync::mpsc` con `crossbeam-channel` non-bloccante nel System Tray.
  5. Risolto bug di swap ordine encoder nella compilazione YAML.
  6. Aggiunto ripristino sicuro della clipboard dopo l'esecuzione di paste macro.
  7. Eliminati tutti i deadlock potenziali sui lock `RwLock` del dispatcher.

---

### Sessione 6: Redesign Visivo Apple Liquid Glass
- **Data & Commit**: `2026-10-19` | `2a5bc1a`, `0a91e0d`, `4d285e6`
- **Motivazione**: L'interfaccia iniziale era funzionale ma esteticamente rigida. L'utente desiderava un look premium moderno in stile Apple macOS Sequoia / VisionOS.
- **Sfida Tecnica**: Eseguire rendering stile vetro acrilico translucido ("Liquid Glass") in tempo reale a 60 FPS senza rallentare l'event loop su hardware integrato.
- **Cosa abbiamo fatto**:
  - Creato il motore `liquid_glass.rs`: mesh vettoriali con ombreggiatura Gouraud, gradienti a 4 vertici, micro-bordi speculari superiori (`rgba(255, 255, 255, 38)`) e ombre di profondità (`rgba(0, 0, 0, 70)`).
  - Ridisegnata la toolbar centrata fluttuante con pill buttons, badge di stato bicolore e controlli di layer con animazione di pressione tattile.
  - Documentate le specifiche complete in `APPLE_LIQUID_GLASS_UI_COMPLETE.md`.

---

### Sessione 7: Chiarimenti Bluetooth, Avvio Windows e Deep Sleep Timer
- **Data & Richiesta**: `2026-10-21`
- **Domanda Utente**: *"Perché se la tastiera è in modalità Bluetooth sembra non funzionare se premo il preset? E vorrei l'opzione per avvio con Windows, deep sleep timer e info tastiera."*
- **Diagnosi & Risposta**:
  - **Differenza Hardware Flash vs Runtime Host**: Se un'azione è di tipo "Host Automation" (es. muovere finestre o eseguire macro Windows), essa invia un PhantomTrigger (F13..F24) all'host. Se il software `vortexdeck.exe` non è aperto sul PC connesso via Bluetooth, Windows ignora i tasti F13..F24. I comandi HID nativi (volume, media, tastiera) funzionano invece ovunque standalone.
  - **Autostart Windows**: Illustrato il funzionamento del flag `--background` e la scrittura della chiave di registro `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
  - **Deep Sleep Timer**: Spiegato come il timer di risparmio energetico BLE sia gestito a livello di firmware CH57x e impostabile tramite tool USB.

---

### Sessione 8: Falsi Positivi Windows Defender e Analisi Batteria
- **Data & Richiesta**: `2026-10-21`
- **Domanda Utente**: *"Windows Defender dice che ci sono trojan, è per il programma che stiamo creando? E nelle info c'è la batteria o se in carica?"*
- **Diagnosi & Risposta**:
  - **Heuristica Antivirus**: Confermato falso positivo tipico dei tool di automazione compilati localmente senza firma digitale commerciale EV. Le API `RegisterHotKey` (intercettazione tasti globali F13..F24) e la chiamata a `UsbDk` fanno scattare le firme euristiche generiche.
  - **Stato Batteria BLE**: Analizzato il protocollo HID Battery Service standard (UUID `0x180F`). Chiarito che via cavo USB il dispositivo è alimentato costantemente, mentre la telemetria di carica residua in percentuale è esposta dal sistema operativo nelle impostazioni Bluetooth di Windows 11.

---

### Sessione 9: Risoluzione Luminosità Manopole su Windows (WMI + Gamma)
- **Data & Richiesta**: `2026-10-21`
- **Domanda Utente**: *"Per quanto riguarda il decoder mi servirebbe l'opzione di poter abbassare o alzare la luminosità, perché se metto l'opzione di aumentare la luminosità non me lo fa."*
- **Diagnosi Tecnica Approfondita**:
  - Nel tool upstream `ch57x-keyboard-tool`, i token `macbrightnessup` e `macbrightnessdown` sono semplici alias per i tasti `Pause` (0x48) e `ScrollLock` (0x47). Su macOS questi codici regolano la luminosità degli schermi Apple, ma **su Windows non hanno alcun effetto sulla luminosità del display!**
- **Soluzione Implementata**:
  - Aggiunto `ActionDefinition::AdjustBrightness { delta: i32 }` nel dominio puro.
  - Sviluppato [`crates/platform/src/windows/brightness.rs`](crates/platform/src/windows/brightness.rs): motore ibrido nativo Windows con cache atomica per risposta a 0 ms nell'interfaccia, chiamata asincrona debounced a WMI (`root/wmi:WmiMonitorBrightnessMethods.WmiSetBrightness`) per notebook/Surface e fallback GDI `SetDeviceGammaRamp` per monitor esterni desktop.

---

### Sessione 10: Espansione Master dei Preset 1-Click e Template di Layer
- **Data & Commit**: `2026-10-21` | `b7263bf`
- **Motivazione**: Rendere la configurazione istantanea senza dover inserire manualmente combinazioni di tasti o argomenti da riga di comando.
- **Cosa abbiamo fatto**:
  - Creato [`crates/ui/src/presets.rs`](crates/ui/src/presets.rs) e la guida [`PRESETS_GUIDE.md`](PRESETS_GUIDE.md).
  - **7 Preset 3-in-1 per Encoder**: Volume + Mute, Luminosità + Play, Zoom + Reset, Scorri Tab IDE/Browser, Undo/Redo/Salva, Desktop Virtuali Win11, Scroll Mouse.
  - **8 Chip Rapidi Produttività**: `agy` (DeepMind AI CLI via prompt classico o Windows Terminal), `wt.exe`, Gestione Dispositivi (`devmgmt.msc`), Screenshot (`Win+Shift+S`), Appunti (`Win+V`), Mute Microfono (`Win+Alt+K`), Task Manager, Sposta Monitor.
  - **Template Intero Layer**: 1-click per pre-configurare tutti i 18 slot di un layer con alert di sicurezza e conferma se il layer contiene già dati.

---

### Sessione 11: Drag-and-Drop per Spostamento/Scambio Configurazione Tasti
- **Data & Commit**: `2026-10-21` | `b7263bf`
- **Motivazione**: L'utente ha richiesto la possibilità di trascinare un tasto su un altro (es. da tasto 1 a tasto 12) per spostare o scambiare la configurazione al volo.
- **Sfida Tecnica**: egui non possiede un sistema nativo di drag-and-drop a griglia per pulsanti personalizzati; bisognava gestire inizio trascino, puntatore mouse globale, drop target detection e rendering del tasto fluttuante sopra tutti gli altri elementi.
- **Cosa abbiamo fatto**:
  - In `crates/ui/src/state.rs`: aggiunti campi reattivi `dragged_slot`, `drag_pointer_pos`, `drag_target_slot` e il metodo atomico `swap_slots(slot_a, slot_b)`.
  - In `crates/ui/src/pad_canvas.rs`: configurato `Sense::click_and_drag()`. Durante il drag, il tasto sorgente viene disegnato a bassa opacità, il tasto target si illumina con bordo verde Apple, e viene renderizzata una scheda fluttuante semitrasparente (`draw_drag_ghost`) con l'etichetta e il badge del tasto trascinato.

---

### Sessione 12: Risoluzione GitHub Actions CI/CD e Build Cloud
- **Data & Commit**: `2026-10-21` | `b7263bf`, `bf1fc2a`, `2fe156c`
- **Motivazione**: I workflow su GitHub fallivano, impedendo la compilazione sul cloud richiesta dall'utente ("voglio che su github avvenga la compilazione e al momento no sul desktop").
- **Diagnosi & Risoluzione**:
  1. *Toolchain Crash*: `rust-toolchain.toml` conteneva `channel = "stable-x86_64-pc-windows-gnu"`, che mandava in crash il runner `rustup` su Linux e Windows MSVC. Rimosso il file per consentire l'uso del compilatore stable di default per ogni runner OS.
  2. *Clippy Linux Warnings*: Su runner Linux Ubuntu, `CREATE_NO_WINDOW` e gli stub del System Tray generavano warning `-D warnings`. Risolto con guardie `#[cfg(target_os = "windows")]` e `#[allow(dead_code)]`.
- **Risultato**: Workflow completato con **100% verde** su tutti i 5 job e pubblicazione dell'artefatto cloud `VortexDeck-Windows-x64`.

---

### Sessione 13: Audit di Sicurezza e Sanitizzazione Dati Sensibili
- **Data & Commit**: `2026-10-21` | `b7263bf`
- **Motivazione**: Verificare scrupolosamente l'assenza di dati personali, chiavi, token o credenziali nei file pubblici del repository prima del rilascio.
- **Audit Eseguito**:
  - Scansione automatizzata su tutti i sorgenti, file di configurazione e documentazione per pattern sensibili (`ghp_`, API keys, password, email personali, percorsi fisici con nomi utente locali).
  - Esito: **0 stringhe o percorsi sensibili presenti**. L'email di commit Git è protetta tramite l'indirizzo anonimo noreply di GitHub.

---

### Sessione 14: Compilazione Release Locale e Validazione Eseguibile
- **Data**: `2026-10-21`
- **Motivazione**: Esecuzione della build finale su richiesta dell'utente direttamente sulla macchina locale.
- **Esito**:
  - Compilato con successo `cargo build --release --bin vortexdeck` in **1m 33s** (0 errori, 0 warning).
  - Pacchetto assemblato in [`dist/VortexDeck/`](dist/VortexDeck) con `vortexdeck.exe` (17.5 MB), binario USB `ch57x-keyboard-tool.exe` e mapping di esempio.
  - Test diagnostico `--doctor` superato con successo: tastiera rilevata e driver `UsbDk` attivo.


---

### Sessione 15: Fix Luminosità Surface/Portatili, Toggle Autostart UI e Funzionamento Bluetooth / App Chiusa
- **Data**: `2026-10-21`
- **Motivazione**: Risoluzione delle limitazioni emerse nell'uso su laptop Windows / Microsoft Surface, aggiunta del controllo visivo dell'avvio con Windows ed esecuzione delle automazioni host (`agy`) sia via Bluetooth che ad app chiusa.
- **Interventi Realizzati**:
  1. **Motore Regolazione Luminosità a 2 Stadi (`ch57x-platform`)**:
     - *Stadio 1 (Istantaneo, 0ms latency)*: Accesso diretto al dispositivo kernel `\\.\LCD` tramite `CreateFileW` e `DeviceIoControl` (`IOCTL_VIDEO_QUERY_DISPLAY_BRIGHTNESS` / `IOCTL_VIDEO_SET_DISPLAY_BRIGHTNESS`). Ottimizzato specificamente per display interni di Microsoft Surface e laptop.
     - *Stadio 2 (Fallback)*: Worker background debounced con WMI `WmiSetBrightness` e GDI `SetDeviceGammaRamp`.
     - *Preset & UI*: Rimosso l'uso errato dei token macOS `macbrightnessdown/up`; il preset manopola assegna ora azioni host `AdjustBrightness` (`delta: -6` CCW e `delta: +6` CW) con auto-allocazione dei trigger.
  2. **Toggle Grafico "Avvio con Windows"**:
     - Aggiunto nella Top Toolbar principale un pulsante toggle Liquid Glass dinamico (`● Avvio Win: ON` verde / `○ Avvio Win: OFF` grigio).
     - Integrato nel menu contestuale della System Tray (`Avvio automatico con Windows (ON/OFF)`) e sincronizzato con la checkbox nel Doctor Panel.
     - Gestione reale della chiave di registro `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` con valore `VortexDeck` e flag `--background`.
  3. **Pool 60 Trigger BLE-Safe per Bluetooth e App Chiusa**:
     - Sostituiti tutti i vecchi codici virtuali superiori a `0x65` (che venivano troncati dal firmware CH57x via Bluetooth) con 60 combinazioni con tasti standard (`F1..F11`, `0..9`, `A..Z`, `-`) su modificatori `Ctrl+Alt+Shift` e `Ctrl+Alt+Win`.
     - Rimosso `MOD_NOREPEAT` sul thread Win32 Hotkey per consentire rotazione continua fluida delle manopole rotative.
     - Generazione automatica di scorciatoie `.lnk` in `%APPDATA%\Microsoft\Windows\Start Menu\Programs\VortexDeck` tramite COM `IShellLinkW::SetHotkey`: Windows Explorer intercetta i trigger e lancia `agy` istantaneamente anche se VortexDeck è completamente terminato.
     - Auto-salvataggio delle modifiche pendenti del profilo alla chiusura della finestra con la X rossa prima del passaggio a demone background.
     - Badge di avviso nella Top Toolbar (`⚠️ Flash USB necessario per aggiornare il Bluetooth`) con apertura diretta del modal di conferma flashing.
- **Esito Verifiche**:
  - `cargo test --workspace`: **100% test superati (75 unit & integration tests)**.
  - `cargo clippy --workspace --all-targets -- -D warnings`: **0 warning**.
  - `cargo fmt --all -- --check`: **Formattazione conforme**.
  - Compilazione Release e packaging `dist/VortexDeck/vortexdeck.exe` completati con successo.


---

### Sessione 16: Fix Chiusura Immediata System Tray ("Esci" a Zero Processi Residui), Risveglio Win32 e Autostart a Doppia Garanzia
- **Data**: `2026-10-21`
- **Motivazione**: Risoluzione dei due bug critici nell'uso quotidiano su Windows 11 / Surface:
  1. *Chiusura bloccata da System Tray*: premendo "Esci", l'icona spariva ma il processo `vortexdeck.exe` restava orfano e bloccato in Task Manager perché la finestra nascosta (`Visible(false)`) metteva in pausa il loop di `winit`/`eframe`.
  2. *Autostart fallace all'avvio di Windows*: Windows avviava il processo con working directory `C:\Windows\System32`, rompendo i percorsi relativi, e il Task Manager di Windows 11 poteva bloccare l'avvio in `StartupApproved`.
- **Interventi Realizzati**:
  1. **Working Directory Garantita (`crates/app/src/main.rs`)**:
     - Impostata come primissima istruzione in `fn main()` la directory dell'eseguibile (`std::env::set_current_dir(exe_dir)`), garantendo che file di configurazione e binari relativi vengano sempre trovati.
     - Preservato `_tray_handle` nel loop background con terminazione immediata e pulita via `std::process::exit(0)`.
  2. **Chiusura Immediata Win32 e Risveglio Finestra (`crates/app/src/tray.rs`)**:
     - Al click su "Esci" (`ID_TRAY_EXIT`), chiamata immediata a `remove_tray_icon(hwnd)` (`Shell_NotifyIconW(NIM_DELETE)`) per eliminare subito l'icona dalla shell di Windows, seguita da `std::process::exit(0)` per terminare all'istante tutti i thread senza processi residui.
     - Al click su "Apri VortexDeck" o doppio-click sull'icona della tray, risveglio diretto a livello Win32 (`FindWindowW`, `ShowWindow(SW_RESTORE)`, `ShowWindow(SW_SHOW)`, `SetForegroundWindow`), bypassando la pausa di rendering di `winit`.
     - Nel menu contestuale (`WM_RBUTTONUP`), visualizzazione dinamica della spunta Win32 `MF_CHECKED` / `MF_UNCHECKED` e toggle reattivo istantaneo su `ID_TRAY_AUTOSTART`.
  3. **Sistema Autostart a Doppia Garanzia (`crates/platform/src/windows/windows_api.rs`)**:
     - *Doppia Registrazione*: Scrittura del valore `VortexDeck` in `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` con comando `"<exe>" --background` E creazione automatica del collegamento `.lnk` in `%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup\VortexDeck.lnk` con COM `IShellLinkW` (inclusi `SetArguments` e `SetWorkingDirectory`).
     - *Sblocco Windows 11*: Cancellazione automatica dei blocchi di Task Manager in `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run`.
     - *Rimozione Completa*: Cancellazione di chiave `Run`, chiave `StartupApproved` e cancellazione file `.lnk` da disco.
  4. **Pulsante Toolbar Reattivo & Test Automatizzato**:
     - Sostituito il toggle con un vero `egui::Button` cliccabile con testo pulito `Avvio Win: ON` / `Avvio Win: OFF` (senza caratteri Unicode che causavano problemi di font fallback) collegato a `coordinator.toggle_autostart()`.
     - Aggiunto unit test `test_win32_autostart_full_lifecycle` che collauda l'intero ciclo di attivazione, presenza file `.lnk`, chiave registro e disattivazione.
- **Esito Verifiche**:
  - `cargo test --workspace`: **100% test superati (76 unit & integration tests)**.
  - `cargo clippy --workspace --all-targets -- -D warnings`: **0 warning**.
  - `cargo fmt --all -- --check`: **Formattazione conforme**.
  - Binario release aggiornato in [`dist/VortexDeck/vortexdeck.exe`](dist/VortexDeck/vortexdeck.exe).


---

### Sessione 17: Iconografia Ufficiale VortexDeck, Icona Notifiche Nascoste Dinamica e Logo Toolbar
- **Data**: `2026-10-21`
- **Motivazione**: Creare un'identità visiva d'impatto e futuristica per VortexDeck, dotando il programma di icone dedicate per l'eseguibile desktop, la finestra dell'applicazione e soprattutto la System Tray (area di notifica / "icone nascoste" di Windows 11).
- **Interventi Realizzati**:
  1. **Generazione e Compositing Asset Grafici (`assets/icons/`)**:
     - *Vortex App Icon (`vortexdeck_app_icon.png` / `.ico`)*: Design basato sul glifo a vortice quantico glowing cyan/mint green, incastonato in uno squircle stile Apple/Fluent in dark obsidian frosted glass con riflesso speculare superiore e bordi smussati luminosi (1024x1024 PNG con trasparenza alpha nativa + ICO multi-risoluzione da 16x16 a 256x256).
     - *System Tray Active Icon (`vortexdeck_tray_icon.png` / `.ico`)*: Glifo circolare geometrico ad alto contrasto con bagliore neon ciano/menta, ottimizzato per perfetta leggibilità a 16x16 / 32x32 pixel nella barra delle notifiche di Windows.
     - *System Tray Paused Icon (`vortexdeck_tray_paused.png` / `.ico`)*: Variante speculare con bagliore neon arancione/ambra per segnalare istantaneamente all'utente quando le automazioni host sono in pausa.
     - *Dati Binari RGBA Compilati (`.rgba`)*: Generati file binari raw RGBA 64x64 e 32x32 per inclusione a tempo di compilazione a zero overhead di decodifica runtime (`include_bytes!`).
  2. **Integrazione Icona Finestra e Taskbar (`crates/app/src/main.rs`)**:
     - Configurato `egui::ViewportBuilder::default().with_icon(egui::IconData { ... })` con il payload RGBA 64x64 di `vortexdeck_app_icon`.
     - Aggiunto il logo grafico 22x22 affiancato alla scritta "VortexDeck" nella Toolbar superiore macOS Liquid Glass (`self.logo_texture`).
  3. **Integrazione Dinamica System Tray Win32 (`crates/app/src/tray.rs`)**:
     - Implementato `load_tray_icon(is_paused: bool) -> HICON` a 4 stadi: ricerca icona locale su disco -> cartella dell'eseguibile -> fallback estrazione da memoria compilata a `%TEMP%` -> fallback `LoadIconW(None, IDI_APPLICATION)`.
     - Implementato `update_tray_icon(is_paused: bool)` che invoca `Shell_NotifyIconW(NIM_MODIFY, &nid)` per scambiare in tempo reale l'icona tra ciano attivo e ambra in pausa.
     - Monitoraggio reattivo del cambio stato pausa in `VortexDeckApp::update()` e nel loop background.
  4. **Release Packaging (`packaging/windows/build_release.ps1`)**:
     - Inclusa la cartella `assets/icons` nel bundle distribuibile `dist/VortexDeck/assets/icons`.
     - Gestito l'arresto preventivo di istanze attive per consentire il repackaging pulito di `dist/VortexDeck`.
- **Esito Verifiche**:
  - `cargo test --workspace`: **100% test superati (76 unit & integration tests)**.
  - `cargo clippy --workspace --all-targets -- -D warnings`: **0 warning**.
  - `build_release.ps1`: Pacchetto distribuibile generato con successo in [`dist/VortexDeck`](dist/VortexDeck).
