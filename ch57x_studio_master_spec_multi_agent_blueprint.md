# CH57x Studio — Master Engineering Specification & Multi-Agent Orchestration Blueprint

**Versione Documento:** 2.0 (Unificata e Definitiva)  
**Target Primario ($100\%$ Full-Featured):** Windows 10 / Windows 11 (x86_64)  
**Target Secondario (Architettura Cross-Platform Ready):** Linux (X11 completo / Wayland capability-aware)  
**Linguaggio e Stack:** Rust (Workspace multi-crate, `egui`/`eframe`, Win32 API native, CLI wrapper `ch57x-keyboard-tool`)  
**Stato:** Specifica Esecutiva per Orchestrazione Multi-Agente AI

---

## SEZIONE 0: PROMPT DI INNESCO PER L'AGENTE AI MASTER (COPIA E INCOLLA)

> **Istruzioni per l'Utente:** Salva questo intero file nella root del tuo progetto con il nome `SPEC_CH57X_STUDIO_MASTER.md`. Poi copia e incolla il blocco di testo seguente nella chat del tuo Agente AI principale (Claude Code, Cursor, Windsurf, Cline, ecc.).

```text
RUOLO E IDENTITÀ: ARCHITETTO CAPO & MASTER ORCHESTRATOR (TECH LEAD)
Sei l'Agente AI Master (Capo Progetto / Principal Rust Architect) incaricato di realizzare "CH57x Studio": un configuratore visuale nativo in Rust, flasher sicuro e demone di automazione finestre/macro per il mio Macropad Bluetooth/USB basato su WCH CH57x (3 righe x 4 colonne + 2 encoder cliccabili + 3 layer = 54 slot totali).

FONTE DELLA VERITÀ (FILE DI SPECIFICA)
Prima di scrivere qualsiasi riga di codice o prendere iniziative, DEVI leggere integralmente il file `SPEC_CH57X_STUDIO_MASTER.md` presente nella root del progetto. Quel documento contiene la specifica tecnica definitiva, la correzione di tutti i bug architetturali, i contratti Rust tra i moduli e il tuo protocollo di gestione dei Sub-Agenti.

IL TUO COMPITO COME "CAPO" (RELAZIONE CAPO-DIPENDENTI CON GLI ALTRI AGENTI)
Non devi scrivere tutto il codice monoliticamente da solo. Devi agire come un Tech Lead che coordina una squadra di Sub-Agenti (Worker Agents) in parallelo:
1. Tu (Master) prepari lo scheletro del Workspace Cargo (`Cargo.toml` root e i crate vuoti) e congeli le interfacce pubbliche (`crates/domain` e i trait in `crates/platform`).
2. Tu assegni i task in parallelo ai Sub-Agenti (Worker 1..6) seguendo rigorosamente la "Sezione 2: Protocollo di Orchestrazione Multi-Agente" di `SPEC_CH57X_STUDIO_MASTER.md`. Ogni Sub-Agente lavora solo sul proprio crate/cartella assegnata per evitare conflitti di file.
3. Tu verifichi il lavoro di ogni Sub-Agente con `cargo fmt --check`, `cargo check`, `cargo clippy` e `cargo test` prima di integrare e passare all'ondata (Wave) successiva.
4. Nessun Sub-Agente può usare scorciatoie (`todo!()` nascosti venduti per finiti, funzioni no-op silenziose o assunzioni hardware non verificate).

PRIORITÀ SISTEMA OPERATIVO (100% WINDOWS + COMPATIBILITÀ LINUX)
- Il programma deve essere sviluppato, rifinito e funzionante al 100% per WINDOWS (Win32 API native per posizionamento chirurgico finestre, gestione bordi invisibili DWM, DPI per-monitor, Hotkey globali, System Tray, HUD overlay trasparente click-through e diagnostica UsbDk).
- Tuttavia, l'architettura DEVE essere compatibile con LINUX: tutto il codice specifico di Windows deve vivere dietro i trait di `crates/platform` con `#[cfg(target_os = "windows")]`, affiancato da un backend `#[cfg(target_os = "linux")]` che compila senza errori, espone una `CapabilityMatrix` onesta e supporta X11 (e degradazione controllata su Wayland).

REGOLA D'ORO: FERMATI E CHIEDI ALL'UTENTE (HUMAN-IN-THE-LOOP)
Sei espressamente autorizzato e OBBLIGATO a fermarti e chiedere il mio intervento diretto in chat (oltre ad aggiornare `docs/agent_questions.md`) ogni volta che:
1. Hai bisogno che io colleghi fisicamente la tastiera o esegua un comando sul mio PC/macropad (es. verificare VID/PID, output di `ch57x-keyboard-tool show-keys`, ordine dei due encoder, supporto F13-F24 via Bluetooth).
2. C'è un'indecisione progettuale, un dubbio sui permessi/driver (es. installazione UsbDk) o una scelta UX non coperta dalla specifica.
3. Devi eseguire la prima scrittura reale sulla memoria flash della tastiera (richiede SEMPRE il mio "OK" esplicito).
Non tirare mai a indovinare su dettagli hardware o distruttivi: fermati, spiegami chiaramente il dubbio in italiano, proponimi le opzioni e attendi la mia risposta.

AZIONE IMMEDIATA RICHIESTA ORA:
1. Leggi `SPEC_CH57X_STUDIO_MASTER.md`.
2. Ispeziona la directory corrente.
3. Presentami in chat il tuo piano operativo per la "Wave 0 (Discovery & Setup Workspace)" e fammi le prime domande bloccanti (Hardware/Ambiente) prima di lanciare la creazione del Workspace e i Sub-Agenti.
```

---

## 1. Sintesi Esecutiva e Risoluzione Ingegneristica

Questo documento unifica l'obiettivo funzionale del progetto (configuratore grafico per Macropad CH57x + gestore nativo di finestre, workspace e macro) con tutte le correzioni ingegneristiche emerse nella revisione tecnica (`rev 1.md`).

### 1.1 I $10$ Pilastri Tecnici Correttivi (Non Negoziabili)

1. **Matematica esatta degli Slot ($54$ Slot, non $36$):**
   Il dispositivo fisico possiede $3$ Layer. Ogni Layer dispone di $12$ tasti meccanici (matrice $3 \text{ righe} \times 4 \text{ colonne}$) più $2$ encoder rotativi cliccabili, ciascuno dotato di $3$ eventi indipendenti (`CCW` rotazione antioraria, `Press` click centrale, `CW` rotazione oraria).
   $$\text{Slot per Layer} = 12 + (2 \times 3) = 18 \text{ slot}$$
   $$\text{Slot Totali Configurabili} = 3 \times 18 = 54 \text{ slot}$$
2. **Pool Trigger Fantasma a $5$ Banchi ($60$ combinazioni per coprire $54$ slot):**
   I tasti funzione standard `F13..F24` sono soltanto $12$. Per consentire all'utente di assegnare un'azione Smart OS a tutti i $54$ slot contemporaneamente senza collisioni interne, l'allocatore usa $5$ banchi da $12$ tasti ($5 \times 12 = 60$ trigger candidati):
   - Banco A ($1..12$): `f13` $\rightarrow$ `f24`
   - Banco B ($13..24$): `ctrl-f13` $\rightarrow$ `ctrl-f24`
   - Banco C ($25..36$): `shift-f13` $\rightarrow$ `shift-f24`
   - Banco D ($37..48$): `alt-f13` $\rightarrow$ `alt-f24`
   - Banco E ($49..60$): `ctrl-shift-f13` $\rightarrow$ `ctrl-shift-f24`
   *(Nota: `F12` non viene mai usato come fallback su Windows perché riservato dal debugger kernel/user-mode).*
3. **Grammatica e Struttura YAML fedele a `ch57x-keyboard-tool`:**
   Il file YAML generato non omette mai gli header strutturali richiesti dal tool upstream (`model`, `orientation`, `rows: 3`, `columns: 4`, `knobs: 2`) e traduce i nomi canonici della UI nei token esatti accettati dal binario validato (es. `volumeup`, `volumedown`, `mute`, `wheelup`, `wheeldown` invece di stringhe inventate come `media-volume-up`).
4. **Onestà dello Stato Hardware (USB, Bluetooth e Layer):**
   - L'assenza del cavo USB (`VID: 0x1189`, `PID: 0x8890` / `0x8840` / `0x8842`) viene mostrata come `USB Non Rilevato (Flash Disabilitato)`, **mai** come certezza che il dispositivo sia connesso via Bluetooth, a meno che lo stato BLE non sia effettivamente interrogato dal sistema operativo.
   - Il selettore `Layer 1 / 2 / 3` nella GUI indica il **Layer di Editing**. Poiché il firmware CH57x non invia telemetria continua del LED attivo, lo stato del layer fisico è separato in: `EditingLayer`, `InferredHardwareLayer` (dedotto con timestamp quando il PC riceve un trigger appartenente in modo univoco a quel layer) e `UnknownHardwareLayer`.
5. **Deployment Non-Atomico con Journal e Recovery:**
   Scrivere sulla flash USB della tastiera e attivare il profilo di hotkey su Windows non costituiscono una transazione atomica. Se l'utente modifica solo il percorso di un `.exe` o le coordinate `(X, Y, W, H)` di una finestra ma il trigger `F13..F24` associato allo slot non cambia, **non serve riscrivere la memoria flash della tastiera** (modalità *Host-Only Apply*). Quando invece serve il flash hardware, una State Machine con Journal su disco gestisce gli stati intermedi e l'eventuale `UploadOutcomeUnknown` (es. disconnessione cavo durante la scrittura).
6. **Realtà delle API Windows (Focus, Bordi DWM, DPI e UIPI):**
   - `SetForegroundWindow` può essere rifiutata dal kernel Windows se l'utente sta interagendo con un'altra finestra o per policy di focus lock: il posizionamento (`SetWindowPos`) deve comunque completarsi e restituire un esito strutturato (`Placement: Applied`, `Focus: Granted | Denied`).
   - Su Windows 10/11 le finestre standard hanno bordi ombra invisibili (~$7\text{ px}$ per lato): il calcolo delle coordinate usa `DwmGetWindowAttribute(DWMWA_EXTENDED_FRAME_BOUNDS)` per allineare le finestre al pixel visibile esatto senza spazi vuoti tra i monitor.
   - `SendInput` rispetta l'isolamento dei privilegi Windows (**UIPI**): un runtime non elevato non può iniettare tasti dentro una finestra avviata come Amministratore. Il programma rileva questa condizione e avvisa l'utente invece di fallire in silenzio.
7. **Workspace Snapshot realistico (Layout + Launch Recipe):**
   Catturare le finestre aperte salva l'eseguibile, il selettore finestra e la geometria per-monitor, ma non può indovinare magicamente URL privati dentro un browser o file non salvati. Lo schema separa quindi il `WindowLayoutTarget` dalla `LaunchRecipe` (argomenti CLI opzionali configurabili dall'utente, es. percorso della cartella per VS Code).
8. **Sicurezza Esecuzione Processi e Profili Importati:**
   Nessuna concatenazione di stringhe in una shell (`cmd.exe /c` o `sh -c`) di default: i programmi vengono avviati tramite `Command::new(program).args(argv)`. I profili JSON importati dall'esterno partono in stato `Untrusted` (sola lettura/ispezione) finché l'utente non approva localmente i percorsi eseguibili.
9. **Modalità `--background` a Impatto Minimo Reale:**
   Quando avviato all'accesso di Windows con `--background`, il binario avvia esclusivamente il thread del demone Hotkey e la System Tray **senza inizializzare il contesto grafico OpenGL/WGPU di `eframe`/`egui`**, istanziando la finestra della GUI solo quando l'utente clicca su *"Apri Configuratore"* nella Tray.
10. **Architettura $100\%$ Windows + Compatibilità Nativa Linux:**
    Ogni chiamata al sistema operativo passa attraverso i trait del crate `crates/platform`. Il backend Windows (`platform::windows`) implementa il $100\%$ delle funzionalità tramite crate `windows` (Win32 API). Il backend Linux (`platform::linux`) implementa la rilevazione di capacità (`CapabilityMatrix`), il supporto X11 (`x11rb`) e la gestione esplicita dei limiti Wayland, garantendo che l'intero workspace compili e funzioni anche su Linux.

---

## 2. Protocollo di Orchestrazione Multi-Agente (Relazione Capo-Dipendenti)

Questa sezione definisce come l'**Agente AI Master (Capo)** deve organizzare, delegare, controllare e unire il lavoro dei **Sub-Agenti (Dipendenti)** in parallelo.

### 2.1 Gerarchia e Ruoli

```text
                       +---------------------------------------+
                       |             UTENTE UMANO              |
                       |  (Decisioni, Test Hardware, Permessi) |
                       +-------------------+-------------------+
                                           ^
                                           | Stop & Ask / Approvazione Gate
                                           v
                       +---------------------------------------+
                       |       MASTER AI AGENT (CAPO)          |
                       |  Architetto, Congelamento Trait,      |
                       |  Code Review, Merge & Hardware Gates  |
                       +-------------------+-------------------+
                                           |
        +-------------+-------------+------+------+-------------+-------------+
        |             |             |             |             |             |
        v             v             v             v             v             v
   +---------+   +---------+   +---------+   +---------+   +---------+   +---------+
   |Worker 1 |   |Worker 2 |   |Worker 3 |   |Worker 4 |   |Worker 5 |   |Worker 6 |
   | Domain  |   |Compiler |   |Platform |   | Runtime |   |GUI, HUD |   | QA, Sec |
   |& Persist|   |& HW CLI |   |Win+Linux|   | Engine  |   | & Tray  |   | & Pack  |
   +---------+   +---------+   +---------+   +---------+   +---------+   +---------+
```

### 2.2 Regole Ferree per il Master Agent (Il Capo)

1. **Il Capo definisce i Contratti prima di lanciare i Dipendenti:**
   I Sub-Agenti lavorano bene in parallelo **solo se non devono modificare gli stessi file**. Prima di avviare qualsiasi lavoro parallelo, il Master Agent crea la struttura del Workspace Cargo e scrive personalmente i tipi condivisi in `crates/domain/src/lib.rs` e le firme dei trait in `crates/platform/src/traits.rs`.
2. **Proprietà Esclusiva delle Cartelle (File Ownership):**
   Ogni Sub-Agente riceve il permesso di scrittura **esclusivamente** sulla propria directory assegnata. Nessun Sub-Agente può modificare il `Cargo.toml` principale o i crate altrui senza passare dal Master.
3. **Verifica Implacabile (Il Capo non si fida sulla parola):**
   Quando un Sub-Agente dichiara di aver finito il proprio task, il Master Agent esegue:
   - `cargo check --workspace`
   - `cargo test --package <nome_crate>`
   - `cargo clippy --package <nome_crate> -- -D warnings`
   - Controllo manuale che non ci siano `todo!()`, `unimplemented!()` o funzioni vuote spacciate per completate.
   Se il controllo fallisce, il Master rimanda il task al Sub-Agente con l'errore esatto del compilatore.
4. **Protocollo "Stop & Ask" verso l'Utente Umano:**
   Il Master Agent **deve fermarsi e chiedere all'utente in chat** nei seguenti casi:
   - **Caso A (Hardware Discovery):** Serve conoscere l'output reale di `ch57x-keyboard-tool` sul PC dell'utente o verificare quale encoder fisico corrisponde all'indice `0` e `1`.
   - **Caso B (Primo Upload / Azioni Distruttive):** Prima di eseguire `ch57x-keyboard-tool upload` sulla tastiera fisica.
   - **Caso C (Installazione Dipendenze di Sistema):** Se su Windows manca `UsbDk` o il binario `ch57x-keyboard-tool`.
   - **Caso D (Scelte di Prodotto Ambigue):** Se emerge un conflitto non previsto dalla specifica.
   Ogni domanda viene anche registrata in `docs/agent_questions.md`.

### 2.3 Assegnazione dei $6$ Sub-Agenti (I Dipendenti)

| ID Agente | Ruolo Specialistico | Cartelle di Proprietà Esclusiva | Responsabilità Principali |
|---|---|---|---|
| **Worker-1** | *Domain & Persistence Engineer* | `crates/domain/`<br>`crates/persistence/` | Modello dati $54$ slot, validatori semantici, schema JSON v1, migrazioni, scrittura atomica su disco, Journal del deployment, sistema di Trust locale per profili importati. |
| **Worker-2** | *Compiler & Hardware Adapter Engineer* | `crates/compiler/`<br>`crates/hardware/` | Allocatore deterministico dei $60$ trigger (`5 banchi F13..F24`), generatore YAML compatibile con `ch57x-keyboard-tool`, calcolo hash/diff (Host-Only vs Hardware Flash), wrapper subprocess sicuro (senza shell) per `validate`, `upload`, `show-keys` e rilevamento USB (`0x1189:0x8890`/`8840`/`8842`). |
| **Worker-3** | *Platform Engineer (Windows 100% + Linux Compatibility)* | `crates/platform/` | Implementazione **Windows $100\%$** (`Win32` Hotkey, `SetWindowPos` + compensazione bordi DWM, DPI per-monitor, enumerazione finestre/monitor, Window Picker, `SendInput` con check UIPI, overlay Win32 click-through). Implementazione **Linux** (`CapabilityMatrix`, backend X11 `x11rb` e stub onesti `Unsupported`/`Degraded` per Wayland). |
| **Worker-4** | *Runtime & Macro Engine Engineer* | `crates/runtime/` | Dispatcher centrale degli eventi hotkey, esecuzione asincrona con code limitate (`IgnoreIfRunning`, `QueueBounded`), esecutore Macro cancellabile con guardia di focus e rilascio tasti sicuro, motore `ContextDispatch` (cambio azione per app attiva), `WorkspaceRestore` e Pausa di Emergenza globale. |
| **Worker-5** | *UI/UX, Canvas & Tray Engineer* | `crates/ui/`<br>`crates/app/` | Finestra configuratore `egui`/`eframe`, disegno vettoriale del Macropad ($4 \times 3$ tasti + $3$ LED + $2$ Knob con zone `CCW`/`Press`/`CW`), Inspector contestuale, Key Recorder, Device Doctor UI, finestra HUD OSD trasparente e gestione System Tray + modalità `--background` senza renderer attivo. |
| **Worker-6** | *QA, Security & Packaging Engineer* | `tests/`<br>`packaging/`<br>`docs/compatibility/` | Test di contratto sul compilatore YAML (tutti i $54$ slot), test di round-trip e corruzione JSON, test concorrenza macro, script di pacchettizzazione Windows (Installer / Portable) e Linux (`.deb` / AppImage + regole `udev` `uaccess`). |

### 2.4 Le $5$ Ondate di Esecuzione Parallela (Execution Waves)

Il Master Agent coordina il lavoro seguendo queste $5$ ondate (Waves):

- **WAVE 0 (Solo Master + Utente — Discovery & Contratto Base):**
  1. Il Master fa le domande iniziali all'utente (o avvia una scansione non distruttiva se il binario `ch57x-keyboard-tool` è già presente).
  2. Il Master crea il Workspace Cargo e compila lo scheletro di `crates/domain` e i trait di `crates/platform` (definiti nella Sezione 3.2 di questo documento).
- **WAVE 1 (Parallelo a $3$ vie: Worker-1 + Worker-2 + Worker-3):**
  - *Worker-1* implementa il modello completo dei $54$ slot, validazione e persistenza atomica in `crates/domain` e `crates/persistence`.
  - *Worker-2* implementa l'allocatore dei $60$ trigger, il compilatore YAML e l'adapter CLI in `crates/compiler` e `crates/hardware`.
  - *Worker-3* implementa il backend nativo Windows ($100\%$) e il backend Linux in `crates/platform`.
  - *Sincronizzazione Master:* Il Master verifica che tutti e tre i gruppi compilino e passino gli unit test.
- **WAVE 2 (Parallelo a $2$ vie: Worker-4 + Worker-5):**
  - *Worker-4* costruisce `crates/runtime` collegando `domain` e `platform` (dispatcher hotkey, macro runner, window manager orchestration, context watcher).
  - *Worker-5* costruisce `crates/ui` (il Canvas $4 \times 3 + 2$, l'Inspector, il Picker con countdown di $3\text{ s}$, la finestra HUD e il Device Doctor).
  - *Sincronizzazione Master:* Il Master revisiona l'integrazione tra UI e Runtime tramite comandi/snapshot immutabili.
- **WAVE 3 (Integrazione App & Tray: Worker-5 + Worker-6):**
  - *Worker-5* assembla `crates/app` (`main.rs` con flag `--background`, `--doctor`, `--safe-mode`, single-instance lock, System Tray e ciclo di vita della finestra).
  - *Worker-6* esegue la suite di test di integrazione (`tests/contract`), i test di sicurezza sui profili importati e prepara gli script di packaging.
- **WAVE 4 (Collaudo Hardware-in-the-Loop guidato dal Master con l'Utente):**
  - Il Master guida l'utente nel test reale di calibrazione della tastiera (verifica dei $54$ slot, ordine dei $2$ encoder, upload controllato e test Bluetooth).

### 2.5 Template del Prompt che il Master deve usare per delegare a un Sub-Agente

Quando il Master Agent avvia un Sub-Agente (tramite sub-agent tool / task runner), deve fornirgli un contesto rigoroso seguendo questo schema:

```text
[TASK SUB-AGENTE: <Nome Worker>]
1. Contesto: Stiamo costruendo CH57x Studio. Leggi la Sezione <X> di `SPEC_CH57X_STUDIO_MASTER.md`.
2. Perimetro di Scrittura Consentito: Puoi creare/modificare file SOLO dentro `<percorso_crate>/`. NON toccare altri crate.
3. Dipendenze e Contratti: Usa i tipi già definiti in `crates/domain` e rispetta le firme esatte dei trait in `crates/platform`.
4. Obiettivo Tecnico: <Elenco puntato dei file e delle funzioni da implementare>.
5. Criteri di Accettazione:
   - `cargo test --package <nome_crate>` deve passare al 100%.
   - `cargo clippy --package <nome_crate> -- -D warnings` deve dare zero warning.
   - Nessun `unwrap()` su input utente/IO (usa `Result` con errori tipizzati `thiserror`).
   - Nessun `todo!()` lasciato nel codice.
6. Restituisci un report conciso con i file modificati, i test aggiunti e gli eventuali blocchi riscontrati.
```

---

## 3. Architettura del Workspace Rust e Contratti dei Moduli

### 3.1 Albero delle Directory del Progetto

```text
ch57x-studio/
├── SPEC_CH57X_STUDIO_MASTER.md     # Questo documento (Fonte della Verità)
├── Cargo.toml                      # Workspace root
├── Cargo.lock
├── rust-toolchain.toml             # Toolchain stabile fissata (es. 1.85+)
├── bin/                            # Cartella opzionale per binario locale ch57x-keyboard-tool
├── crates/
│   ├── domain/                     # Zero dipendenze OS/GUI: modelli, 54 slot, validazione, capability types
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── hardware.rs         # Matrix 3x4, 2 Encoders (CCW, Press, CW), 3 Layers = 54 slots
│   │       ├── actions.rs          # HidNative, LaunchOrFocus, Workspace, Macro, ExecProcess, ContextDispatch
│   │       ├── triggers.rs         # 5 banchi F13..F24 (60 combinazioni) e TriggerMap
│   │       ├── capabilities.rs     # CapabilityMatrix per Windows / Linux X11 / Linux Wayland
│   │       └── errors.rs
│   ├── persistence/                # Salvataggio atomico JSON, migrazioni schema, Deployment Journal, Local Trust
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── storage.rs          # Directory standard OS (%APPDATA% su Win, XDG su Linux) + Portable mode
│   │       ├── journal.rs          # State machine di deployment e recovery dopo crash
│   │       └── trust.rs            # Hash approvati localmente per comandi ed eseguibili
│   ├── compiler/                   # Traduzione pura: Profile -> Trigger Allocation -> YAML + Diff
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── allocator.rs        # Allocatore deterministico stabile su 54 slot
│   │       ├── yaml_gen.rs         # Serializer YAML per ch57x-keyboard-tool
│   │       └── diff.rs             # Calcola se serve Flash Hardware o basta Host-Only Apply
│   ├── hardware/                   # Interfaccia USB (VID 0x1189) e wrapper processo ch57x-keyboard-tool
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── usb_probe.rs        # Rilevamento presenza/accesso USB e diagnostica UsbDk / udev
│   │       └── cli_adapter.rs      # Esecuzione sicura senza shell di validate, upload, show-keys
│   ├── platform/                   # Trait di astrazione OS + Backend Windows (100%) + Backend Linux
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── traits.rs           # Contratti unificati per Hotkey, Finestre, Monitor, Input, Overlay
│   │       ├── windows/            # #[cfg(target_os = "windows")] Implementazione Win32 completa
│   │       │   ├── mod.rs
│   │       │   ├── hotkeys.rs      # RegisterHotKey / UnregisterHotKey + message pump dedicato
│   │       │   ├── windows_api.rs  # EnumWindows, GetForegroundWindow, SetWindowPos, DWM frame bounds
│   │       │   ├── monitors.rs     # EnumDisplayMonitors, DPI per-monitor, work area
│   │       │   ├── input.rs        # SendInput Unicode/VirtualKey + controllo integrità UIPI
│   │       │   └── overlay.rs      # Flag Win32 WS_EX_TRANSPARENT | WS_EX_NOACTIVATE per HUD
│   │       └── linux/              # #[cfg(target_os = "linux")] Implementazione X11 + Wayland probe
│   │           ├── mod.rs
│   │           ├── x11_backend.rs  # x11rb per X11 (finestre attive, posizionamento, hotkey)
│   │           └── wayland_stub.rs # Rilevamento Wayland e ritorno strutturato CapabilityStatus
│   ├── runtime/                    # Motore in background: smistamento trigger, macro, workspace, context
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── dispatcher.rs       # Riceve Trigger -> risolve Slot -> esegue Action
│   │       ├── window_actor.rs     # Logica LaunchOrFocus + calcolo coordinate + retry/timeout
│   │       ├── workspace_actor.rs  # Ripristino multi-finestra con report parziale
│   │       ├── macro_actor.rs      # Esecutore step-by-step cancellabile con focus guard
│   │       └── context_actor.rs    # Valutazione regole ordinate per app in primo piano
│   ├── ui/                         # Interfaccia egui: Canvas 4x3+2, Inspector, Picker 3s, HUD, Doctor
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── state.rs            # Stato UI (EditingLayer, DirtyState, SelectedSlot)
│   │       ├── pad_canvas.rs       # Rendering interattivo 12 tasti + 3 LED + 2 Encoder (CCW/Press/CW)
│   │       ├── inspector.rs        # Editor contestuale slot selezionato
│   │       ├── doctor_panel.rs     # Diagnostica USB, UsbDk, CLI, Hotkey conflicts, CapabilityMatrix
│   │       └── hud_window.rs       # Cheat-Sheet OSD per 12 tasti + 2 encoder
│   └── app/                        # Binario eseguibile finale (`ch57x-studio`)
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs             # Parsing argomenti (--background, --doctor, --safe-mode), single-instance
│           ├── tray.rs             # System Tray icon e menu contestuale
│           └── coordinator.rs      # Ponte di comunicazione tra UI e Runtime
├── tests/
│   ├── contract/                   # Test sui 54 slot, YAML output, JSON schema
│   └── hardware/                   # Test #[ignore] da lanciare solo con tastiera collegata e consenso utente
├── packaging/
│   ├── windows/                    # Script build release Windows + istruzioni UsbDk
│   └── linux/                      # 99-ch57x.rules (TAG+="uaccess") + desktop entry
└── docs/
    ├── adr/
    ├── compatibility/
    └── agent_questions.md          # Registro domande del Master Agent per l'Utente
```

### 3.2 Contratti Rust Fondamentali (Da congelare nella Wave 0)

Il Master Agent deve definire questi contratti in `crates/domain` e `crates/platform/src/traits.rs` prima di avviare i Sub-Agenti:

```rust
// In crates/domain/src/hardware.rs
use serde::{Deserialize, Serialize};

pub const LAYER_COUNT: usize = 3;
pub const ROW_COUNT: usize = 3;
pub const COL_COUNT: usize = 4;
pub const BUTTONS_PER_LAYER: usize = ROW_COUNT * COL_COUNT; // 12
pub const ENCODERS_PER_LAYER: usize = 2;
pub const EVENTS_PER_ENCODER: usize = 3; // CCW, Press, CW
pub const SLOTS_PER_LAYER: usize = BUTTONS_PER_LAYER + (ENCODERS_PER_LAYER * EVENTS_PER_ENCODER); // 18
pub const TOTAL_SLOTS: usize = LAYER_COUNT * SLOTS_PER_LAYER; // 54

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LayerIndex(pub u8); // 0..=2 (mostrato in UI come Layer 1..=3)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ControlId {
    Button { row: u8, col: u8 }, // row: 0..=2, col: 0..=3
    Encoder { index: u8, event: EncoderEvent }, // index: 0 (Top), 1 (Bottom)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EncoderEvent {
    Ccw,
    Press,
    Cw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SlotId {
    pub layer: LayerIndex,
    pub control: ControlId,
}
```

```rust
// In crates/platform/src/traits.rs
use ch57x_domain::capabilities::{CapabilityMatrix, PlatformError};
use ch57x_domain::triggers::PhantomTrigger;
use ch57x_domain::actions::{WindowSelector, WindowPlacement, WindowActionResult, ForegroundAppInfo};
use std::path::Path;

pub trait PlatformBackend: Send + Sync {
    fn capability_matrix(&self) -> CapabilityMatrix;
    fn hotkeys(&self) -> &dyn HotkeyBackend;
    fn windows(&self) -> &dyn WindowManagerBackend;
    fn input(&self) -> &dyn InputInjectorBackend;
    fn autostart(&self) -> &dyn AutostartBackend;
}

pub trait HotkeyBackend: Send + Sync {
    /// Registra l'insieme di trigger richiesti dal profilo attivo.
    /// Restituisce la lista di eventuali trigger andati in conflitto col sistema operativo.
    fn register_triggers(&self, triggers: &[PhantomTrigger]) -> Result<Vec<PhantomTrigger>, PlatformError>;
    fn unregister_all(&self) -> Result<(), PlatformError>;
}

pub trait WindowManagerBackend: Send + Sync {
    /// Legge la finestra attualmente in primo piano (escludendo il processo corrente se richiesto).
    fn capture_foreground_window(&self, exclude_pid: u32) -> Result<ForegroundAppInfo, PlatformError>;
    /// Enumera tutte le finestre visibili idonee per uno Snapshot del Workspace.
    fn list_visible_windows(&self, exclude_pid: u32) -> Result<Vec<ForegroundAppInfo>, PlatformError>;
    /// Esegue la logica completa: trova finestra o avvia processo, posiziona (compensando DWM) e richiede focus.
    fn launch_or_place_window(
        &self,
        selector: &WindowSelector,
        placement: &WindowPlacement,
        launch_cmd: Option<(&Path, &[String], Option<&Path>)>,
        timeout_ms: u64,
    ) -> Result<WindowActionResult, PlatformError>;
}

pub trait InputInjectorBackend: Send + Sync {
    /// Verifica se la finestra target in primo piano ha un livello di integrità (UIPI) superiore al nostro.
    fn can_inject_into_foreground(&self) -> bool;
    fn tap_key(&self, key_token: &str) -> Result<(), PlatformError>;
    fn chord_keys(&self, modifiers: &[String], key_token: &str) -> Result<(), PlatformError>;
    fn type_unicode_text(&self, text: &str) -> Result<(), PlatformError>;
}

pub trait AutostartBackend: Send + Sync {
    fn is_enabled(&self) -> Result<bool, PlatformError>;
    fn set_enabled(&self, enabled: bool, exe_path: &Path) -> Result<(), PlatformError>;
}
```

---

## 4. Modello Dati ($54$ Slot) e Allocazione Trigger ($60$ Combinazioni)

### 4.1 Mappa Canonica dei $54$ Slot

Ogni profilo ha esattamente $3$ Layer (`LayerIndex(0)`, `LayerIndex(1)`, `LayerIndex(2)`).
All'interno di ciascun Layer, l'ordine canonico degli $18$ slot è:
- **Slot $0..=11$ (Matrice Pulsanti $3 \times 4$):**
  - Riga $0$: `(r0, c0)`, `(r0, c1)`, `(r0, c2)`, `(r0, c3)`
  - Riga $1$: `(r1, c0)`, `(r1, c1)`, `(r1, c2)`, `(r1, c3)`
  - Riga $2$: `(r2, c0)`, `(r2, c1)`, `(r2, c2)`, `(r2, c3)`
- **Slot $12..=14$ (Encoder Superiore `index: 0`):** `Ccw`, `Press`, `Cw`
- **Slot $15..=17$ (Encoder Inferiore `index: 1`):** `Ccw`, `Press`, `Cw`

*(Nota di Calibrazione Hardware: Il parametro `encoder_order_swapped: bool` nel `DeviceDescriptor` permette di invertire la serializzazione di Encoder $0$ ed Encoder $1$ nel caso in cui il PCB fisico dell'utente mappi la manopola inferiore come primo elemento dell'array YAML `knobs`).*

### 4.2 I $5$ Banchi di Trigger Fantasma ($60$ Slot Disponibili)

Quando uno slot è configurato come `SlotBinding::HostActionRef(ActionId)`, il compilatore gli assegna un `PhantomTrigger` dal pool di $60$ combinazioni:

| Banco | Modificatori | Tasti Funzione | Sintassi YAML (`ch57x-keyboard-tool`) | Indici Pool |
|---|---|---|---|---|
| **Banco A** | Nessuno | `F13` $\rightarrow$ `F24` | `"f13"` .. `"f24"` | $0 ..= 11$ |
| **Banco B** | `Ctrl` | `F13` $\rightarrow$ `F24` | `"ctrl-f13"` .. `"ctrl-f24"` | $12 ..= 23$ |
| **Banco C** | `Shift` | `F13` $\rightarrow$ `F24` | `"shift-f13"` .. `"shift-f24"` | $24 ..= 35$ |
| **Banco D** | `Alt` | `F13` $\rightarrow$ `F24` | `"alt-f13"` .. `"alt-f24"` | $36 ..= 47$ |
| **Banco E** | `Ctrl + Shift` | `F13` $\rightarrow$ `F24` | `"ctrl-shift-f13"` .. `"ctrl-shift-f24"` | $48 ..= 59$ |

**Regola di Persistenza del Trigger:**
Il trigger assegnato a uno `SlotId` viene salvato nella `trigger_allocation` del profilo JSON. Se l'utente cambia l'applicazione da aprire o le coordinate della finestra su quello stesso tasto, **il trigger non cambia** e l'applicazione aggiorna istantaneamente il demone Windows senza chiedere di collegare il cavo USB per riflashare la tastiera.

### 4.3 Tipi di Azione Supportati (`ActionDefinition`)

1. **`SlotBinding::Disabled`**:
   Slot disattivato. Poiché `ch57x-keyboard-tool` richiede una stringa valida per ogni posizione della matrice YAML, uno slot `Disabled` può essere compilato(a seconda del supporto verificato nella Wave 0) con il token neutro supportato dal tool o con un trigger passivo ignorato dal demone host.
2. **`SlotBinding::HidNative(HidAction)`**:
   Vive interamente nella memoria flash della tastiera (funziona anche su PC dove `ch57x-studio` non è installato).
   - `KeyboardChord { modifiers: Vec<HidMod>, key: HidKey }` (es. `"ctrl-c"`, `"ctrl-shift-esc"`)
   - `ConsumerMedia(MediaToken)` — La UI mostra nomi amichevoli (`Volume +`, `Volume -`, `Muto`, `Play/Pausa`, `Traccia Successiva`, `Traccia Precedente`) e il compilatore li traduce nei token reali del tool (`"volumeup"`, `"volumedown"`, `"mute"`, `"playpause"`, `"nextsong"`, `"previoussong"`).
   - `MouseAction(MouseToken)` — Tradotto in `"wheelup"`, `"wheeldown"`, `"click"`, `"rclick"`, `"mclick"`.
3. **`SlotBinding::HostActionRef(ActionId)`** (Gestito dal Demone Rust su PC):
   - **`LaunchOrFocusWindow`**: Avvia un eseguibile (se non è già aperto), trova la finestra tramite `WindowSelector` e la posiziona usando `WindowPlacement` (coordinate assolute in pixel oppure coordinate normalizzate $0.0 \rightarrow 1.0$ rispetto alla `WorkArea` del monitor selezionato).
   - **`RestoreWorkspace`**: Lista di `WorkspaceTarget`, ciascuno con il proprio `WindowSelector`, `WindowPlacement` e `LaunchRecipe` opzionale (`program`, `args`, `working_dir`).
   - **`MacroSequence`**: Lista sequenziale di step (`KeyTap`, `KeyChord`, `TypeText`, `ClipboardPaste`, `DelayMs`, `WaitForWindow`) con `ConcurrencyPolicy` (`IgnoreIfRunning` default, `RestartCurrent`, `QueueBounded`) e `require_initial_foreground_match: bool` (interrompe la macro se l'utente clicca su un'altra finestra mentre la macro sta scrivendo).
   - **`ExecProcess`**: Esecuzione diretta sicura (`program: PathBuf`, `args: Vec<String>`, `working_dir: Option<PathBuf>`, `timeout_ms: u64`, `hide_console_window: bool` con flag Windows `CREATE_NO_WINDOW`).
   - **`ContextDispatch`**: Lista ordinata per priorità di regole `ContextRule { match_exe_name, match_title_contains, target_action_id }` più un `fallback_action_id` obbligatorio.
   - **`ShowHud`**: Mostra su schermo per $1.5\text{ s}$ (o finché premuto) l'HUD OSD con la mappa dei $12$ tasti e dei $2$ encoder.

---

## 5. Compilatore YAML & Contratto `ch57x-keyboard-tool`

### 5.1 Esempio Completo e Validato a $54$ Slot

Il modulo `crates/compiler/src/yaml_gen.rs` deve produrre un file YAML conforme allo schema di `ch57x-keyboard-tool`:

```yaml
model: ch57x-2
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
      - ccw: "volumedown"
        press: "mute"
        cw: "volumeup"
      - ccw: "wheelup"
        press: "ctrl-f13"
        cw: "wheeldown"
  - buttons:
      - ["ctrl-f14", "ctrl-f15", "ctrl-f16", "ctrl-f17"]
      - ["ctrl-f18", "ctrl-f19", "ctrl-f20", "ctrl-f21"]
      - ["ctrl-f22", "ctrl-f23", "ctrl-f24", "shift-f13"]
    knobs:
      - ccw: "shift-f14"
        press: "shift-f15"
        cw: "shift-f16"
      - ccw: "shift-f17"
        press: "shift-f18"
        cw: "shift-f19"
  - buttons:
      - ["shift-f20", "shift-f21", "shift-f22", "shift-f23"]
      - ["shift-f24", "alt-f13", "alt-f14", "alt-f15"]
      - ["alt-f16", "alt-f17", "alt-f18", "alt-f19"]
    knobs:
      - ccw: "alt-f20"
        press: "alt-f21"
        cw: "alt-f22"
      - ccw: "alt-f23"
        press: "alt-f24"
        cw: "ctrl-shift-f13"
```

### 5.2 Differenziale Intelligente: "Host-Only Apply" vs "Hardware Flash"

Ogni volta che l'utente preme **"Applica Modifiche"**, `crates/compiler/src/diff.rs` calcola l'hash SHA-256 del YAML canonico generato (`new_yaml_hash`) e lo confronta con `last_uploaded_yaml_hash` salvato nel Journal:
1. **Se `new_yaml_hash == last_uploaded_yaml_hash`:**
   La mappa fisica sulla tastiera è già aggiornata! Il programma applica immediatamente le nuove azioni finestre/macro nel demone Windows (anche se la tastiera è collegata solo via Bluetooth) e mostra: *"✅ Profilo Host aggiornato istantaneamente (nessuna scrittura USB necessaria)"*.
2. **Se `new_yaml_hash != last_uploaded_yaml_hash`:**
   Il programma avvisa che la modifica richiede l'aggiornamento della memoria della tastiera via cavo USB-C, mostra la tabella delle differenze (quali slot hanno cambiato binding HID/Trigger) e abilita il pulsante **"⚡ Scrivi sulla Tastiera via USB"**.

---

## 6. Implementazione Piattaforma: Windows $100\%$ (Primario) + Linux Ready (Secondario)

### 6.1 Specifiche Tecniche Backend Windows (`crates/platform/src/windows/`)

Il backend Windows deve essere implementato al $100\%$ usando il crate ufficiale Microsoft `windows` (`windows::Win32::*`):

1. **Consapevolezza DPI (Per-Monitor DPI Aware v2):**
   All'avvio del processo, o tramite manifesto applicativo embedded, impostare `DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2` così che `GetWindowRect`, `EnumDisplayMonitors` e `SetWindowPos` operino sempre in pixel fisici reali senza distorsioni su configurazioni multi-monitor con scaling diversi (es. $100\%$ sul monitor $1$ e $150\%$ sul monitor $2$).
2. **Compensazione Bordi Invisibili Windows 10/11 (DWM Extended Frame Bounds):**
   Su Windows 10 e 11, `SetWindowPos` include nel rettangolo della finestra l'ombra trasparente di ridimensionamento (~$7\text{ px}$ a sinistra, destra e in basso), causando un fastidioso spazio vuoto quando si affiancano due finestre a $X = 0$.
   - Durante la cattura (`Window Picker`) e il posizionamento (`SetWindowPos`), il codice interroga `DwmGetWindowAttribute(hwnd, DWMWA_EXTENDED_FRAME_BOUNDS, ...)` per calcolare l'offset esatto tra il frame visibile e il frame logico (`border_offset = rect_win32 - rect_dwm_visible`) e applica la compensazione affinché il bordo **visibile** della finestra cada esattamente alle coordinate `(X, Y, W, H)` richieste dall'utente.
3. **Gestione Robusta di Avvio, Stato e Focus (`launch_or_place_window`):**
   - Se la finestra target è minimizzata (`IsIconic(hwnd)`), chiama `ShowWindow(hwnd, SW_RESTORE)`.
   - Se la finestra è massimizzata (`IsZoomed(hwnd)`), chiama `ShowWindow(hwnd, SW_RESTORE)` prima di applicare un rettangolo custom (altrimenti Windows ignora le nuove dimensioni).
   - Chiama `SetWindowPos(hwnd, None, adj_x, adj_y, adj_w, adj_h, SWP_NOZORDER | SWP_NOACTIVATE)`.
   - Se è richiesto il focus, tenta `SetForegroundWindow(hwnd)` (usando `AllowSetForegroundWindow` o il trick lecito del thread di input quando consentito) e controlla con `GetForegroundWindow()` se il focus è stato effettivamente concesso, riportando l'esito reale in `WindowActionResult`.
4. **Thread Dedicato per `RegisterHotKey`:**
   Su Windows `RegisterHotKey` è legata alla coda messaggi del thread chiamante. Il modulo `windows/hotkeys.rs` avvia un thread leggero dedicato con un message loop `GetMessageW` che ascolta `WM_HOTKEY` (usando il flag `MOD_NOREPEAT` per i pulsanti singoli e gestendo i tick rapidi per gli encoder) e invia gli eventi al `RuntimeDispatcher` tramite un canale `crossbeam_channel` / `tokio::sync::mpsc`.
5. **Esecuzione Processi Senza Finestra Console (`CREATE_NO_WINDOW`):**
   Quando un'azione `ExecProcess` viene lanciata su Windows con `hide_console_window: true`, usa `std::os::windows::process::CommandExt::creation_flags(0x08000000)` (`CREATE_NO_WINDOW`) per evitare che compaia per una frazione di secondo un terminale nero sullo schermo.
6. **HUD Overlay Click-Through e No-Focus:**
   La finestra HUD su Windows applica gli stili estesi Win32 `WS_EX_TOPMOST | WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE` al proprio `HWND`. In questo modo l'overlay appare sopra qualsiasi programma, non compare nella barra delle applicazioni, i click del mouse lo attraversano e non ruba mai il focus alla finestra su cui l'utente sta lavorando.

### 6.2 Specifiche Tecniche Compatibilità Linux (`crates/platform/src/linux/`)

Per garantire che il programma possa essere compilato e utilizzato anche su Linux:
1. **Rilevamento Sessione a Runtime:**
   `linux/mod.rs` ispeziona `XDG_SESSION_TYPE` e `WAYLAND_DISPLAY` / `DISPLAY`.
2. **Su Linux X11 (`x11_backend.rs`):**
   Usa il crate puro Rust `x11rb` per interrogare `_NET_ACTIVE_WINDOW`, `_NET_WM_PID`, leggere `/proc/<pid>/exe`, ottenere la geometria e inviare richieste `_NET_MOVERESIZE_WINDOW` / `_NET_ACTIVE_WINDOW` al Window Manager, oltre a registrare i grab per `F13..F24`.
3. **Su Linux Wayland (`wayland_stub.rs`):**
   Restituisce una `CapabilityMatrix` che segnala `WindowPlacement: UnsupportedOnWaylandCompositor` e `WindowPicker: UnsupportedOnWaylandCompositor` (disabilitando con un tooltip esplicativo in UI i controlli non permessi dal protocollo Wayland standard), mantenendo invece attivi al $100\%$ il configuratore grafico dei $54$ slot, la generazione YAML, il flash hardware USB via `ch57x-keyboard-tool` e l'esecuzione di processi/script.

---

## 7. Design dell'Interfaccia Utente (`egui`) e Flussi Utente

### 7.1 Struttura della Finestra Principale

1. **Top Bar (Barra di Stato e Azioni Globali):**
   - **Selettore Profilo:** Menu a tendina con pulsanti `Nuovo`, `Salva`, `Importa`, `Esporta`.
   - **Selettore Layer di Editing:** Tre pulsanti `Layer 1`, `Layer 2`, `Layer 3`. Accanto appare un badge separato: `Layer Fisico: Sconosciuto` oppure `Layer Fisico Inferito: Layer 2 (visto 4s fa)`.
   - **Badge Stato USB:**
     - `🟢 USB Pronta (1189:8890) - UsbDk OK`
     - `🟡 USB Rilevata ma Accesso Negato (Apri Device Doctor)`
     - `⚪ Cavo USB Scollegato (Modifiche Host attive, Flash disabilitato)`
   - **Pulsante Primario Contestuale:**
     - Se i trigger hardware non sono cambiati: **"💾 Salva e Attiva sul PC (Istantaneo)"**.
     - Se i trigger hardware sono cambiati: **"⚡ Valida, Flasha su Tastiera & Attiva"**.
   - **Toggle Emergenza:** **"⏸️ Pausa Runtime"** e **"👁️ Mostra HUD"**.

2. **Pannello Sinistro — Visual Pad Canvas (`pad_canvas.rs`):**
   Riproduce fedelmente il case acrilico fumè della tastiera:
   - **A Sinistra:** Griglia $4 \text{ colonne} \times 3 \text{ righe}$ di keycap bianchi quadrati. Ogni tasto mostra:
     - L'etichetta scelta dall'utente (es. `VS Code SX`),
     - Un badge colorato del tipo di azione (`HID`, `WIN`, `WSP`, `MAC`, `CMD`, `CTX`),
     - Il codice tecnico sottostante in piccolo (es. `f13` o `ctrl-c`).
   - **Al Centro:** I $3$ LED indicatori verticali.
   - **A Destra:** I $2$ grandi encoder circolari in alluminio. Ciascun encoder è suddiviso visivamente in $3$ pulsanti interattivi distinti e grandi:
     - Arco Sinistro: `↺ CCW` (Rotazione Antioraria)
     - Cerchio Centrale: `● PRESS` (Pressione Manopola)
     - Arco Destro: `↻ CW` (Rotazione Oraria)

3. **Pannello Destro — Inspector dello Slot Selezionato (`inspector.rs`):**
   Cliccando su uno qualsiasi dei $18$ controlli del layer corrente, l'Inspector permette di configurare:
   - **Etichetta e Colore identificativo** (usati nel Canvas e nell'HUD su schermo).
   - **Selettore Tipo Binding:** `Disabilitato` | `Tasto/Media Nativo (On-Board)` | `Apri e Posiziona Finestra` | `Snapshot Workspace (Multi-Finestra)` | `Sequenza Macro` | `Esegui Programma/Script` | `Azione Dinamica per App (Context-Aware)`.
   - **Pannello "Apri e Posiziona Finestra":**
     - Pulsante **"🎯 Cattura Finestra Attiva dopo 3 Secondi"** (avvia un conto alla rovescia non bloccante con beep/indicatore visivo; l'utente clicca sulla finestra desiderata e i campi `Percorso .exe`, `Monitor`, `X`, `Y`, `Larghezza`, `Altezza` si compilano da soli).
     - Pulsanti di **Quick Snap**: `Metà Sinistra`, `Metà Destra`, `Schermo Intero`, `Quadrante Alto-SX`, `Quadrante Basso-DX`.
     - Pulsante **"▶️ Testa Azione Ora"** per verificare immediatamente apertura e posizionamento della finestra senza premere il tasto fisico.

4. **Pannello "Device Doctor" (`doctor_panel.rs`):**
   Una vista diagnostica chiara che verifica con semafori Verdi/Gialli/Rossi:
   1. Presenza e hash del binario `ch57x-keyboard-tool`.
   2. Presenza del driver `UsbDk` su Windows (o regole `udev` su Linux).
   3. Visibilità USB del Macropad (`1189:8890` / `8840` / `8842`).
   4. Stato registrazione delle Hotkey `F13..F24` su Windows (segnala se un altro programma sul PC sta già occupando una scorciatoia).
   5. Procedura guidata di **"Calibrazione Hardware Iniziale"** per testare i $12$ tasti, l'ordine dei $2$ encoder e i $3$ layer.

---

## 8. Roadmap Operativa a Gate e Definition of Done (DoD)

Il Master Agent deve guidare lo sviluppo attraverso questi $5$ Gate di Verifica:

| Gate | Fase / Wave | Criterio di Superamento (Obbligatorio prima di procedere) |
|---|---|---|
| **Gate 0** | **Discovery & Setup Workspace** | Il Master ha raccolto le risposte dell'utente su ambiente/binario CLI, ha creato il Workspace Cargo e `cargo check --workspace` compila con i contratti `domain` e `platform` congelati. |
| **Gate 1** | **Core Offline (Wave 1)** | I $54$ slot sono modellati, l'allocatore assegna i $60$ trigger senza collisioni, il compilatore genera il YAML valido per `ch57x-keyboard-tool` e i backend `platform::windows` e `platform::linux` compilano entrambi senza warning. |
| **Gate 2** | **Motore Runtime & GUI (Wave 2)** | Il Canvas $4 \times 3 + 2$ permette di configurare tutti i $54$ slot, il Window Picker cattura una finestra reale su Windows in $3\text{ s}$ e il pulsante *"Testa Azione Ora"* sposta/avvia correttamente una finestra compensando i bordi DWM. |
| **Gate 3** | **App Completa, Tray & HUD (Wave 3)** | Il binario supporta `--background`, `--doctor`, `--safe-mode`, l'icona System Tray funziona, l'HUD trasparente non ruba il focus e i test di sicurezza sui profili importati passano al $100\%$. |
| **Gate 4** | **Collaudo Hardware con l'Utente (Wave 4)** | Con il consenso esplicito dell'utente, il Device Doctor valida e carica una configurazione sul Macropad fisico e si verifica il funzionamento sia via USB che via Bluetooth. |

### Domande Iniziali che il Master Agent deve porre all'Utente all'avvio della Wave 0:

1. *"Hai già scaricato `ch57x-keyboard-tool.exe` (e installato `UsbDk` su Windows) sul tuo PC, oppure vuoi che ti guidi io nel download/verifica prima di fare i test hardware?"*
2. *"Vuoi che iniziamo subito a costruire e compilare tutta l'applicazione (Wave 1, 2 e 3) in modalità sicura offline così puoi già aprire la GUI e testare il posizionamento finestre su Windows, rimandando il collegamento USB della tastiera al Gate 4?"*