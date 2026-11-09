//! HID backend for the portable, read-only-by-default unpair GUI.
//! Never initiate pairing. Never send 00:42 twice. Fail closed on ambiguity.
use secbird_core::commands as cmd;
use secbird_core::device::{candidates, open_interface};
use secbird_core::hidapi::HidApi;
use secbird_core::proto::Command;
use secbird_core::{discover, Db, Device};
use std::ffi::CString;
use std::process::Command as ProcessCommand;
use std::thread::sleep;
use std::time::Duration;

const BLACKWIDOW: u16 = 0x02C9;
const BASILISK: u16 = 0x00EA;
const TXID_KEYBOARD: u8 = 0xFF;
const UNPAIR: Command = Command::new(0x00, 0x42);
const EMPTY: u16 = 0xFFFF;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slot {
    pub pid: u16,
    pub connected: u8,
}

#[derive(Clone)]
pub struct Receiver {
    pub pid: u16,
    pub interface: i32,
    pub txid: u8,
    pub path: Vec<u8>,
    pub name: String,
    pub slots: Vec<Slot>,
}

pub struct ScanResult {
    pub receivers: Vec<Receiver>,
    pub details: String,
}

pub struct UnpairResult {
    pub message: String,
    pub verified: bool,
}

/// Returns no candidate on unsupported/ambiguous reply. No writes involved.
pub fn parse_slots(args: &[u8]) -> Result<Vec<Slot>, String> {
    let n = usize::from(*args.first().ok_or("Risposta associazioni vuota")?);
    if !(1..=8).contains(&n) {
        return Err(format!("Numero slot sospetto: {n}"));
    }
    let b = args.get(1..1 + 3 * n).ok_or("Risposta associazioni incompleta")?;
    let slots = b.as_chunks::<3>().0.iter().map(|a| Slot {
        connected: a[0],
        pid: u16::from_be_bytes([a[1], a[2]]),
    }).collect::<Vec<_>>();
    if slots.iter().any(|s| s.connected > 1 || s.pid == 0) {
        return Err("Tabella non conforme; nessuna azione consentita".into());
    }
    Ok(slots)
}

fn read_slots(dev: &Device) -> Result<Vec<Slot>, String> {
    let reply = dev.get(cmd::WIRELESS_DEVICES, cmd::WIRELESS_DEVICES_SIZE, &[])
        .map_err(|e| e.to_string())?;
    if reply.txid() != dev.txid {
        return Err("Il dongle ha risposto con un transaction ID diverso: operazione non sicura".into());
    }
    parse_slots(reply.args())
}

pub fn device_name(pid: u16) -> &'static str {
    match pid {
        BLACKWIDOW => "BlackWidow V4 Low-Profile HyperSpeed",
        BASILISK => "Basilisk V4 HyperSpeed",
        EMPTY => "Slot libero",
        _ => "Periferica Razer",
    }
}
pub fn slot_text(slot: Slot) -> String {
    let state = match (slot.pid, slot.connected) {
        (EMPTY, _) => "libero",
        (_, 1) => "connesso",
        _ => "associato, ora disconnesso",
    };
    format!("{}  [PID {:04X}]  -  {}", device_name(slot.pid), slot.pid, state)
}

fn receiver_text(dev: &Device) -> String {
    let name = if dev.pid == BLACKWIDOW {
        "Ricevitore BlackWidow V4"
    } else if dev.pid == BASILISK {
        "Ricevitore Basilisk V4"
    } else {
        "Dispositivo / ricevitore Razer"
    };
    format!("{name}  |  USB {:04X}  |  IF {}  |  TX {:02X}", dev.pid, dev.interface, dev.txid)
}

/// Only two well-understood targets for this user's actual mouse/keyboard.
/// Never unpair the receiver's host device or a device not explicitly in its list.
pub fn eligibility(r: &Receiver, target: u16) -> Result<bool, String> {
    if target == EMPTY || ![BASILISK, BLACKWIDOW].contains(&target) {
        return Err("Rimozione consentita solo per BlackWidow 02C9 o Basilisk 00EA".into());
    }
    if target == r.pid {
        return Err("Non puoi rimuovere dal proprio ricevitore il dispositivo principale".into());
    }
    if r.slots.iter().filter(|s| s.pid == target).count() != 1 {
        return Err("Il dispositivo non compare una volta sola nella lista delle associazioni".into());
    }
    if r.slots.iter().filter(|s| s.pid == r.pid && s.connected <= 1).count() != 1 {
        return Err("Il dispositivo principale del ricevitore non è identificabile senza ambiguità".into());
    }
    if r.slots.iter().any(|s| s.pid != EMPTY && s.pid != r.pid && s.pid != target) {
        return Err("La tabella contiene altre periferiche: non è un caso supportato".into());
    }
    let verified = r.pid == BLACKWIDOW
        && r.interface == 2
        && r.txid == TXID_KEYBOARD
        && target == BASILISK
        && r.slots.iter().any(|s| s.pid == BLACKWIDOW && s.connected == 1);
    Ok(verified)
}

pub fn scan() -> Result<ScanResult, String> {
    let mut api = HidApi::new().map_err(|e| e.to_string())?;
    api.refresh_devices().map_err(|e| e.to_string())?;
    let db = Db::builtin().map_err(|e| e.to_string())?;
    let physical = candidates(&api);
    let mut devices = discover(&api, &db);
    // The 02C9 receiver also answers for a mouse on 1F. Probe FF explicitly
    // so that a shared dongle can never be mistaken for the attached mouse.
    for c in physical.iter().filter(|c| c.pid == BLACKWIDOW && c.interface == 2) {
        if let Some(dev) = open_interface(&api, &c.path, c.pid, c.interface, &[TXID_KEYBOARD], db.spec(c.pid)) {
            devices.retain(|other| other.path != dev.path);
            devices.push(dev);
        }
    }
    let total = devices.len();
    let mut receivers = Vec::new();
    let mut ignored = Vec::new();
    for dev in devices {
        // Ignore SECBIRD_SIM entries and virtual paths: real HID candidate only.
        if !physical.iter().any(|c| c.pid == dev.pid && c.path == dev.path) {
            continue;
        }
        match read_slots(&dev) {
            Ok(slots) => {
                // Never show the same physical interface twice.
                if receivers.iter().any(|r: &Receiver| r.path == dev.path.to_bytes()) {
                    continue;
                }
                receivers.push(Receiver {
                    pid: dev.pid,
                    interface: dev.interface,
                    txid: dev.txid,
                    path: dev.path.to_bytes().to_vec(),
                    name: receiver_text(&dev),
                    slots,
                });
            }
            Err(_) => ignored.push(format!("{:04X}", dev.pid)),
        }
    }
    let details = format!(
        "{} dispositivi HID Razer rilevati. {} ricevitori con una tabella associazioni leggibile.{}",
        total, receivers.len(),
        if ignored.is_empty() { String::new() } else {
            format!("  Altri dispositivi senza lista compatibile: {}.", ignored.join(", "))
        }
    );
    Ok(ScanResult { receivers, details })
}

fn tray_running() -> Result<bool, String> {
    let out = ProcessCommand::new("tasklist")
        .args(["/FI", "IMAGENAME eq secbird-tray.exe", "/NH"])
        .output().map_err(|e| format!("Impossibile controllare Secretarybird: {e}"))?;
    if !out.status.success() {
        return Err("Impossibile verificare che Secretarybird sia chiuso".into());
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_ascii_lowercase().contains("secbird-tray.exe"))
}

/// Reopen the *same physical HID path*, redo the entire read-only preflight,
/// send exactly one unpair SET, and check afterwards. Never auto-retry.
pub fn unpair(r: Receiver, target: u16, was_verified: bool) -> Result<UnpairResult, String> {
    if tray_running()? {
        return Err("Chiudi Secretarybird completamente dalla tray (Exit) prima di scollegare un'associazione".into());
    }
    let verified = eligibility(&r, target)?;
    if verified != was_verified {
        return Err("La modalità di sicurezza è cambiata: aggiornare la lista".into());
    }
    let mut api = HidApi::new().map_err(|e| e.to_string())?;
    api.refresh_devices().map_err(|e| e.to_string())?;
    let physical = candidates(&api);
    let path = CString::new(r.path.clone()).map_err(|_| "Percorso HID non valido")?;
    let matching: Vec<_> = physical.iter().filter(|c| {
        c.pid == r.pid && c.interface == r.interface && c.path == path
    }).collect();
    if matching.len() != 1 {
        return Err("Il ricevitore scelto non è più presente esattamente una volta. Nessuna modifica".into());
    }
    let db = Db::builtin().map_err(|e| e.to_string())?;
    let dev = open_interface(&api, &path, r.pid, r.interface, &[r.txid], db.spec(r.pid))
        .ok_or("Il ricevitore non risponde al transaction ID originale. Nessuna modifica")?;
    if dev.txid != r.txid {
        return Err("Transaction ID cambiato. Nessuna modifica".into());
    }
    let before = read_slots(&dev)?;
    if before != r.slots {
        return Err("La tabella delle associazioni è cambiata dopo la scansione. Rileggi la lista; nessuna modifica".into());
    }
    if eligibility(&r, target)? != verified {
        return Err("Controlli di sicurezza non superati. Nessuna modifica".into());
    }
    // Sending precisely ONE SET 00:42; transact_as retries only reads of the reply.
    let response = dev.transact_as(r.txid, UNPAIR, 2, &target.to_be_bytes());
    sleep(Duration::from_millis(650));
    let after = read_slots(&dev);
    let retained_host = after.as_ref().is_ok_and(|v| v.iter().any(|s| s.pid == r.pid));
    let removed_target = after.as_ref().is_ok_and(|v| !v.iter().any(|s| s.pid == target));
    let success = retained_host && removed_target;
    let command = match &response {
        Ok(reply) => format!("SET 00:42: risposta OK, txid {:02X}.", reply.txid()),
        Err(e) => format!("SET 00:42: risposta incerta ({e})."),
    };
    let detail = match &after {
        Ok(v) => v.iter().map(|s| slot_text(*s)).collect::<Vec<_>>().join("\r\n"),
        Err(e) => format!("Non sono riuscito a rileggere la tabella: {e}"),
    };
    let message = format!(
        "{}\r\n\r\n{}\r\n\r\n{}\r\n\r\n{}",
        if success { "VERIFICATO: il dispositivo scelto non figura più nella lista, il principale è presente." }
        else { "ESITO INCERTO: NON ripetere l'operazione automaticamente. Scollega/ricollega il dongle e controlla la lista." },
        command, detail,
        if verified { "Profilo: 02C9 / 00EA verificato in precedenza." }
        else { "Profilo: ricevitore diverso - comportamento sperimentale, non validato sul tuo hardware." }
    );
    Ok(UnpairResult { message, verified: success })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn receiver(pid: u16, txid: u8, iface: i32, slots: &[Slot]) -> Receiver {
        Receiver { pid, txid, interface: iface, path: Vec::new(),
            name: String::new(), slots: slots.to_vec() }
    }
    const K: Slot = Slot { pid: BLACKWIDOW, connected: 1 };
    const M: Slot = Slot { pid: BASILISK, connected: 1 };

    #[test]
    fn parses_verified_keyboard_plus_mouse_reply() {
        assert_eq!(parse_slots(&[2, 1, 0, 0xEA, 1, 2, 0xC9]).unwrap(), vec![M, K]);
        assert_eq!(parse_slots(&[2, 0, 0, 0xEA, 1, 2, 0xC9]).unwrap()[0].connected, 0);
        assert!(parse_slots(&[2, 1, 0, 0xEA]).is_err());
        assert!(parse_slots(&[255]).is_err());
        assert!(parse_slots(&[1, 8, 0, 0xEA]).is_err());
    }

    #[test]
    fn verified_case_only_keyboard_receiver_to_mouse() {
        let r = receiver(BLACKWIDOW, TXID_KEYBOARD, 2, &[M, K]);
        assert_eq!(eligibility(&r, BASILISK), Ok(true));
        assert!(eligibility(&r, BLACKWIDOW).is_err());
        assert_eq!(eligibility(&receiver(BASILISK, 0x1F, 2, &[K, M]), BLACKWIDOW), Ok(false));
    }

    #[test]
    fn fails_closed_on_ambiguous_or_unrelated_slots() {
        assert!(eligibility(&receiver(BLACKWIDOW, TXID_KEYBOARD, 2, &[M, M, K]), BASILISK).is_err());
        assert!(eligibility(&receiver(BLACKWIDOW, TXID_KEYBOARD, 2, &[M]), BASILISK).is_err());
        assert!(eligibility(&receiver(BLACKWIDOW, TXID_KEYBOARD, 2, &[M, K, Slot { pid: 0x0011, connected: 1 }]), BASILISK).is_err());
        assert!(eligibility(&receiver(BASILISK, 0x1F, 2, &[K, M]), BASILISK).is_err());
    }

    #[test]
    fn unpair_is_one_pid_big_endian_packet() {
        let req = secbird_core::proto::Report::request(TXID_KEYBOARD, UNPAIR, 2, &BASILISK.to_be_bytes());
        assert_eq!(&req.0[..10], &[0, 0xFF, 0, 0, 0, 2, 0, 0x42, 0, 0xEA]);
    }
}
