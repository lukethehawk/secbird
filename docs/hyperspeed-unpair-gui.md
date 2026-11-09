# Secretarybird HyperSpeed Unpair GUI (Windows)

Standalone Windows x64 program to **inspect** Razer wireless receiver pairings and, where allowed, issue a **single explicitly confirmed** unpair request. It is not part of the Secretarybird tray: no installation, Synapse, browser or .NET runtime is needed.

**Status:** experimental build. The only real-hardware unpair command verified in this project is removing a **Basilisk V4 HyperSpeed (00EA) from the BlackWidow V4 Low-Profile receiver (02C9)**. Other directions are **not hardware-verified** and must be treated as risky.

## Download / build

On the branch `tool-hyperspeed-unpair-gui`, open the successful [Windows GitHub Actions workflow](https://github.com/lukethehawk/secbird/actions/workflows/unpair-gui-windows.yml) and download the artifact `Secretarybird-HyperSpeed-Unpair-GUI-Windows`. Extract and run `secbird-unpair-gui.exe`; it is a Windows GUI program, not a command prompt.

For developers on Windows:

```powershell
cargo test --release -p secbird --bin secbird-unpair-gui
cargo build --release -p secbird --bin secbird-unpair-gui
.\target\release\secbird-unpair-gui.exe
```

## How to use

1. Connect the receiver you want to inspect. Wake the keyboard and mouse; optionally unplug unnecessary duplicate receivers to simplify identification.
2. Start the GUI. Click **Rileva / aggiorna**. Detection only reads HID reports.
3. In the left list, choose the **physical USB receiver**. In the right list choose the peripheral **listed as paired with that receiver**. A free `FFFF` slot is not a device.
4. Read the label above the confirmation box. If the table is unsafe or the device is the receiver's own main peripheral, the action is blocked.
5. For the verified BlackWidow 02C9 → Basilisk 00EA case, explicitly type `UNPAIR 00EA`. Any other eligible pairing requires `EXPERIMENTAL UNPAIR <PID>`.
6. Close **Secretarybird tray** completely (Exit) before issuing a change. The program will refuse to unpair while `secbird-tray.exe` is running.
7. Read the final warning and click **Yes** only if you understand that the device will lose this receiver's wireless association.
8. The program reopens the **same physical HID interface**, re-reads the association table, and sends at most **one** vendor HID SET `00:42` with the target PID (big-endian). It reads the table again to verify removal while the receiver's own device remains.
9. **Never blindly repeat** a request with an uncertain outcome. Unplug/replug the receiver and use the read-only scan to verify its pairing state. If needed, use Razer's official pairing utility for recovery.

## Safety and supported scenarios

| Scenario | Status |
| --- | --- |
| Inspect paired devices on receivers that respond to GET `00:BF` | Read-only; supported where firmware exposes the list |
| Remove Basilisk `00EA` from BlackWidow `02C9` USB receiver, interface 2, txid FF | **Previously verified** on actual hardware; still requires confirmation |
| Remove BlackWidow `02C9` from a different USB receiver (e.g., a mouse dongle) | **Experimental, unverified**; only offered if GET `00:BF` explicitly shows it *alongside* the receiver's own main device |
| Remove the receiver's own native peripheral, e.g., BlackWidow from the 02C9 receiver or Basilisk from its 00EA receiver | **Blocked** |
| Unknown/unlisted PID, ambiguous list, duplicate slot, mismatched HID path/transaction ID | **Blocked** |
| Receiver firmware not supporting paired-device list | Not actionable; the application cannot safely guess the association |

Only Razer PIDs `00EA` (Basilisk V4) and `02C9` (BlackWidow V4 LP) can be selected as removal targets in this initial release. Support for additional product families requires documented protocol evidence and hardware testing.

**Important:** the program does *not* pair, reset, update firmware or automatically retry an unpair. A lost association can require Synapse or a Razer pairing utility to restore. Keep a USB cable or spare input device available before making changes.

Razer's official [pairing utilities and Multi-Device guidance](https://mysupport.razer.com/app/answers/detail/a_id/6188) describe pairing and recovery but do not validate this program on every receiver.

## Implementation

- Backend: `crates/secbird/src/bin/secbird-unpair-gui/backend.rs`
- Native Win32 interface: `crates/secbird/src/bin/secbird-unpair-gui/gui.rs`
- GUI executable target: `crates/secbird/src/bin/secbird-unpair-gui.rs`
- Pipeline: `.github/workflows/unpair-gui-windows.yml`
- Stable original one-target recovery utility: branch `tool-unpair-00ea-from-02c9`.

The unit tests cover pairing-table decoding, verified versus experimental cases, disallowed device removal, duplicate/ambiguous slots, and the exact 00:42 request payload. They do **not** substitute for real-hardware tests on other receivers.

If the GUI fails to list a mouse receiver, do not assume the firmware is broken: some dongles do not expose a compatible pairing-table GET. The failure is intentional rather than a blind write.
