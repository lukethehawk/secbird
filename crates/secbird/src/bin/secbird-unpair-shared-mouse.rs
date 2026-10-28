//! One-off, guarded recovery utility for an experimentally verified 02C9 receiver.
//!
//! This deliberately does not change the Secretarybird CLI or run automatically.
//! It reads the receiver's paired list and optionally sends exactly ONE 00:42
//! command for the Basilisk V4 HyperSpeed PID 00EA. Never unpairs the keyboard.
//! 00:42 is documented by OpenRazer for other Razer receivers. Its behaviour on
//! this particular 02C9 firmware is unverified, so user confirmation is required.

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
const UNPAIR: Command = Command::new(0x00, 0x42);

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

fn describe(slots: &[Slot]) {
    println!("Receiver 1532:02C9 paired-device table:");
    for slot in slots {
        println!(
            "  PID {:04X}: {}",
            slot.pid,
            if slot.connected == 1 { "connected" } else { "not currently connected (still listed)" }
        );
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() && args != ["--execute"] {
        return Err(Error::InvalidArg(
            "usage: secbird-unpair-shared-mouse.exe [--execute]; default is read-only preview".into(),
        ));
    }
    let execute = args == ["--execute"];
    println!("BlackWidow 02C9 / Basilisk 00EA pairing recovery (experimental)");
    println!("Protocol source: OpenRazer 00:42 [target PID], not yet validated on 02C9.");
    println!("Default: read-only. The --execute option requires interactive confirmation.\n");

    let api = HidApi::new()?;
    let receiver_ifaces: Vec<_> = candidates(&api)
        .into_iter()
        .filter(|c| c.pid == KEYBOARD_RECEIVER && c.interface == 2)
        .collect();
    if receiver_ifaces.len() != 1 {
        return Err(Error::InvalidArg(format!(
            "expected one physical 02C9 HID interface 2; found {}. Nothing changed.",
            receiver_ifaces.len()
        )));
    }

    let db = Db::builtin()?;
    let devices = discover(&api, &db);
    let mut keyboards = devices.into_iter().filter(|d| {
        d.pid == KEYBOARD_RECEIVER && d.interface == 2 && d.txid == KEYBOARD_TXID
    });
    let dev = keyboards.next().ok_or_else(|| Error::NotFound(
        "BlackWidow on 02C9/txid FF is unavailable; ensure keyboard is awake in 2.4 GHz mode. Nothing changed.".into()
    ))?;
    if keyboards.next().is_some() {
        return Err(Error::InvalidArg("multiple BlackWidow devices found; nothing changed".into()));
    }
    let slots = read_slots(&dev)?;
    describe(&slots);

    // Fail closed unless the table matches the user's observed configuration.
    let correct_table = slots.len() == 2
        && slots.iter().any(|s| s.pid == KEYBOARD_RECEIVER && s.connected == 1)
        && slots.iter().any(|s| s.pid == MOUSE_PID && (s.connected == 0 || s.connected == 1));
    if !correct_table {
        println!("\nThe expected keyboard + mouse pairing was NOT found. No command will be sent.");
        if execute {
            return Err(Error::InvalidArg("preflight did not match; no changes made".into()));
        }
        return Ok(());
    }

    println!("\nPreflight OK: keyboard 02C9 present, Basilisk 00EA still listed.");
    if !execute {
        println!("READ-ONLY PREVIEW COMPLETE. No pairing was changed.");
        println!("To attempt removal: run this same tool with --execute.");
        return Ok(());
    }

    println!("\nWARNING: this action changes pairing stored by the receiver.");
    println!("The 00:42 command is known for other Razer dongles, not verified on this 02C9.");
    println!("Have the keyboard USB cable available if 2.4 GHz pairing needs recovery.");
    println!("ONLY target PID 00EA will be sent; no firmware update/reset.");
    print!("To proceed, type exactly UNPAIR 00EA and press Enter: ");
    io::stdout().flush().map_err(|e| Error::System(e.to_string()))?;
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| Error::System(e.to_string()))?;
    if input.trim() != "UNPAIR 00EA" {
        println!("Cancelled: no command sent.");
        return Ok(());
    }

    // A single, explicitly requested SET, with strict FF transaction routing.
    // Never retry automatically: a timeout may occur AFTER the receiver changed state.
    let outcome = dev.transact(UNPAIR, 2, &MOUSE_PID.to_be_bytes());
    match &outcome {
        Ok(r) => println!("00:42 response: Success (reply txid {:02X})", r.txid()),
        Err(e) => println!("00:42 response: {e} (state may still have changed)"),
    }
    sleep(Duration::from_millis(500));
    match read_slots(&dev) {
        Ok(after) => {
            describe(&after);
            if after.iter().any(|s| s.pid == KEYBOARD_RECEIVER)
                && !after.iter().any(|s| s.pid == MOUSE_PID)
            {
                println!("RESULT: Basilisk 00EA is absent; keyboard 02C9 remains paired.");
            } else {
                println!("RESULT INCONCLUSIVE: do NOT repeat the command.");
                println!("Unplug/replug the BlackWidow dongle, then run a read-only preview.");
            }
        }
        Err(e) => {
            println!("Unable to verify updated table: {e}");
            println!("Do NOT retry automatically. Check with the read-only preview after reconnecting.");
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_exact_observed_slots() {
        assert_eq!(
            decode_slots(&[2, 1, 0, 0xEA, 1, 2, 0xC9]).unwrap(),
            vec![Slot { connected: 1, pid: 0x00EA }, Slot { connected: 1, pid: 0x02C9 }]
        );
        assert_eq!(
            decode_slots(&[2, 0, 0, 0xEA, 1, 2, 0xC9]).unwrap(),
            vec![Slot { connected: 0, pid: 0x00EA }, Slot { connected: 1, pid: 0x02C9 }]
        );
        assert!(decode_slots(&[2, 1, 0, 0xEA]).is_err());
        assert!(decode_slots(&[255]).is_err());
    }

    #[test]
    fn unpair_packet_targets_only_mouse() {
        let request = secbird_core::proto::Report::request(KEYBOARD_TXID, UNPAIR, 2, &MOUSE_PID.to_be_bytes());
        assert_eq!(&request.0[..10], &[0, 0xFF, 0, 0, 0, 2, 0, 0x42, 0, 0xEA]);
    }
}
