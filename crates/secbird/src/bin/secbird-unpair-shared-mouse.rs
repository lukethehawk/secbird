//! Guarded tool that removes the Basilisk V4 HyperSpeed (PID 00EA) from the
//! BlackWidow V4 Low-Profile HyperSpeed receiver (PID 02C9), for when both got
//! paired to the keyboard's receiver.
//!
//! This deliberately does not change the Secretarybird CLI or run automatically.
//! It reads the receiver's paired list and, only after a typed confirmation,
//! sends exactly ONE 00:42 command for PID 00EA. Never unpairs the keyboard.
//! 00:42 comes from OpenRazer; on this 02C9 receiver it was verified once on real
//! hardware: the mouse went back to its own receiver, the keyboard stayed paired.
//!
//! Made to be started with a double click: every step is explained (in Italian,
//! for the user who runs it) and the window waits for Enter before closing.

use secbird_core::commands as cmd;
use secbird_core::device::candidates;
use secbird_core::hidapi::HidApi;
use secbird_core::proto::Command;
use secbird_core::{Db, Device, Error, Result, discover};
use std::io::{self, Write};
use std::process::ExitCode;
use std::thread::sleep;
use std::time::Duration;

const KEYBOARD_RECEIVER: u16 = 0x02C9;
const KEYBOARD_TXID: u8 = 0xFF;
const MOUSE_PID: u16 = 0x00EA;
const FREE_SLOT: u16 = 0xFFFF;
const UNPAIR: Command = Command::new(0x00, 0x42);
const CONFIRM: &str = "UNPAIR 00EA";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Slot {
    connected: u8,
    pid: u16,
}

fn decode_slots(data: &[u8]) -> Result<Vec<Slot>> {
    let count = usize::from(*data.first().ok_or_else(|| Error::InvalidArg("empty paired-device response".into()))?);
    if count > 8 {
        return Err(Error::InvalidArg(format!("unexpected paired-device count {count}")));
    }
    let end = 1 + count * 3;
    let bytes = data.get(1..end).ok_or_else(|| Error::InvalidArg("truncated paired-device response".into()))?;
    Ok(bytes
        .as_chunks::<3>().0
        .iter()
        .map(|e| Slot { connected: e[0], pid: u16::from_be_bytes([e[1], e[2]]) })
        .collect())
}

fn read_slots(dev: &Device) -> Result<Vec<Slot>> {
    let report = dev.get(cmd::WIRELESS_DEVICES, cmd::WIRELESS_DEVICES_SIZE, &[])?;
    if report.txid() != KEYBOARD_TXID {
        return Err(Error::InvalidArg("receiver returned a different transaction ID".into()));
    }
    decode_slots(report.args())
}

/// The table this tool was verified on: the keyboard connected, the mouse listed
/// (connected or not), and nothing else but free slots.
fn mouse_on_keyboard_receiver(slots: &[Slot]) -> bool {
    let count = |pid: u16| slots.iter().filter(|s| s.pid == pid).count();
    count(KEYBOARD_RECEIVER) == 1
        && count(MOUSE_PID) == 1
        && slots.iter().any(|s| s.pid == KEYBOARD_RECEIVER && s.connected == 1)
        && slots.iter().any(|s| s.pid == MOUSE_PID && s.connected <= 1)
        && slots.iter().all(|s| matches!(s.pid, KEYBOARD_RECEIVER | MOUSE_PID | FREE_SLOT))
}

fn describe(slots: &[Slot]) {
    println!("Dispositivi associati al dongle della tastiera (1532:02C9):");
    for slot in slots {
        let name = match slot.pid {
            KEYBOARD_RECEIVER => "02C9, la tastiera BlackWidow".to_string(),
            MOUSE_PID => "00EA, il mouse Basilisk".to_string(),
            FREE_SLOT => "FFFF, posto libero".to_string(),
            pid => format!("{pid:04X}, dispositivo sconosciuto"),
        };
        let state = match (slot.pid, slot.connected) {
            (FREE_SLOT, _) => "",
            (_, 1) => ": collegato",
            _ => ": associato, ora non collegato",
        };
        println!("  {name}{state}");
    }
}

fn read_line() -> Result<String> {
    io::stdout().flush().map_err(|e| Error::System(e.to_string()))?;
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| Error::System(e.to_string()))?;
    Ok(input)
}

/// Secretarybird's tray talks to the same receiver; its requests must not
/// interleave with ours.
fn tray_running() -> bool {
    std::process::Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq secbird-tray.exe", "/NH"])
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).to_ascii_lowercase().contains("secbird-tray.exe"))
        .unwrap_or(false)
}

/// The keyboard's receiver, answering on txid FF, and its paired table.
fn find_keyboard(api: &mut HidApi, db: &Db) -> Result<(Device, Vec<Slot>)> {
    api.refresh_devices()?;
    let receivers = candidates(api)
        .into_iter()
        .filter(|c| c.pid == KEYBOARD_RECEIVER && c.interface == 2)
        .count();
    match receivers {
        0 => return Err(Error::System("Non trovo il dongle della tastiera (1532:02C9). Collegalo.".into())),
        1 => {}
        n => return Err(Error::System(format!("Trovo {n} dongle della tastiera (02C9): lasciane collegato uno solo."))),
    }
    let mut keyboards = discover(api, db)
        .into_iter()
        .filter(|d| d.pid == KEYBOARD_RECEIVER && d.interface == 2 && d.txid == KEYBOARD_TXID);
    let dev = keyboards.next().ok_or_else(|| {
        Error::System("Il dongle c'è, ma la tastiera non risponde. Accendila in modalità 2,4 GHz e premi un tasto per svegliarla.".into())
    })?;
    if keyboards.next().is_some() {
        return Err(Error::System("Trovo più di una BlackWidow: non faccio nulla.".into()));
    }
    let slots = read_slots(&dev)?;
    Ok((dev, slots))
}

fn run() -> Result<()> {
    println!("Togli il mouse Basilisk V4 HyperSpeed (00EA) dal dongle della tastiera BlackWidow (02C9)");
    println!("Prima legge soltanto. Cambia l'associazione solo se lo confermi scrivendo {CONFIRM}.\n");

    while tray_running() {
        println!("Secretarybird è aperto: chiudilo dal vassoio (Exit), poi premi Invio.");
        read_line()?;
    }

    let mut api = HidApi::new()?;
    let db = Db::builtin()?;
    let (dev, slots) = loop {
        match find_keyboard(&mut api, &db) {
            Ok(found) => break found,
            Err(e) => {
                println!("{e}");
                print!("Premi Invio per riprovare, oppure chiudi la finestra: ");
                read_line()?;
                println!();
            }
        }
    };
    describe(&slots);

    if !mouse_on_keyboard_receiver(&slots) {
        let mouse_listed = slots.iter().any(|s| s.pid == MOUSE_PID);
        println!();
        if mouse_listed {
            println!("La tabella non è quella su cui l'operazione è stata provata: non mando nessun comando.");
        } else {
            println!("Il mouse non è associato al dongle della tastiera: non c'è niente da togliere.");
        }
        println!("Nessuna associazione è cambiata.");
        return Ok(());
    }

    println!("\nIl mouse risulta associato al dongle della tastiera.");
    println!("Se confermi, il dongle della tastiera dimentica il mouse (un solo comando, 00:42 per 00EA).");
    println!("La tastiera resta associata. Dopo, il mouse funziona solo con il suo dongle.");
    println!("Tieni a portata di mano il cavo USB della tastiera, nel caso servisse.");
    print!("\nPer procedere scrivi {CONFIRM} e premi Invio. Per uscire senza cambiare nulla premi solo Invio: ");
    if !read_line()?.trim().eq_ignore_ascii_case(CONFIRM) {
        println!("Annullato: nessun comando inviato.");
        return Ok(());
    }

    // A single, explicitly requested SET, with strict FF transaction routing.
    // Never retry automatically: a timeout may occur AFTER the receiver changed state.
    match dev.transact(UNPAIR, 2, &MOUSE_PID.to_be_bytes()) {
        Ok(r) => println!("\nRisposta al comando 00:42: ok (txid {:02X})", r.txid()),
        Err(e) => println!("\nRisposta al comando 00:42: {e}. L'associazione potrebbe essere cambiata lo stesso."),
    }
    sleep(Duration::from_millis(500));
    match read_slots(&dev) {
        Ok(after) => {
            describe(&after);
            if after.iter().any(|s| s.pid == KEYBOARD_RECEIVER) && !after.iter().any(|s| s.pid == MOUSE_PID) {
                println!("\nFATTO: il mouse non è più associato al dongle della tastiera, la tastiera sì.");
                println!("Collega il dongle del mouse: da ora il mouse funziona con quello.");
            } else {
                println!("\nRISULTATO INCERTO: NON ripetere il comando.");
                println!("Scollega e ricollega il dongle della tastiera, poi riapri questo programma solo per leggere la tabella.");
            }
        }
        Err(e) => {
            println!("Non riesco a rileggere la tabella: {e}");
            println!("NON ripetere il comando. Scollega e ricollega il dongle della tastiera, poi riapri questo programma solo per leggere la tabella.");
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    let code = match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            println!("Errore: {e}");
            ExitCode::FAILURE
        }
    };
    print!("\nPremi Invio per chiudere.");
    let _ = read_line();
    code
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slot(connected: u8, pid: u16) -> Slot {
        Slot { connected, pid }
    }

    #[test]
    fn extracts_exact_observed_slots() {
        assert_eq!(decode_slots(&[2, 1, 0, 0xEA, 1, 2, 0xC9]).unwrap(), vec![slot(1, 0x00EA), slot(1, 0x02C9)]);
        assert_eq!(decode_slots(&[2, 0, 0, 0xEA, 1, 2, 0xC9]).unwrap(), vec![slot(0, 0x00EA), slot(1, 0x02C9)]);
        assert!(decode_slots(&[2, 1, 0, 0xEA]).is_err());
        assert!(decode_slots(&[255]).is_err());
    }

    #[test]
    fn unpair_packet_targets_only_mouse() {
        let request = secbird_core::proto::Report::request(KEYBOARD_TXID, UNPAIR, 2, &MOUSE_PID.to_be_bytes());
        assert_eq!(&request.0[..10], &[0, 0xFF, 0, 0, 0, 2, 0, 0x42, 0, 0xEA]);
    }

    #[test]
    fn acts_only_on_the_verified_table() {
        // The tables seen on the real receiver before the first unpair.
        assert!(mouse_on_keyboard_receiver(&[slot(1, MOUSE_PID), slot(1, KEYBOARD_RECEIVER)]));
        assert!(mouse_on_keyboard_receiver(&[slot(0, MOUSE_PID), slot(1, KEYBOARD_RECEIVER)]));
        // A free slot next to them changes nothing.
        assert!(mouse_on_keyboard_receiver(&[slot(0, FREE_SLOT), slot(1, MOUSE_PID), slot(1, KEYBOARD_RECEIVER)]));
        // The table seen after it: nothing to remove.
        assert!(!mouse_on_keyboard_receiver(&[slot(0, FREE_SLOT), slot(1, KEYBOARD_RECEIVER)]));
        // Keyboard asleep or gone, unknown devices, odd states: no command.
        assert!(!mouse_on_keyboard_receiver(&[slot(1, MOUSE_PID), slot(0, KEYBOARD_RECEIVER)]));
        assert!(!mouse_on_keyboard_receiver(&[slot(1, MOUSE_PID), slot(1, KEYBOARD_RECEIVER), slot(1, 0x0099)]));
        assert!(!mouse_on_keyboard_receiver(&[slot(2, MOUSE_PID), slot(1, KEYBOARD_RECEIVER)]));
        assert!(!mouse_on_keyboard_receiver(&[slot(1, MOUSE_PID), slot(1, MOUSE_PID), slot(1, KEYBOARD_RECEIVER)]));
    }
}
