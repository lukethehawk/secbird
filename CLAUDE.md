# CLAUDE.md — Secretarybird / Razer HyperSpeed: memoria tecnica e handoff
**Aggiornato: 29 ottobre 2026** (seconda sessione: diagnosi del doppio standby dal log reale e correzione nel branch `experiment-spectrum-wake-double-standby`, vedi **§4.1**). Lingua preferita per interazione con l'utente: **italiano**.

- Fork dell'utente: https://github.com/lukethehawk/secbird
- Progetto originale: https://github.com/infinityInfinite/secbird
- Questo documento raccoglie fatti osservati su hardware reale, implementazioni, tentativi falliti, branch Git e prossimi passi. **Leggilo prima di modificare il codice.**
- **Non equiparare CI verde a funzionamento reale.** La sincronizzazione automatica RGB è solo **parzialmente funzionante** sul PC dell'utente, come precisato di seguito.

## 1. Stato corrente: verifica hardware reale (IMPORTANTISSIMO)

| Funzione | Stato verificato |
| --- | --- |
| BlackWidow V4 Low-Profile HyperSpeed full-size, wireless PID 02C9 | Riconosciuta da Secretarybird; RGB funzionante; sul suo dongle originale separato. |
| BlackWidow in modalità cablata PID 02CC | Supporto integrato upstream (PR #31). |
| Basilisk V4 HyperSpeed wireless PID 00EA | Riconosciuto sul proprio dongle originale separato; batteria e DPI letti, RGB controllabile. |
| Ricevitore HyperSpeed Multi-Device 02C9 con entrambi associati | Discovery sperimentale funzionava, ma **instabilità di connessione tastiera** osservata anche con Secretarybird chiuso. Non ripristinare senza motivo. |
| Disassociazione mirata di 00EA dal dongle 02C9 | **SUCCESSO.** Utility standalone 00:42; ora mouse e tastiera usano dongle distinti. |
| Mouse: numero LED | **UNO SOLO**: Underglow (parte posteriore inferiore), **LED ID 10 decimale / 0x0A**. Nessun LED nel logo o nella rotella. |
| Mouse RGB Static, Spectrum Cycling, Wave, Breathing | **FUNZIONANO** (feedback diretto dell'utente). |
| Mouse RGB Reactive, Starlight | **NON funzionano correttamente**: Reactive non reagisce; Starlight rimane fisso. Esclusi dall'UI e bloccati lato HID nella patch RGB. |
| Basilisk standby | **FUNZIONA**: con timer impostato a **1 minuto**, si è addormentato dopo ~1 minuto **anche con Secretarybird aperto**. Non attribuire automaticamente malfunzionamenti a polling software. |
| Spectrum Cycling sincronizzazione automatica: **un solo dispositivo va in standby, poi si risveglia** | **FUNZIONA: dopo il risveglio i due colori si riallineano.** Ultima correzione esplicita dell'utente, 29/10/2026. |
| Spectrum Cycling sincronizzazione automatica: **entrambi vanno in standby, poi uno viene riattivato e dopo l'altro** | **NON FUNZIONA**: il caso è riproducibile secondo il feedback. Deve funzionare **indipendentemente da quale si sveglia prima**. È il bug prioritario. |
| PR di correzione RGB verso upstream | Branch pronto, **PR non ancora aperta** al controllo del 29/10; creazione automatica via integrazione GitHub fallita con HTTP 403. |
| Correzione doppio standby, branch `experiment-spectrum-wake-double-standby` | Causa principale **confermata dal log reale** (§4.1). Nuova macchina a stati + rescan mirato + retry limitati + log diagnostici. CI Windows: vedi §6. **NON ancora verificata su hardware: non dichiararla risolta.** |

**Precisazione critica ricevuta durante l'handoff (29/10/2026):**
> «Errata corrige, non ha funzionato quando entrambi i device sono andati in stand by e ne ho riattivato uno alla volta, se invece ne va solo uno e lo riattivo, si syncano correttamente».

Interpretazione corretta: **il trigger di wake e il riavvio Spectrum funzionano almeno in alcuni casi**. Non descrivere la funzione come universalmente guasta; il difetto riguarda la transizione da **entrambi addormentati** a **entrambi nuovamente attivi**.

## 2. Hardware, sistema, configurazione da preservare

- Sistema: Windows 11; Secretarybird come sostituto leggero di Razer Synapse. L'utente usa eseguibili compilati con GitHub Actions, lanciati dalla cartella scaricata.
- Tastiera: **Razer BlackWidow V4 Low-Profile HyperSpeed**, layout full-size. Dongle originale: Vendor **1532**, PID fisico **02C9**, HID interface **2**, txid **FF** per la tastiera. FW wireless osservato **1.4.1.0**. Cablata PID **02CC**.
- Mouse: **Razer Basilisk V4 HyperSpeed**, NON V4 Pro e NON V3. PID logico wireless **00EA**, txid **1F** quando era condiviso su dongle 02C9. Firmware osservato **1.0.16.0**. Database include anche PID **00EB** per la variante cablata, non validata separatamente sul banco prova.
- **Stato attuale: due dongle ORIGINALI distinti** (quello della tastiera e quello del mouse). Il mouse è stato disassociato dal dongle della tastiera. Non riaccoppiarli per i test RGB.
- Non pubblicare seriali integrali dei dispositivi; in passato erano riportati solo in forma ridotta.
- Obiettivo: RGB corretto, batteria/DPI/sleep, e **Spectrum Cycling in fase** dopo il wake in qualsiasi ordine, senza traffico HID periodico che comprometta il risparmio energetico.

## 3. Storia del lavoro e conferme tecniche

### 3.1 Supporto tastiera e contributi già integrati upstream

- PR #28: supporto BlackWidow V4 Low-Profile HyperSpeed wireless 02C9, merged il 18/10/2026:
  https://github.com/infinityInfinite/secbird/pull/28
- PR #31: BlackWidow modalità cablata PID 02CC, merged il 19/10/2026:
  https://github.com/infinityInfinite/secbird/pull/31
- PR #32: risoluzione DPI sui dispositivi non-mouse, merged il 19/10/2026:
  https://github.com/infinityInfinite/secbird/pull/32
- Questi cambiamenti sono già nel repository originale; non riproporli.

### 3.2 Ricevitore Multi-Device: discovery con due PID logici

- Synapse aveva associato **mouse e tastiera allo stesso dongle fisico tastiera 02C9**. Secretarybird vedeva inizialmente un solo dispositivo o sceglieva mouse su txid 1F prima della tastiera.
- Probe GET non distruttivi hanno mostrato **stessa interfaccia fisica 02C9/2** con indirizzamento differente:
  - Transaction ID **FF** → tastiera logica 02C9.
  - Transaction ID **1F** → mouse logico 00EA, solo se connesso/attivo.
- GET **00:BF** (“wireless paired devices”, response size 0x50): formato [count, connected_flag, pid_hi, pid_lo, ...]. Esempi:
  - `02 01 00 EA 01 02 C9` = 2 dispositivi collegati: mouse 00EA + tastiera 02C9.
  - `02 00 00 EA 01 02 C9` = mouse ancora associato ma spento/disconnesso, tastiera connessa.
- Implementazione in branch `support-02c9-shared-hyperspeed-receiver`; entry `crates/secbird-core/src/device.rs`: `discover_blackwidow_shared_receiver`, `probe_exact`, `connected_mouse_pid`, `shared_receiver_mouse` e `strict_txid`. Matching strict del reply txid per evitare che comandi del mouse finiscano sul keyboard HID e viceversa.
- Branch `test-02c9-shared-hyperspeed-receiver` con CI Windows, test report e editor. In modalità condivisa apparivano **2 dispositivi** ma 3 zone RGB perché la mappatura Basilisk precedente era ERRATA.
- **Problema hardware indipendente:** tastiera occasionalmente perdeva HyperSpeed (tasto lampeggiante, talvolta necessario cavo USB), e **capitava anche con Secretarybird CHIUSO**. Non affermare con certezza che sia causato dal programma. Multi-Device non è configurazione raccomandata attualmente.

### 3.3 Utility di unpair mirato: comando 00:42 CONFERMATO

- Fonte iniziale: OpenRazer, `driver/razerchromacommon.c`, funzione `razer_chroma_misc_set_hyperpolling_wireless_dongle_unpair(unsigned short pid)`; SET vendor **00:42**, 2 byte PID big-endian.
- Implementazione standalone, volutamente non agganciata alla UI:
  Branch https://github.com/lukethehawk/secbird/tree/tool-unpair-00ea-from-02c9
  File `crates/secbird/src/bin/secbird-unpair-shared-mouse.rs`.
- Guardie: verifica esattamente un dongle fisico 02C9 su interfaccia 2, txid FF, lista paired [tastiera 02C9 connessa, mouse 00EA presente]; di default **sola lettura**; flag `--execute`, conferma interattiva esatta `UNPAIR 00EA`; invia UNA sola SET 00:42 (payload `00 EA`) senza retry e poi rilegge 00:BF.
- Risultato osservato:
  ```text
  Preflight OK: keyboard 02C9 present, Basilisk 00EA still listed.
  00:42 response: Success (reply txid FF)
  Receiver 1532:02C9 paired-device table:
    PID FFFF: not currently connected (still listed)
    PID 02C9: connected
  RESULT: Basilisk 00EA is absent; keyboard 02C9 remains paired.
  ```
- L'utente ha confermato: il mouse **non si collega più** al dongle tastiera e funziona di nuovo soltanto collegando il proprio dongle. `FFFF` è uno slot libero.
- Utility originariamente definita “non ancora validata su 02C9” nei messaggi/commenti; **il test fisico ha successivamente validato questa singola operazione**, non generalizzarla ad altri dongle.
- **NON ESEGUIRE nuovamente unpair** se il pairing attuale è funzionante; rischio configurazione periferiche.

### 3.4 Correzione RGB Basilisk: verificata e pronta per PR

- Database iniziale (ERRATO): PID 00EA/00EB `leds=[{name:"logo",id:4},{name:"scroll",id:1}]`.
- Il Basilisk V4 HyperSpeed fisico ha **una sola zona Underglow** nella parte inferiore. GET LED dal mouse mostrava solo **ID 10 decimale**. Modifica:
  ```json
  "leds": [{"name": "underglow", "id": 10}],
  "unsupported_effects": ["reactive", "starlight"]
  ```
- Colori/effect reali testati nel UI: **Static, Spectrum Cycling, Wave, Breathing funzionano**; **Reactive non va; Starlight rimane fisso**. Non inventare altri test.
- Patch UI+core: campo opzionale `DeviceSpec.unsupported_effects` con default lista vuota; filtro scelta degli effetti nell'editor; enforcement in `Device::set_effect` prima di inviare HID; vecchi profili con effetti non supportati ignorati in fase di apply; test DB e test di simulazione per bloccare traffico HID.
- Branch **solo RGB** https://github.com/lukethehawk/secbird/tree/fix-basilisk-v4-hyperspeed-rgb
  Commit **8819d13a6444b8384c730ee248f49a91bbc30235** (basato su upstream 5c6c37940d0d7ee9e252cb8fbf9801a893d65cc4).
- Esattamente **6 file** della PR: `crates/secbird-core/devices.json`, `crates/secbird-core/src/db.rs`, `crates/secbird-core/src/effects.rs`, `crates/secbird-core/src/profile.rs`, `crates/secbird-tray/src/server.rs`, `crates/secbird-tray/src/editor.html`.
- **Non contiene** alcuna parte della gestione sperimentale dongle condiviso o sync automatico.
- PR upstream **non ancora aperta al 29/10/2026**: il connettore GitHub non ha permesso POST cross-repo (HTTP 403 Resource not accessible by integration).
  Aprire manualmente su https://github.com/infinityInfinite/secbird/compare/main...lukethehawk:fix-basilisk-v4-hyperspeed-rgb?expand=1
- Build/test patch RGB: https://github.com/lukethehawk/secbird/actions/runs/40760671241, **success**.

### 3.5 Standby Basilisk

- Software: `crates/secbird-core/src/features.rs`:
  - `sleep_timer()` GET **07:03**, 2 byte secondi BE, minuti = div_ceil(60).
  - `set_sleep_minutes(m)` SET **07:03** con `m*60` e GET per verificare il readback.
  - Auto-sleep GET/SET **07:08** su dispositivi che lo supportano.
- Feedback: con **5 minuti** il Basilisk si è effettivamente sospeso, però apparentemente dopo più tempo; senza cronometrare la causa è indeterminata.
- Test più importante: con **1 minuto** impostato tramite Secretarybird, **il Basilisk è andato in sleep dopo circa 1 minuto con Secretarybird aperto**. Quindi NON c'è evidenza di un bug sistematico che impedisce sleep.
- Tray: controllo batteria ogni **5 min**; remapping check ogni **20 sec** (ma `buttons_lost` evita l'HID se nel profilo non ci sono rimappature); tentativo refresh per-key ogni **3 sec** solo su tastiere CABLATE, perché quelle wireless devono dormire; browser editor fa `/api/live` ogni secondo ma legge cache DPI, non interroga continuamente il mouse.
- Una GET batteria, in linea teorica, potrebbe resettare idle, **ma questa non è una causa dimostrata**. Evitare di disabilitare funzioni senza riprodurre problema.

## 4. Esperimento Spectrum Cycling: dettagli, implementazione e bug aperto

### Requisito funzionale

L'utente vuole che i due dispositivi con Spectrum Cycling ritornino in fase dopo risvegli indipendenti, sia mouse→tastiera sia tastiera→mouse. Non basta risincronizzare quando si risveglia solo uno e l'altro è già attivo: **deve funzionare anche quando entrambi sono stati dormienti e vengono svegliati uno per volta**.

### Evoluzione

1. Ricerca codice originale: la sync globale `actions::sync_effect` riapplica i comandi built-in quasi insieme; i singoli device hanno timer hardware autonomi, quindi possono driftare. Il rescan/apply alla riconnessione può risettare un solo timer.
2. Primo prototipo `experiment-spectrum-resync-on-wake` ascoltava input del solo mouse tramite HID passivo dopo inattività, con pulsante `Re-sync now`. Branch CI success: https://github.com/lukethehawk/secbird/actions/runs/40767796461. Non copriva robustamente entrambi gli ordini.
3. Secondo prototipo, stesso branch: **Win32 Raw Input in background** con `RIDEV_INPUTSINK`, `RegisterRawInputDevices` per usage 01:02 (mouse) e 01:06 (keyboard), `WM_INPUT`, lettura sola header via `GetRawInputData(..., RID_HEADER, ...)`, `GetRawInputDeviceInfoW(..., RIDI_DEVICENAME, ...)` e filtro device name per VID 1532/PID 02C9 (tastiera) oppure 00EA (mouse). `WakeTracker` tiene timestamp ultima attività per ciascun PID e programma un timer unico di **1.500 ms** una volta rilevato risveglio e attività “recente” di entrambi.
4. `Engine::resync_spectrum(None)` esegue un nuovo SET 0F:02 Effect::Spectrum sulle zone il cui **profilo effettivo** richiede Spectrum, se ci sono almeno **due distinti dispositivi** compatibili e il profilo non ha `lighting_off`. Non modifica profili.
5. UI Settings: toggle **“Re-sync Spectrum when either device wakes (experimental)”**, default **OFF** memorizzato in `tray.json`, e pulsante **“Re-sync now”** per manuale. Il listener Raw Input viene registrato SOLO all'avvio della tray se il toggle era già true: serve chiudere/riavviare Secretarybird dopo averlo attivato.
6. Test unitari in `crates/secbird-tray/src/wake_sync.rs` per i due ordini e per dispositivi esterni. Versione compilata/testata con Windows CI verde:
   https://github.com/lukethehawk/secbird/actions/runs/40771276269
   Commit **09b0cc35b55db2e8213fb7536381fc0616d22f63** (prima dell'aggiunta di questo documento).
7. **RISULTATO REALE, secondo correzione utente:** se **un solo** device entra in standby e si riattiva, **si sincronizzano correttamente**. Se **entrambi** si addormentano, poi si attiva uno alla volta (in un ordine o nell'altro), **NON si sincronizzano correttamente**. Questo è il comportamento esatto che Claude deve riprodurre e risolvere; non trasformarlo in “non funziona in assoluto”.

### Sorgenti del prototipo da esaminare

- `crates/secbird-tray/src/win.rs`: registrazione Raw Input nell'init, blocco `WM_INPUT`, `App::raw_input`, `raw_input_razer_pid`, `TIMER_SPECTRUM_WAKE`, gestione rescan e avvio.
- `crates/secbird-tray/src/wake_sync.rs`: `WakeTracker`, `last_input` HashMap, `pending_wake`, `scheduled`, soglia `WAKE_IDLE = 60 sec`.
- `crates/secbird-tray/src/engine.rs`: `resync_spectrum()` (solo zone Spectrum/2+ device), `Engine::rescan()` (clear/reopen handles), `Engine::apply()`, `event_paths()`.
- `crates/secbird-tray/src/settings.rs`: `spectrum_wake_sync`.
- `crates/secbird-tray/src/editor.html`: toggle experimental e pulsante.
- `crates/secbird-tray/src/server.rs`: API `POST /api/resync-spectrum`.

### Ipotesi tecniche da verificare, NON fatti dimostrati

- **Stato WakeTracker scorretto quando entrambi dormono.** Attualmente la condizione `both_active` significa soltanto “l'ultimo Raw Input di entrambi entro 60 secondi”, **non** “entrambi effettivamente connessi/svegli”. Se il primo si sveglia, resta attivo senza generare ulteriori input, e il secondo si sveglia **oltre 60 secondi dopo**, il tracker non considera il primo attivo anche se lo è. Potrebbe rimanere pending senza mai fare sync. Viceversa i timestamp ancora freschi possono falsare il significato di wake.
- **Raw Input registration o device identity dopo doppio standby.** I receiver possono ri-enumerare HID collections; i nomi Raw Input rappresentano ricevitori fisici o interfacce, non sempre il PID logico. `raw_devices` cache può trattenere handle non più validi; bisogna loggare handle, VID/PID identificati e nuovi eventi. Se si disconnette un HID receiver dopo sleep, gli eventi `WM_INPUT_DEVICE_CHANGE` possono richiedere `RIDEV_DEVNOTIFY` per aggiornare la cache.
- **Engine conserva o perde il secondo device quando entrambi dormono?** `Engine::rescan()` chiude i vecchi handle, rifà la discovery e può marcare un mouse wireless come “asleep/unanswered”; quando arriva il secondo wake, il dispositivo potrebbe non essere ancora presente in `Engine::devices()`. Una sync con meno di due device restituisce 0 senza fare altro.
- **Timing di SET dopo wake**: 1,5 secondi potrebbero non bastare perché entrambi rispondano ai feature report. Il risultato del SET per ciascun LED va osservato; l'attuale `resync_spectrum()` conta i successi e ignora silenziosamente i singoli errori.
- **Reapply e profiling**: rescan e cambio foreground possono riapplicare uno solo degli effetti e sfasare il sync appena fatto.
- **Parte input effettivamente osservata**: conoscere l'ordine di attivazione e i tempi reali dal log è essenziale. Nessun log di questo particolare errore è stato ancora fornito, dunque non dichiarare una causa definitiva.

### Debug raccomandato a Claude

1. **Non riscrivere il progetto da zero.** Usa il branch `experiment-spectrum-resync-on-wake`, conserva il branch RGB isolato.
2. Aggiungi logging strutturato, **senza HID GET cicliche**: inizializzazione Raw Input, PID di ogni handle, input prima/dopo idle, transizioni di stato per device, quando `pending_wake` e `both_active` cambiano, timer pianificato/eseguito, numero device presenti, esito SET di ogni zone ed eventuali errori (senza seriali interi).
3. Riproduci i casi controllati in tabella, con almeno **1 min di idle**, due ordini e tempi brevi/lunghi di attesa tra i due risvegli; valuta cosa significhi davvero “wake” per il firmware.
4. Per lo stato di “sveglio” usa preferibilmente **eventi passivi di Windows per entrambi** e, solo al momento del trigger, una verifica non invasiva di disponibilità. Nessun poll di feature report ripetuto durante inattività.
5. Se l'engine ha perso il device, pianifica un rescan **event-driven e limitato** dopo wake o device-arrival, senza loop infinito né retry aggressivi.
6. Sincronizza solo se entrambi hanno Spectrum, controllando che **entrambi i SET abbiano avuto successo**. Se uno non è pronto, non dichiarare successo; pochi retry con backoff SOLO post-wake e solo se necessario.
7. Test Rust della macchina a stati: due dormono + mouse prima, tastiera dopo; viceversa; intervalli di 10 s / 90 s; seconda periferica che riappare tardi; un solo sleep; altro device scollegato; input continuo che non crea tempesta di SET.
8. Test reale su Windows; chiedere il log `%APPDATA%\\Secretarybird\\secbird.log` solo se serve dopo aver introdotto logging utile. Confermare visivamente il risultato con l'utente prima di considerare risolto.

### Criteri di accettazione da non perdere

- Funziona se uno va in standby e si riattiva (**già osservato**; non regredire).
- Funziona se entrambi vanno in standby, mouse prima poi tastiera.
- Funziona se entrambi vanno in standby, tastiera prima poi mouse.
- Funziona se il secondo risveglio arriva dopo >60 secondi e il primo device è rimasto acceso.
- Dopo il risveglio di entrambi, riavvia entrambi gli Spectrum solo quando sono disponibili; niente comandi continui su device spenti.
- La gestione sleep del Basilisk resta corretta (1 min test).
- Nessun mutamento per effetti non-Spectrum, o per utenti con opzione OFF.
- Niente modifica al pairing dei dongle e niente cambio ai device RGB reali.
- Non integrare upstream una feature non verificata sul PC reale.

### 4.1 Seconda sessione (29/10/2026): diagnosi dal log reale e correzione

**Branch:** `experiment-spectrum-wake-double-standby`, creato da `experiment-spectrum-resync-on-wake` (`071929c`). Commit della correzione: `5171084`. Il branch RGB `fix-basilisk-v4-hyperspeed-rgb` **non è stato toccato**.

#### Evidenza: il log reale `%APPDATA%\Secretarybird\secbird.log` del PC dell'utente

Stato al momento della prova (letto da `tray.json` e `profiles.json`): `spectrum_wake_sync: true`, **`away_lights: "off"`**, Spectrum sia su tastiera (`backlight`, LED 5) sia sul mouse (`underglow`, LED 10), `sleep_minutes: 1` su **entrambi**. Estratto (build del prototipo bidirezionale, senza seriali):

```text
07:57:52Z tray started
07:57:53Z found: Basilisk V4 HyperSpeed (Wireless), BlackWidow V4 Low-Profile HyperSpeed
07:57:54Z Spectrum wake: observing raw input PID 00EA
07:57:58Z Spectrum wake: observing raw input PID 00EA      <- raw_devices svuotata dal retry rescan +4 s
07:59:18Z Spectrum wake: observing raw input PID 00EA      <- mouse risvegliato
07:59:30Z Spectrum wake: observing raw input PID 02C9      <- tastiera risvegliata (primo tasto dall'avvio)
07:59:31Z Spectrum wake: restarted 0 zones on keyboard/mouse
07:59:59Z found: BlackWidow V4 Low-Profile HyperSpeed       <- "nuova": era stata tolta dalla lista
08:02:13Z ... 08:21:52Z  Spectrum wake: restarted 2 zones (7 volte)
```

#### Causa CONFERMATA (catena ricostruita dal log + codice)

1. All'avvio `start_retries` fa rescan a +4 s, +15 s, +45 s, poi ogni 60 s per 30 min finché `has_sleeping()`. Il rescan delle ~07:58:59 è avvenuto con la tastiera **già in standby** (nessun tasto dall'avvio, timer 1 min): `probe_exact` (GET seriale, txid FF) non ha risposto e **`Engine::rescan` ha tolto la tastiera da `devices`** (finita in `unsupported` come ASLEEP). Il rescan svuotava anche la cache `raw_devices` (da qui i doppi "observing").
2. 07:59:18 mouse sveglio, 07:59:30 tastiera sveglia → `WakeTracker` ha programmato la sync → `resync_spectrum` ha trovato **un solo** dispositivo con Spectrum (`distinct.len() < 2`) → **0 zone, nessun comando**. Poi `finished()` ha **azzerato comunque `pending_wake`**: il risveglio è andato perso.
3. 07:59:59 lo "sleeper retry" ha ritrovato la tastiera come dispositivo nuovo e `App::rescan` le ha fatto **`apply()` del profilo da sola** → Spectrum della sola tastiera riavviato **fuori fase**. Nessuno l'ha più risincronizzato.

Quindi il caso "entrambi dormono" fallisce quando un rescan cade mentre un dispositivo dorme. Succede sempre nei primi ~2 minuti dopo l'avvio, poi ogni 60 s per 30 minuti dopo avvio o resume (finché c'è un dispositivo addormentato), e a ogni `WM_DEVICECHANGE` Razer. Con un solo dispositivo addormentato e l'altro in uso non c'è rescan di mezzo: per questo quel caso funzionava.

#### Altri difetti reali trovati nel codice (non ancora visti nel log)

- **`away_lights: "off"`** (impostato dall'utente): al ritorno `lights_back` riapplica i profili a chi risponde; chi dormiva entra in `relight` e `retry_lights_back` (ogni 3 s per 2 min) gli riapplica il profilo **da solo** quando risponde → stesso sfasamento.
- `both_active` significava solo "input negli ultimi 60 s": un dispositivo sveglio ma fermo da >60 s era trattato come addormentato (con timer più lunghi di 1 min).
- Se `with_app` falliva al timer (app ri-entrata), `scheduled` restava `true` per sempre: nessuna sync fino al riavvio.
- `resync_spectrum(None)` dal timer: un errore sul dispositivo appena svegliato non fermava la scrittura sugli altri; gli esiti per zona non venivano registrati.

#### Ipotesi NON confermate (da verificare con il nuovo log)

- I 7 eventi `restarted 2 zones` dopo le 08:00 hanno scritto entrambi i LED con successo; **non si sa** quali corrispondano a prove "entrambi in standby". Se l'utente ha visto sfasamento anche in uno di questi, esiste una seconda causa. Candidata principale: il firmware della **tastiera** reinizializza le luci più tardi di 1,5 s dopo il primo tasto, sovrascrivendo il riavvio. Il caso "solo tastiera in standby" **non è mai stato confermato** su hardware. Per questo esiste `spectrum_wake_settle_ms` (sotto).
- Al rescan delle 07:58:59 il mouse **non** è stato tolto dalla lista, pur con ~61 s di inattività. Due spiegazioni: il dongle Basilisk risponde alla GET seriale anche col mouse addormentato, oppure il mouse non dormiva ancora. Il nuovo log (`gone:`) lo chiarirà.
- Non è noto se una SET verso un dispositivo addormentato lo risvegli; il nuovo codice **evita** di scrivere a dispositivi presunti addormentati.

#### Nuovo progetto (`crates/secbird-tray/src/wake_sync.rs`, logica pura e testata)

- **Peer** per ogni PID: `wireless` + timer di standby dal profilo (`sleep_minutes`; 55 s se ignoto). Un device cablato non "si sveglia" mai ed è sempre sveglio.
- **Wake** = primo input Raw Input dopo un silenzio ≥ timer − 5 s (55 s con timer 1 min). **Sveglio** = input più recente di quella soglia. Nessuna interrogazione HID per saperlo.
- A ogni wake, dopo `SETTLE` (1,5 s): **Plan** = prima i device appena svegli (strict), poi gli altri con Spectrum **certamente svegli**. Se meno di due → `Wait` ("waiting for 02C9 to wake"): **nessun comando**, nemmeno al device sveglio. Quando l'altro si sveglia, il suo wake fa la sync. Funziona in entrambi gli ordini e con qualunque distanza tra i risvegli.
- **Missing**: se il device sveglio non è in `Engine::devices` (tolto da un rescan mentre dormiva) → **un rescan mirato**, poi la scrittura.
- **NotReady**: se il device svegliato non accetta la SET → stop (gli altri non vengono riavviati fuori fase) e al massimo **2 retry** (+2 s, +4 s, ciascuno preceduto da rescan); poi si attende il prossimo wake. Niente flusso continuo di comandi.
- **Profilo rimandato dalla tray da sola** (rescan che ritrova un device, `retry_lights_back`) → nuova sync con i device svegli. Non dopo `apply_all` (avvio, resume, `lights_back`): lì tutti ripartono insieme.
- `RIDEV_DEVNOTIFY` + `WM_INPUT_DEVICE_CHANGE` mantengono la cache handle→PID (prima veniva svuotata a ogni rescan). PID Razer letto genericamente da `VID_1532&PID_xxxx`, non più due PID cablati nel codice. Il timer viene ri-armato se l'app è occupata.
- Disattivando l'opzione nell'editor i wake vengono ignorati subito; per **attivarla** serve ancora riavviare la tray (registrazione Raw Input all'avvio).
- `tray.json` → `"spectrum_wake_settle_ms": 3000` (opzionale, solo a mano, 500–10000) per provare un'attesa più lunga dopo il wake **senza ricompilare**.

#### Messaggi di log nuovi (prefisso `Spectrum wake:`)

| Riga | Significato |
| --- | --- |
| `raw input device 0x… is Razer 02C9 MI_02 COL01` | Collezione HID del ricevitore vista da Raw Input (all'avvio e a ogni arrivo). |
| `raw input device 0x… (Razer 00EA) removed` | Windows ha rimosso la collezione: utile per capire se lo standby ri-enumera il dongle. |
| `00EA woke (after 312 s quiet); 00EA quiet 0 s, 02C9 quiet 300 s, may be asleep` | Wake rilevato e stato presunto degli altri. |
| `attempt 1: nothing written, waiting for 02C9 to wake` | Corretto quando l'altro dorme. |
| `02C9 isn't in the device list (dropped while asleep?): rescanning` | Il caso del bug confermato, ora gestito. |
| `attempt 1 on 02C9, 00EA: 02C9 backlight ok (31 ms), 00EA underglow ok (28 ms)` | Scrittura riuscita, nell'ordine e con i tempi. |
| `… failed: … ` / `trying again in 2 s` / `giving up until the next wake` | Device non pronto, retry limitati. |
| `00EA got its profile again; syncing` | La tray ha rimandato un profilo da sola; segue sync. |
| `gone: BlackWidow … (asleep or unplugged)` (senza prefisso) | Un rescan ha tolto un device dalla lista. |
| `Spectrum re-sync now: …` | Esito del pulsante manuale. |

#### Test automatici aggiunti (19, in `wake_sync.rs`)

Entrambi in standby → mouse poi tastiera e tastiera poi mouse, a **10 s** e **90 s** (primo device in uso); entrambi gli ordini con il primo device **tornato in standby** prima del secondo wake; un solo device in standby (mouse; tastiera) come regressione del caso funzionante; **tastiera tolta dalla lista mentre dormiva** (il caso del log); 2 retry poi attesa; nuovo wake che azzera i retry; input continuo per 10 min senza nuove scritture; profilo rimandato; device cablato; timer di standby dal profilo; PID sconosciuti; device senza Spectrum; riga di log; parsing dei percorsi Raw Input.

#### Limiti noti

- Ricevitore Multi-Device condiviso (mouse su 02C9): Raw Input attribuisce anche l'input del mouse al PID 02C9. Non gestito: configurazione attualmente non usata.
- Gli `apply` fatti dall'editor o dal cambio profilo per programma in primo piano non attivano la sync (c'è "Re-sync now").
- Due unità dello stesso modello (stesso PID) non sono distinte.
- Il primo input dopo l'avvio della tray conta come wake: una sync in più all'avvio.
- Sul PC c'è anche `%APPDATA%\Secretarybird\devices.json` (18/10, solo 02C9/02CC) che si fonde col database integrato: non modificato.
- All'avvio di Windows parte la versione **installata** 0.5.0 (autostart): nel log delle 07:43 dà `unknown lighting zone 'underglow'` (database vecchio). Per i test va chiusa prima di avviare la build di prova.

## 5. Branch e commit sul fork (controllati il 29/10/2026)

| Branch | Commit noto prima di questo handoff | Uso |
| --- | --- | --- |
| `main` | `9f6d0cac305c093ae2f582614e8fed1d48205481` | Default fork. Non incorpora automaticamente tutti i branch. |
| `fix-basilisk-v4-hyperspeed-rgb` | `8819d13a6444b8384c730ee248f49a91bbc30235` | Patch RGB **testata**, isolata, pronta per upstream PR. |
| `test-02c9-shared-hyperspeed-receiver` | `f4658023a9c2b4c6266b2c5d03b5f7062e94dc2c` | Test/build Windows; correzioni RGB, supporto receiver. |
| `support-02c9-shared-hyperspeed-receiver` | `7c061b1d6f44d251c2ee1ddd0415e2ca790d864c` | Discovery Multi-Device su un dongle. |
| `tool-unpair-00ea-from-02c9` | `9192ae127038a873d6a89e48a8b581904157253a` | Utility **usata con successo**; storica, non ri-eseguire. |
| `experiment-spectrum-resync-on-wake` | `09b0cc35b55db2e8213fb7536381fc0616d22f63` | **Prototipo con sync parzialmente funzionante**: uno dorme funziona; entrambi dormono fallisce. |
| `read-only-hyperspeed-multidevice-diagnostics` | `8844c3315b3b1ed6021e40d6ffab80f95bc10c45` | Sola lettura/diagnostica. |
| `test-hyperspeed-multidevice-diagnostics` | `78ce18ff0643c400531f0c0b114a82159065a28c` | Diagnostica iniziale. |
| `test-02c9-readonly-tray` | `2127df4285c4b005e766ed5883d9fec8b76f6b6d` | Esperimento incompleto/obsoleto; evitare come base. |
| `add-blackwidow-v4-low-profile-wired-02cc` | `b019fc9da2e713aec0204f25b0636ecb3df09917` | PR #31 già merged. |
| `fix-dpi-on-non-mouse-devices` | `2e4bc68955631c839c152179a87c9baccc8aea66` | PR #32 già merged. |
| `experiment-spectrum-wake-double-standby` | `5171084` (+ commit di questo documento) | **Correzione doppio standby (§4.1)**, basata su `experiment-spectrum-resync-on-wake`. Sperimentale, da verificare su hardware. |

**Tutti questi branch esistono già nel fork remoto**. Non occorre accorparli in `main` per “salvarli”: farlo mescolerebbe codice sperimentale a patch validate. Il presente `CLAUDE.md` è copiato su `main` e su `experiment-spectrum-resync-on-wake`. Dopo il salvataggio, gli HEAD di questi due branch saranno nuovi rispetto alla tabella; ricontrollarli con `git fetch --all`.

## 6. CI, test e artefatti

- Shared receiver CI: https://github.com/lukethehawk/secbird/actions/runs/40741074248
- Utility unpair: https://github.com/lukethehawk/secbird/actions/runs/40756754825 — artifact `guarded-02c9-unpair-00ea`.
- RGB patch test: https://github.com/lukethehawk/secbird/actions/runs/40760671241 — successful.
- Primo Spectrum wake prototype (solo mouse): https://github.com/lukethehawk/secbird/actions/runs/40767796461 — successful.
- Prototipo bidirezionale più recente: https://github.com/lukethehawk/secbird/actions/runs/40771276269 — **Clippy, Rust tests, Windows release build SUCCESSFUL**, artifact `secbird-wake-spectrum-windows-test`, **ma bug double-standby in hardware reale**.
- **Correzione doppio standby** (`experiment-spectrum-wake-double-standby`, commit `5171084`): https://github.com/lukethehawk/secbird/actions/runs/40844505774 — **Clippy `-D warnings`, test Rust (core 103 ok/5 ignored; tray 20 ok, di cui 19 nuovi in `wake_sync`), build release: SUCCESSFUL** (29/10/2026). Artifact `secbird-experiment-spectrum-wake-double-standby-windows` (scade il 12/11/2026). Il messaggio di commit dice "21 tests": il numero corretto è 19. **Verifica hardware ancora da fare.**
- GitHub Actions artifacts possono scadere; si possono ricompilare i branch.
- Installazione build: uscire **completamente** dall'istanza tray precedente. Single-instance mutex `Local\\Secretarybird.Tray`: se già attiva, lanciare altro EXE potrebbe aprire l'editor della **vecchia istanza** e falsare il test.
- PowerShell nella cartella dei binari: `.\\secbird-tray.exe --editor`. Non usare solo il nome senza `.\\`. Log: `%APPDATA%\\Secretarybird\\secbird.log`; profili/settings: `profiles.json`, `tray.json` nella cartella Secretarybird di AppData.
- Sorgenti base: `crates/secbird-core/src/commands.rs`, `device.rs`, `features.rs`, `effects.rs`, `profile.rs`, `actions.rs`; tray `engine.rs`, `win.rs`, `server.rs`, `settings.rs`, `editor.html`, `wake_sync.rs`.

## 7. Come proseguire: ordine di lavoro

1. **Priorità immediata:** sistemare il bug Spectrum con **entrambi i dispositivi addormentati**, preservando l'esito funzionante con un solo device dormiente. Iniziare da una diagnosi con logging e una macchina a stati corretta, non da sleep timer o ripair.
   - **Stato al 29/10 (seconda sessione):** correzione in `experiment-spectrum-wake-double-standby` (§4.1), in attesa del test hardware dell'utente. Prossimo passo: far provare i 5 scenari, farsi dare **`secbird.log`** e confrontare le righe `Spectrum wake:` con quello che l'utente ha visto. Se resta sfasamento con `… ok` su entrambi i LED, provare `spectrum_wake_settle_ms` 3000 (ipotesi firmware tastiera).
2. Tenere separato il branch `experiment-spectrum-resync-on-wake` dalla PR RGB. Il codice non ancora verificato non deve finire nella patch upstream.
3. Gestire la PR RGB isolata appena opportuno con il pulsante GitHub “Create pull request” al link indicato. È indipendente dalla sincronizzazione.
4. Dopo Spectrum verificare ancora button remapping e profili DPI, stabilità wireless duratura, eventuale allineamento RGB al risveglio di tastiera sola, e comportamenti in caso di lock/wake del PC.
5. Documentare ogni nuova prova utente con data, firmware, condizioni e risultato. Non trattare screenshot UI o CI come dimostrazione del risultato fisico.

## 8. Prompt per Claude Code

> Leggi integralmente il CLAUDE.md e ispeziona il branch `experiment-spectrum-resync-on-wake`. Il problema prioritario è il seguente: **se solo mouse o tastiera vanno in standby e si riattivano, Spectrum si risincronizza; se entrambi vanno in standby e vengono poi risvegliati in sequenza, non si risincronizza**. Deve funzionare in entrambi gli ordini, anche con lunga distanza temporale tra risvegli, senza disattivare lo sleep e senza HID polling costante. Prima analizza criticamente Raw Input, state machine, re-enumeration, rescan, disponibilità Device e resync_spectrum. Aggiungi log diagnostici e test dei due dispositivi dormienti, individua la causa anziché ipotizzarla come certa; proponi e implementa la correzione in un branch sperimentale, mantenendo la PR RGB isolata. Non dichiarare riuscita senza test hardware dell'utente e non riconnettere mouse e tastiera allo stesso dongle.
