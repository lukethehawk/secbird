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
| PR di correzione RGB verso upstream | Branch pronto, **PR non ancora aperta** al controllo del 29/10; creazione automatica via integrazione GitHub fallita con HTTP 403. **Aperta il 29/10/2026 con `gh`: https://github.com/infinityInfinite/secbird/pull/69.** |
| PR upstream aperte il 29/10/2026 (su ok esplicito dell'utente) | Retry "Lights when away": [#68](https://github.com/infinityInfinite/secbird/pull/68). RGB Basilisk: [#69](https://github.com/infinityInfinite/secbird/pull/69). Sync Spectrum al risveglio: [#70](https://github.com/infinityInfinite/secbird/pull/70), che contiene anche il commit di #68. **Non ancora accettate.** |
| Correzione doppio standby, branch `experiment-spectrum-wake-double-standby` | Causa principale **confermata dal log reale** (§4.1). Nuova macchina a stati + rescan mirato + retry limitati + log diagnostici. CI Windows: vedi §6. **NON ancora verificata su hardware: non dichiararla risolta.** |
| ↳ Prima prova hardware della correzione (29/10/2026, 08:52–09:02 UTC) | **POSITIVA.** Utente: «mi pare si syncano sempre in ogni caso ora». Il log conferma la sync su entrambi i LED nei casi "entrambi in standby → tastiera poi mouse", "→ mouse poi tastiera" e nel caso esatto del bug (tastiera tolta dalla lista mentre dormiva). Dettagli e scenari non ancora coperti in §4.1. La riga "NON FUNZIONA" sopra resta valida per il **vecchio** prototipo. |
| ↳ Terza prova, build `3b4c522` (29/10/2026, 10:22–10:33 UTC) | **POSITIVA.** Utente: «mi pare sia ok». Il retry dello spegnimento ha recuperato 2 volte il mouse. Il percorso riaccensione → sync è stato provato e ha scritto `ok` su entrambi. Resta non spiegata una tastiera che non risponde allo "screen off". Ancora da provare: sospensione del PC e uso prolungato (§4.1). |

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
- **Aperta il 29/10/2026: https://github.com/infinityInfinite/secbird/pull/69** (con `gh`, dall'account dell'utente). Prima di allora: PR upstream **non ancora aperta al 29/10/2026**: il connettore GitHub non ha permesso POST cross-repo (HTTP 403 Resource not accessible by integration).
  Aprire manualmente su https://github.com/infinityInfinite/secbird/compare/main...lukethehawk:fix-basilisk-v4-hyperspeed-rgb?expand=1
- Build/test patch RGB: https://github.com/lukethehawk/secbird/actions/runs/40760671241, **success**. *(Correzione 29/10: quella run è sul branch `test-02c9-shared-hyperspeed-receiver` (`f465802`), non sul branch isolato `8819d13`. Il branch isolato è passato in CI per la prima volta con la run 40859801048, vedi §6.)*

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

#### Prima prova hardware della correzione (29/10/2026)

Build della run 40844505774, avviata da cartella nuova; log vecchio rinominato in `secbird-prima.log`. Impostazioni invariate: Spectrum su entrambi, standby 1 min su entrambi, `away_lights: "off"`, nessun `spectrum_wake_settle_ms` (quindi 1,5 s). Feedback utente: **«mi pare si syncano sempre in ogni caso ora»**. Log (UTC, ora locale = UTC+1):

```text
08:52:40 tray started; Raw Input: 4 collezioni per ricevitore (00EA e 02C9: MI_00, MI_01 COL01, MI_01 COL02, MI_02)
08:52:42 attempt 1: nothing written, waiting for 02C9 to wake          <- primo input del mouse all'avvio
08:53:46 gone: BlackWidow V4 Low-Profile HyperSpeed (asleep or unplugged)   <- il bug originale si ripresenta (retry +45 s)
08:53:54 attempt 1: nothing written, no other device shows Spectrum    <- mouse sveglio, tastiera fuori lista: niente comandi
08:55:00 02C9 woke (first input seen); 00EA quiet 1 s
08:55:01 02C9 isn't in the device list (dropped while asleep?): rescanning
08:55:02 found: BlackWidow ...; attempt 1 on 02C9, 00EA: 02C9 backlight ok (23 ms), 00EA underglow ok (18 ms)   <- CASO DEL BUG: RISOLTO
08:57:52 02C9 woke (after 171 s quiet); 00EA quiet 160 s, may be asleep -> waiting for 00EA
08:58:10 00EA woke (after 179 s quiet) -> attempt 1 on 00EA, 02C9: ok, ok     <- entrambi in standby, TASTIERA poi MOUSE (18 s)
08:59:10 02C9 woke; locked: lights to 0% -> nothing written, lights are down while away   <- Win+L
08:59:18 back: lights as saved                                         <- lights_back: profili a entrambi insieme
09:00:25 00EA woke (after 92 s quiet); 02C9 quiet 66 s, may be asleep -> waiting for 02C9
09:01:36 02C9 woke (after 137 s quiet); 00EA quiet 0 s -> attempt 1 on 02C9, 00EA: ok, ok   <- entrambi in standby, MOUSE poi TASTIERA (71 s, mouse in uso)
```

**Fatti nuovi emersi dal log:**
- Il rescan di avvio a +45 s ha di nuovo tolto la tastiera addormentata: il meccanismo della causa è **riprodotto** e ora viene **gestito** (rescan mirato al risveglio, poi scrittura su entrambi).
- Nessuna riga `raw input device … removed` durante gli standby: con dongle separati lo standby **non ri-enumera** le collezioni HID, quindi gli handle Raw Input restano stabili.
- Nessun `trying again`: entrambi accettano la SET 1,5 s dopo il risveglio (18–39 ms per scrittura). L'ipotesi "firmware tastiera lento" non è emersa in queste prove; `spectrum_wake_settle_ms` non è servito.
- Con Win+L breve (8 s) la sync viene giustamente saltata ("away"); al ritorno `lights_back` riavvia entrambi insieme.
- Al rescan delle 08:53:46 il mouse era fermo da ~53 s (non ancora in standby): l'ipotesi sul dongle Basilisk che risponde mentre il mouse dorme **resta aperta**.

**Non ancora coperto esplicitamente dal log** (il feedback dice "in ogni caso", ma restano da verificare in modo mirato):
- Il primo dispositivo che torna in standby prima che si svegli il secondo (scenario 5).
- La regressione con un solo dispositivo in standby e l'altro in uso: mouse da solo, tastiera da sola senza Win+L. *(Rilettura del log, 29/10: la tastiera da sola, col mouse in uso, è coperta alle 09:03:39 e 09:06:02 UTC, con `ok` su entrambi. Resta da provare il mouse da solo.)*
- Blocco lungo o schermo spento con `away_lights: "off"` mentre entrambi dormono: è il percorso `retry_lights_back` → sync. *(Percorso coperto nella terza prova, 10:32 UTC.)*
- Sospensione e ripresa del PC; uso reale per ore.

#### Seconda prova: schermo spento da Windows e Win+L (29/10/2026, 09:10–09:13 UTC)

Condizioni: timeout schermo di Windows a 1 minuto; il pulsante del monitor **non** va bene, perché Windows non lo vede. **Alle 09:09 UTC l'utente ha cambiato dall'editor i timer di standby: mouse 6 min, tastiera 7 min** (prima 1 min entrambi). Feedback: «2 volte in standby lo schermo, la prima volta ok, la seconda il mouse non ha spento il led. con win+l tutto ok, i led si accendono dopo aver inserito la password».

```text
09:10:47 screen off: lights to 0%
09:10:59 00EA woke (after 72 s quiet); 02C9 quiet 99 s, may be asleep   <- timer ancora da 1 min: peer non aggiornati
09:10:59 back: lights as saved
09:11:01 attempt 1: nothing written, waiting for 02C9 to wake
09:12:05 screen off: lights to 0%          <- utente: qui il LED del mouse NON si è spento
09:13:08 back: lights as saved
09:13:16 locked: lights to 0%              <- Win+L: ok
09:13:25 back: lights as saved             <- LED riaccesi dopo la password: ok
```

Analisi:
- **La sync Spectrum si è comportata correttamente**: dopo le 09:11 nessun wake, perché con timer da 6–7 minuti pause di 2–4 minuti non sono standby; nessun comando inutile. Nessuna scrittura mentre si era "away".
- **Il percorso `retry_lights_back` → sync resta NON provato.** Con timer lunghi i dispositivi non dormivano, quindi entrambi hanno risposto a `lights_back` insieme. Per provarlo serve uno schermo spento più lungo del timer di standby.
- **LED del mouse acceso al secondo "screen off": problema della funzione "Lights when away" originale (#48/#52), NON della sync.** `Engine::lights_away` è identico a upstream e scarta gli errori: GET brightness poi SET live, nessun log per dispositivo, **nessun retry** (solo `lights_back` ce l'ha, `relight`). Se il mouse perde quel comando resta acceso per tutto il tempo "away". La causa del mancato comando non si vede dal log. Ipotesi: dongle occupato o mouse in transizione. Proposta: loggare l'esito per dispositivo di away/back ed eventualmente aggiungere per "away" un retry come quello di "back".
- **Peer non aggiornati (difetto minore della correzione):** i timer di standby vengono riletti solo dopo un rescan o un tentativo di sync, non quando l'editor salva. Alle 09:10:59 la tastiera è stata giudicata "may be asleep" con il vecchio timer da 1 min. Effetto innocuo in questo caso (nessuna scrittura; `lights_back` ha riallineato), ma il planner potrebbe saltare un dispositivo sveglio. Correzione semplice: rileggere i peer **all'inizio** di `spectrum_due` invece che alla fine.

#### Modifica successiva: retry per le luci away, log per dispositivo, timer freschi (commit `3b4c522`)

Richiesta esplicita dell'utente dopo la seconda prova.
- `Engine::lights_away` restituisce i device che hanno perso il comando. `retry_lights_away` li riprova sul battito esistente di 3 s (`TIMER_LIGHTS`) per **30 s** (`DIM_FOR`), solo mentre si è "away". È simmetrico a `retry_lights_back` (2 min, `RELIGHT_FOR`), che esisteva già. Costo massimo: ~10 GET+SET verso un device che non risponde, poi basta.
- Log: `screen off: lights to 0%; not answering: <nome>, trying again for 30 s`; `back: lights as saved; not answering: …, trying again for 120 s`; `lights away|back: <nome> answered on a retry` / `gave up on <nome> (not answering)`. Nessun seriale.
- I retry non contano più come "fatto" un device assente dalla lista (prima veniva tolto da `relight` senza scrivere nulla).
- `spectrum_due` rilegge i timer di standby (peer) **all'inizio** del tentativo: un timer cambiato dall'editor vale subito.
- Il messaggio "no other device shows Spectrum" ora aggiunge i peer fuori lista: `…; 02C9 not in the device list (asleep?)`.
- Nuovi test: timer cambiati dall'editor (caso del 29/10), nuovo messaggio, scadenza dei retry (`engine.rs`).

**Messaggio da migliorare (cosmetico, risolto in `3b4c522`):** con la tastiera fuori lista il log dice `no other device shows Spectrum`. Più preciso sarebbe "the other device is not in the list (asleep?)": il planner vede solo i dispositivi presenti.

#### Terza prova: build `3b4c522`, schermo spento (29/10/2026, 10:22–10:33 UTC = 11:22–11:33 locali)

Build della run 40848946895, tray avviata alle 10:22:10. `tray.json`: `away_lights: "off"`, `spectrum_wake_sync: true`, nessun `spectrum_wake_settle_ms`. `profiles.json` al momento della lettura: standby **6 min su entrambi** (mouse e tastiera). Timeout schermo di Windows: 1 min. Feedback utente: **«mi pare sia ok»**.

```text
10:22:11 00EA woke (first input seen); 02C9 no input yet          <- limite noto: primo input dopo l'avvio
10:22:12 attempt 1: nothing written, waiting for 02C9 to wake
10:25:46 screen off: lights to 0%; not answering: Basilisk V4 HyperSpeed (Wireless), trying again for 30 s
10:25:47 lights away: Basilisk V4 HyperSpeed (Wireless) answered on a retry
10:26:07 back: lights as saved
10:27:13 screen off: lights to 0%; not answering: Basilisk V4 HyperSpeed (Wireless), trying again for 30 s
10:27:14 lights away: Basilisk V4 HyperSpeed (Wireless) answered on a retry
10:27:42 02C9 woke (first input seen); 00EA quiet 90 s
10:27:42 back: lights as saved
10:27:44 attempt 1 on 02C9, 00EA: 02C9 backlight ok (38 ms), 00EA underglow ok (19 ms)
10:32:04 screen off: lights to 0%; not answering: BlackWidow V4 Low-Profile HyperSpeed, trying again for 30 s
10:32:36 lights away: gave up on BlackWidow V4 Low-Profile HyperSpeed (not answering)
10:32:49 back: lights as saved; not answering: Basilisk V4 HyperSpeed (Wireless), trying again for 120 s
10:32:57 lights back: Basilisk V4 HyperSpeed (Wireless) answered on a retry
10:32:57 00EA got its profile again; syncing
10:32:58 attempt 1 on 00EA, 02C9: 00EA underglow ok (18 ms), 02C9 backlight ok (23 ms)
```

Analisi:
- **Retry "away" CONFERMATO su hardware.** Il mouse ha perso il primo comando di spegnimento **2 volte su 3** e lo ha preso al retry ~1 s dopo. È lo stesso tipo di errore che alle 09:12:05 aveva lasciato il LED acceso; ora viene recuperato. Il mancato comando allo "screen off" quindi non è raro. La causa resta ignota (ipotesi: dongle occupato o radio del mouse in risparmio).
- **Percorso `retry_lights_back` → sync CONFERMATO su hardware** (era il punto non provato della seconda prova). Al ritorno delle 10:32:49 il mouse non ha risposto per ~8 s, il retry lo ha riacceso e la sync è partita subito dopo con `ok` su entrambi.
- **Tastiera senza risposta per 30 s allo "screen off" delle 10:32:04: NON spiegato.** Con il timeout schermo di 1 min l'ultimo input era al più alle 10:31:04. La tastiera aveva dato input alle 10:27:42, e con un timer di 6 min non doveva ancora dormire. Ipotesi non confermate: il firmware della tastiera entra in un risparmio energetico prima del timer, oppure il dongle era occupato. L'effetto visibile non è noto: da chiedere all'utente se i LED della tastiera sono rimasti accesi verso le 11:32 locali. Al ritorno (10:32:49) la tastiera ha preso subito il profilo, quindi la sync non ne ha risentito.
- Nessun `gone:`, nessun rescan e nessun `trying again` della sync Spectrum in questa sessione.

**Ancora da coprire:** sospensione e ripresa del PC; uso reale per ore; scenario 5 (il primo device torna in standby prima che si svegli il secondo); regressione con un solo device in standby senza Win+L; la tastiera che non risponde allo "screen off".

#### Deriva con i dispositivi svegli: riavvio a fine giro (commit `1910dfb`, solo branch sperimentale)

Osservazione dell'utente (29/10, dopo la terza prova): «dopo un po' i dispositivi perdono il sync, come se gli effetti differissero di qualche ms e alla lunga perdano il ritmo».

- **Causa (ragionamento, coerente con il codice):** lo Spectrum gira nel firmware di ogni dispositivo, con il suo orologio. Secretarybird manda solo la SET 0F:02, e lo Spectrum non ha un parametro di velocità (`Effect::Spectrum.params() == (3, 0, 0, [])`). Due orologi non coincidono mai, quindi lo sfasamento cresce mentre i dispositivi restano svegli. La sync al risveglio li riallinea solo a risveglio, ritorno da away o "Re-sync now". Lo sfasamento iniziale fra le due scritture (20–40 ms) è costante e non spiega la crescita.
- **NON noti:** la velocità della deriva; la durata esatta di un giro; se i due dispositivi hanno lo stesso periodo nominale (la crescita lenta lo fa supporre); se una SET conta come attività per il timer di standby del dispositivo.
- **Scartati:**
  - Effetto calcolato dal PC: traffico continuo, tiene svegli i wireless, contro i vincoli.
  - Riavvio periodico semplice: salto di colore visibile.
- **Soluzione:** riavvio periodico allineato al giro, `wake_sync::Cycle::realign`.
  - Dall'ultimo riavvio comune (`Engine::spectrum_since`) la tray conta i giri e riavvia dopo `every` minuti, esattamente su un multiplo del periodo, con tolleranza di ±250 ms. In quel momento il colore iniziale è quello già mostrato: il salto è solo la deriva.
  - L'errore sul periodo misurato non si accumula oltre un intervallo, perché ogni riavvio azzera il conteggio.
- **Quando scrive:**
  - Solo a dispositivi in uso, cioè con input Raw Input negli ultimi 60 s e comunque entro il loro timer di standby (`WakeSync::in_use`).
  - Se uno non è in uso: riga `due, waiting for input from …`; il suo prossimo input riarma il timer. Nessun polling e nessun comando nel frattempo.
  - Niente mentre si è away, e niente con meno di due dispositivi su Spectrum.
  - Una scrittura fallita non viene ritentata: aspetta la prossima sync.
- **Da cosa conta:** `spectrum_since` si aggiorna a ogni sync scritta con successo su ≥2 dispositivi (wake, Re-sync now, battito) e all'invio dei profili a tutti (avvio, resume, ritorno da away). Il battito luci da 3 s, già esistente, vede il cambio e riarma `TIMER_SPECTRUM_CYCLE` (9); questo battito non manda nulla ai dispositivi.
- **Impostazioni `tray.json`, solo a mano:**
  - `"spectrum_cycle_ms"`: durata misurata di un giro, 1000–120000. Senza questo valore la funzione è spenta e nulla cambia.
  - `"spectrum_resync_minutes"`: intervallo, 1–240, default 10.
  - Serve `spectrum_wake_sync: true`. Va riavviata la tray dopo aver cambiato i valori, oppure si aspetta il prossimo riavvio comune.
- **Log:** `Spectrum cycle: next restart in 605 s, on a cycle boundary`; `Spectrum cycle: restart on a cycle boundary, 10 min after the last: 02C9 backlight ok (…), 00EA underglow ok (…)`; `Spectrum cycle: due, waiting for input from 00EA; …`.
- **Test (4 nuovi):** primo confine dopo l'intervallo; timer in anticipo o in ritardo di qualche ms; un'ora dopo; solo i dispositivi in uso (timer di standby da 1 min incluso). CI: vedi §6.
- **Misure richieste all'utente:**
  1. Durata del giro: dopo "Re-sync now", cronometrare 20 o più giri completi (per esempio da rosso a rosso), su tastiera e su mouse.
  2. Dopo quanti minuti lo sfasamento diventa visibile: serve a scegliere `spectrum_resync_minutes`.
- **Non provato su hardware.** Non è nella PR #70.

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
| `experiment-spectrum-wake-double-standby` | `5171084`, poi `3b4c522` (+ commit di questo documento) | **Correzione doppio standby (§4.1)**, basata su `experiment-spectrum-resync-on-wake`. Sperimentale. Tre prove hardware positive il 29/10; non ancora provati la sospensione del PC e l'uso prolungato. Non è in `main`. |
| `lights-away-retry` | `3bd594c` | **Per PR upstream.** Base: upstream `main` `5c6c379`. Solo il retry "Lights when away" + log per dispositivo (`engine.rs`, `win.rs`). |
| `spectrum-wake-sync` | `2701d93` | **Per PR upstream.** Base: `lights-away-retry`. Solo la sync Spectrum: niente ricevitore condiviso, niente RGB, niente `CLAUDE.md`/workflow del fork, niente `spectrum_wake_settle_ms`. Per il resto è identico al codice provato su hardware. |
| `ci/lights-away-retry`, `ci/spectrum-wake-sync`, `ci/fix-basilisk-v4-hyperspeed-rgb` | +1 commit `[ci-only]` | Copia del branch + `ci-branch-check.yml` (la `ci.yml` di upstream su push `ci/**`). Servono solo per la CI sul fork: **non usarli come base delle PR**. **Cancellati dal fork il 29/10** su ok dell'utente; restano in locale e le run CI restano consultabili. |

**Tutti questi branch esistono già nel fork remoto**. Non occorre accorparli in `main` per “salvarli”: farlo mescolerebbe codice sperimentale a patch validate. Il presente `CLAUDE.md` è copiato su `main` e su `experiment-spectrum-resync-on-wake`. Dopo il salvataggio, gli HEAD di questi due branch saranno nuovi rispetto alla tabella; ricontrollarli con `git fetch --all`.

## 6. CI, test e artefatti

- Shared receiver CI: https://github.com/lukethehawk/secbird/actions/runs/40741074248
- Utility unpair: https://github.com/lukethehawk/secbird/actions/runs/40756754825 — artifact `guarded-02c9-unpair-00ea`.
- RGB patch test: https://github.com/lukethehawk/secbird/actions/runs/40760671241 — successful (sul branch `test-02c9-shared-hyperspeed-receiver`, non sul branch isolato).
- **Branch per le PR upstream** (29/10/2026, workflow `ci.yml` di upstream: Windows clippy+test, Linux core+CLI):
  - `lights-away-retry`: https://github.com/lukethehawk/secbird/actions/runs/40859512653 — **success** (tray 2 test).
  - `spectrum-wake-sync`: https://github.com/lukethehawk/secbird/actions/runs/40859512622 — **success** (core 100 ok/5 ignored; tray 22).
  - `fix-basilisk-v4-hyperspeed-rgb` (`8819d13`): https://github.com/lukethehawk/secbird/actions/runs/40859801048 — **success** (Windows: core 102 ok/5 ignored, inclusi i 2 test nuovi).
- Primo Spectrum wake prototype (solo mouse): https://github.com/lukethehawk/secbird/actions/runs/40767796461 — successful.
- Prototipo bidirezionale più recente: https://github.com/lukethehawk/secbird/actions/runs/40771276269 — **Clippy, Rust tests, Windows release build SUCCESSFUL**, artifact `secbird-wake-spectrum-windows-test`, **ma bug double-standby in hardware reale**.
- **Correzione doppio standby** (`experiment-spectrum-wake-double-standby`, commit `5171084`): https://github.com/lukethehawk/secbird/actions/runs/40844505774 — **Clippy `-D warnings`, test Rust (core 103 ok/5 ignored; tray 20 ok, di cui 19 nuovi in `wake_sync`), build release: SUCCESSFUL** (29/10/2026). Artifact `secbird-experiment-spectrum-wake-double-standby-windows` (scade il 12/11/2026). Il messaggio di commit dice "21 tests": il numero corretto è 19. **Verifica hardware ancora da fare.**
- **Retry luci away + timer freschi** (commit `3b4c522`): https://github.com/lukethehawk/secbird/actions/runs/40848946895 — Clippy, test (core 103 ok/5 ignored; tray **22 ok**), build release: **SUCCESSFUL**. Artifact `secbird-experiment-spectrum-wake-double-standby-windows` (scade il 12/11/2026). Provata su hardware il 29/10: **positiva** (terza prova, §4.1).
- **Riavvio a fine giro** (commit `1910dfb`, branch sperimentale): https://github.com/lukethehawk/secbird/actions/runs/40864904890 — Clippy, test (core 103 ok/5 ignored; tray **26 ok**), build release: **SUCCESSFUL**. Artifact `secbird-experiment-spectrum-wake-double-standby-windows` (scade il 12/11/2026). Da provare su hardware, dopo la misura del giro.
- GitHub Actions artifacts possono scadere; si possono ricompilare i branch.
- Installazione build: uscire **completamente** dall'istanza tray precedente. Single-instance mutex `Local\\Secretarybird.Tray`: se già attiva, lanciare altro EXE potrebbe aprire l'editor della **vecchia istanza** e falsare il test.
- PowerShell nella cartella dei binari: `.\\secbird-tray.exe --editor`. Non usare solo il nome senza `.\\`. Log: `%APPDATA%\\Secretarybird\\secbird.log`; profili/settings: `profiles.json`, `tray.json` nella cartella Secretarybird di AppData.
- Sorgenti base: `crates/secbird-core/src/commands.rs`, `device.rs`, `features.rs`, `effects.rs`, `profile.rs`, `actions.rs`; tray `engine.rs`, `win.rs`, `server.rs`, `settings.rs`, `editor.html`, `wake_sync.rs`.

## 7. Come proseguire: ordine di lavoro

1. **Priorità immediata:** sistemare il bug Spectrum con **entrambi i dispositivi addormentati**, preservando l'esito funzionante con un solo device dormiente. Iniziare da una diagnosi con logging e una macchina a stati corretta, non da sleep timer o ripair.
   - **Stato al 29/10 (seconda sessione):** correzione in `experiment-spectrum-wake-double-standby` (§4.1), in attesa del test hardware dell'utente. Prossimo passo: far provare i 5 scenari, farsi dare **`secbird.log`** e confrontare le righe `Spectrum wake:` con quello che l'utente ha visto. Se resta sfasamento con `… ok` su entrambi i LED, provare `spectrum_wake_settle_ms` 3000 (ipotesi firmware tastiera).
   - **Aggiornamento 29/10, dopo la prima prova:** esito **positivo** su hardware (§4.1, "Prima prova hardware"). Restano da coprire: primo device tornato in standby, regressione con un solo device in standby, blocco lungo/schermo spento con `away_lights: "off"`, sospensione del PC e uso prolungato. Solo dopo valutare una PR upstream; andrà ricostruita su upstream `main` **senza** il codice del ricevitore condiviso e separata dalla PR RGB.
   - **Aggiornamento 29/10, dopo la terza prova (build `3b4c522`):** positiva. Il retry dello spegnimento funziona e il percorso riaccensione → sync è stato provato. Restano: sospensione e ripresa del PC, uso prolungato, scenario 5, un solo device in standby senza Win+L, e la tastiera che allo "screen off" delle 10:32 UTC non ha risposto per 30 s (causa ignota). Nessun merge su `main` senza la richiesta esplicita dell'utente.
   - **29/10, PR upstream preparate su richiesta dell'utente** («possiamo proporre una PR sul repo originale con le nostre modifiche»). Sono tre PR separate verso `infinityInfinite/secbird`:
     1. `lights-away-retry`;
     2. `spectrum-wake-sync`, che contiene anche il commit della 1 e si riduce a un commit dopo il suo merge;
     3. `fix-basilisk-v4-hyperspeed-rgb`. Il template PR di upstream chiede un device report per le modifiche a `devices.json`.
     Il codice del ricevitore condiviso resta fuori. Le bozze delle descrizioni sono in `pr-drafts/` nella cartella locale (esclusa da git con `.git/info/exclude`). Le PR si aprono solo dopo l'ok esplicito dell'utente.
   - **Aperte il 29/10/2026** dopo l'ok dell'utente («Apri tutte e tre»): [#68](https://github.com/infinityInfinite/secbird/pull/68) `lights-away-retry`, [#69](https://github.com/infinityInfinite/secbird/pull/69) `fix-basilisk-v4-hyperspeed-rgb`, [#70](https://github.com/infinityInfinite/secbird/pull/70) `spectrum-wake-sync`. Le correzioni chieste dal maintainer vanno fatte su quei branch, non su `experiment-spectrum-wake-double-standby`. Se cambia il codice, riportare la modifica anche sul branch sperimentale.
   - **29/10, deriva con dispositivi svegli:** riavvio a fine giro sul branch sperimentale (`1910dfb`, §4.1). Prossimo passo: l'utente misura la durata del giro e il tempo dopo cui lo sfasamento si vede; poi si imposta `spectrum_cycle_ms` in `tray.json` (a tray chiusa) e si prova. Solo se funziona va proposto upstream, come PR separata o come aggiunta a #70.
2. Tenere separato il branch `experiment-spectrum-resync-on-wake` dalla PR RGB. Il codice non ancora verificato non deve finire nella patch upstream.
3. Gestire la PR RGB isolata appena opportuno con il pulsante GitHub “Create pull request” al link indicato. È indipendente dalla sincronizzazione.
4. Dopo Spectrum verificare ancora button remapping e profili DPI, stabilità wireless duratura, eventuale allineamento RGB al risveglio di tastiera sola, e comportamenti in caso di lock/wake del PC.
5. Documentare ogni nuova prova utente con data, firmware, condizioni e risultato. Non trattare screenshot UI o CI come dimostrazione del risultato fisico.

## 8. Prompt per Claude Code

> Leggi integralmente il CLAUDE.md e ispeziona il branch `experiment-spectrum-resync-on-wake`. Il problema prioritario è il seguente: **se solo mouse o tastiera vanno in standby e si riattivano, Spectrum si risincronizza; se entrambi vanno in standby e vengono poi risvegliati in sequenza, non si risincronizza**. Deve funzionare in entrambi gli ordini, anche con lunga distanza temporale tra risvegli, senza disattivare lo sleep e senza HID polling costante. Prima analizza criticamente Raw Input, state machine, re-enumeration, rescan, disponibilità Device e resync_spectrum. Aggiungi log diagnostici e test dei due dispositivi dormienti, individua la causa anziché ipotizzarla come certa; proponi e implementa la correzione in un branch sperimentale, mantenendo la PR RGB isolata. Non dichiarare riuscita senza test hardware dell'utente e non riconnettere mouse e tastiera allo stesso dongle.
