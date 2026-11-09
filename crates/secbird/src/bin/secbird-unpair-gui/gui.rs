//! Windows native UI: portable, no WebView, no extra runtime.
use super::backend::{self, Receiver, ScanResult, UnpairResult};
use std::cell::RefCell;
use std::ffi::c_void;
use std::mem::zeroed;
use std::ptr::{null, null_mut};
use std::thread;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Graphics::Gdi::{GetStockObject, DEFAULT_GUI_FONT};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

const CLASS_NAME: &str = "Secretarybird.Unpair.Gui";
const WORK_DONE: u32 = WM_APP + 1;
const ID_SCAN: u16 = 101;
const ID_RECEIVER: u16 = 102;
const ID_PAIRING: u16 = 103;
const ID_CONFIRM: u16 = 104;
const ID_REMOVE: u16 = 105;
const ID_COPY: u16 = 106;

enum WorkerReply {
    Scan(Result<ScanResult, String>),
    Unpair(Result<UnpairResult, String>),
}

struct State {
    hwnd: HWND,
    receiver_list: HWND,
    pairing_list: HWND,
    scan_btn: HWND,
    remove_btn: HWND,
    confirmation: HWND,
    confirm_label: HWND,
    status: HWND,
    log: HWND,
    receivers: Vec<Receiver>,
    selected_receiver: Option<usize>,
    selected_slot: Option<usize>,
    busy: bool,
}
thread_local! {
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
}

fn with_state(f: impl FnOnce(&mut State)) {
    STATE.with(|s| {
        if let Some(app) = s.borrow_mut().as_mut() {
            f(app);
        }
    });
}
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
unsafe fn widget(parent: HWND, class: &str, name: &str, x: i32, y: i32, w: i32, h: i32, style: u32, id: u16) -> HWND {
    let handle = CreateWindowExW(
        0, wide(class).as_ptr(), wide(name).as_ptr(),
        WS_CHILD | WS_VISIBLE | style,
        x, y, w, h, parent, id as usize as HMENU, GetModuleHandleW(null()), null_mut(),
    );
    let font = GetStockObject(DEFAULT_GUI_FONT);
    SendMessageW(handle, WM_SETFONT, font as WPARAM, 1);
    handle
}
unsafe fn label(parent: HWND, title: &str, x: i32, y: i32, w: i32, h: i32) -> HWND {
    widget(parent, "STATIC", title, x, y, w, h, 0, 0)
}
unsafe fn button(parent: HWND, title: &str, x: i32, y: i32, w: i32, h: i32, id: u16) -> HWND {
    widget(parent, "BUTTON", title, x, y, w, h, BS_PUSHBUTTON | WS_TABSTOP, id)
}
unsafe fn set_text(hwnd: HWND, message: &str) {
    SetWindowTextW(hwnd, wide(message).as_ptr());
}
unsafe fn clear_list(hwnd: HWND) {
    SendMessageW(hwnd, LB_RESETCONTENT, 0, 0);
}
unsafe fn add_list(hwnd: HWND, s: &str) {
    SendMessageW(hwnd, LB_ADDSTRING, 0, wide(s).as_ptr() as LPARAM);
}
unsafe fn list_selection(hwnd: HWND) -> Option<usize> {
    let index = SendMessageW(hwnd, LB_GETCURSEL, 0, 0);
    (index != LB_ERR as isize && index >= 0).then_some(index as usize)
}
unsafe fn get_text(hwnd: HWND) -> String {
    let len = GetWindowTextLengthW(hwnd);
    let mut b = vec![0u16; len.max(0) as usize + 1];
    let n = GetWindowTextW(hwnd, b.as_mut_ptr(), b.len() as i32);
    String::from_utf16_lossy(&b[..n.max(0) as usize])
}
fn start_worker(hwnd: HWND, operation: impl FnOnce() -> WorkerReply + Send + 'static) {
    let id = hwnd as usize;
    thread::spawn(move || {
        let ptr = Box::into_raw(Box::new(operation()));
        if unsafe { PostMessageW(id as HWND, WORK_DONE, 0, ptr as LPARAM) } == 0 {
            unsafe { drop(Box::from_raw(ptr)); }
        }
    });
}

impl State {
    unsafe fn create(hwnd: HWND) -> Self {
        label(hwnd, "RAZER HYPERSPEED  /  GESTIONE ASSOCIAZIONI", 22, 14, 850, 24);
        label(hwnd, "Leggi i ricevitori e scegli quale associazione rimuovere. Non modifica nulla senza conferma.", 22, 41, 860, 21);
        label(hwnd, "1. Ricevitore USB", 22, 79, 290, 24);
        label(hwnd, "2. Dispositivo associato", 468, 79, 290, 24);
        let scan_btn = button(hwnd, "Rileva / aggiorna", 292, 76, 146, 29, ID_SCAN);
        let receiver_list = widget(hwnd, "LISTBOX", "", 22, 110, 416, 177,
            LBS_NOTIFY as u32 | WS_BORDER | WS_VSCROLL | WS_TABSTOP, ID_RECEIVER);
        let pairing_list = widget(hwnd, "LISTBOX", "", 468, 110, 418, 177,
            LBS_NOTIFY as u32 | WS_BORDER | WS_VSCROLL | WS_TABSTOP, ID_PAIRING);
        label(hwnd, "3. Conferma manuale (nessuna rimozione automatica)", 22, 308, 710, 23);
        let confirm_label = label(hwnd, "Seleziona un'associazione. Le operazioni non validate sono sperimentali.", 22, 333, 850, 39);
        let confirmation = widget(hwnd, "EDIT", "", 22, 376, 482, 31,
            ES_AUTOHSCROLL | WS_BORDER | WS_TABSTOP, ID_CONFIRM);
        let remove_btn = button(hwnd, "Rimuovi associazione", 520, 376, 205, 32, ID_REMOVE);
        label(hwnd, "Stato e diagnostica", 22, 426, 240, 22);
        let status = label(hwnd, "Pronto. Premi Rileva / aggiorna.", 22, 453, 858, 37);
        let log = widget(hwnd, "EDIT", "Nessun comando HID di modifica inviato.", 22, 500, 864, 135,
            ES_MULTILINE | ES_READONLY | ES_AUTOVSCROLL | WS_BORDER | WS_VSCROLL, 0);
        EnableWindow(remove_btn, 0);
        let _ = SetFocus(scan_btn);
        Self { hwnd, receiver_list, pairing_list, scan_btn, remove_btn,
            confirmation, confirm_label, status, log,
            receivers: Vec::new(), selected_receiver: None, selected_slot: None, busy: false }
    }

    unsafe fn busy(&mut self, on: bool) {
        self.busy = on;
        EnableWindow(self.scan_btn, (!on) as i32);
        EnableWindow(self.receiver_list, (!on) as i32);
        EnableWindow(self.pairing_list, (!on) as i32);
        EnableWindow(self.confirmation, (!on) as i32);
        if on {
            EnableWindow(self.remove_btn, 0);
        } else {
            self.pick_slot();
        }
    }
    unsafe fn scan(&mut self) {
        if self.busy { return; }
        self.busy(true);
        set_text(self.status, "Scansione HID Razer in corso: solo letture...");
        start_worker(self.hwnd, || WorkerReply::Scan(backend::scan()));
    }
    unsafe fn pick_receiver(&mut self) {
        self.selected_receiver = list_selection(self.receiver_list);
        self.selected_slot = None;
        clear_list(self.pairing_list);
        set_text(self.confirmation, "");
        if let Some(r) = self.selected_receiver.and_then(|i| self.receivers.get(i)) {
            for slot in &r.slots {
                add_list(self.pairing_list, &backend::slot_text(*slot));
            }
            set_text(self.status, &format!("Ricevitore: {}. {} slot nella tabella.", r.name, r.slots.len()));
        } else {
            set_text(self.status, "Seleziona un ricevitore dalla lista di sinistra.");
        }
        self.pick_slot();
    }
    unsafe fn pick_slot(&mut self) {
        self.selected_slot = list_selection(self.pairing_list);
        EnableWindow(self.remove_btn, 0);
        set_text(self.confirmation, "");
        let Some(r) = self.selected_receiver.and_then(|i| self.receivers.get(i)) else {
            set_text(self.confirm_label, "Seleziona un ricevitore.");
            return;
        };
        let Some(s) = self.selected_slot.and_then(|i| r.slots.get(i)) else {
            set_text(self.confirm_label, "Seleziona il dispositivo associato da rimuovere.");
            return;
        };
        match backend::eligibility(r, s.pid) {
            Ok(verified) => {
                let command = if verified {
                    format!("UNPAIR {:04X}", s.pid)
                } else {
                    format!("EXPERIMENTAL UNPAIR {:04X}", s.pid)
                };
                set_text(self.confirm_label, &format!(
                    "{}\r\nPer sbloccare il pulsante, scrivi esattamente: {}",
                    if verified { "Caso 02C9/00EA gia' verificato su hardware reale." }
                    else { "ATTENZIONE: ricevitore NON verificato. Rischio di perdita del pairing." },
                    command,
                ));
            }
            Err(why) => set_text(self.confirm_label, &format!("Operazione bloccata: {why}")),
        }
    }
    unsafe fn confirmation_changed(&mut self) {
        let enabled = self.selected_receiver.and_then(|i| self.receivers.get(i))
            .and_then(|r| self.selected_slot.and_then(|j| r.slots.get(j))
                .and_then(|s| backend::eligibility(r, s.pid).ok().map(|verified| (verified, s.pid))))
            .is_some_and(|(verified, pid)| {
                let expected = if verified { format!("UNPAIR {pid:04X}") }
                    else { format!("EXPERIMENTAL UNPAIR {pid:04X}") };
                get_text(self.confirmation).trim() == expected
            });
        EnableWindow(self.remove_btn, (enabled && !self.busy) as i32);
    }
    unsafe fn remove(&mut self) {
        if self.busy { return; }
        let Some(r) = self.selected_receiver.and_then(|i| self.receivers.get(i)) else { return; };
        let Some(slot) = self.selected_slot.and_then(|i| r.slots.get(i)) else { return; };
        let Ok(verified) = backend::eligibility(r, slot.pid) else { return; };
        let expected = if verified { format!("UNPAIR {:04X}", slot.pid) }
            else { format!("EXPERIMENTAL UNPAIR {:04X}", slot.pid) };
        if get_text(self.confirmation).trim() != expected { return; }
        let alert = format!(
            "Ricevitore: {}\r\nDispositivo da dimenticare: {} (PID {:04X})\r\n\r\n\
            Il dispositivo potrebbe perdere il collegamento radio con questo dongle.\r\n\
            {} \r\n\r\n\
            Continua? VERRA' INVIATO UN SOLO COMANDO 00:42. Nessun retry automatico.",
            r.name, backend::device_name(slot.pid), slot.pid,
            if verified { "Caso verificato su 02C9: rimozione del solo mouse." }
            else { "CASO SPERIMENTALE: questo ricevitore NON e' stato validato. Tieni un collegamento USB di riserva." }
        );
        if MessageBoxW(self.hwnd, wide(&alert).as_ptr(), wide("Conferma rimozione selettiva").as_ptr(),
            MB_YESNO | MB_DEFBUTTON2 | MB_ICONWARNING) != IDYES { return; }
        let r = r.clone();
        let target = slot.pid;
        self.busy(true);
        set_text(self.status, "Preflight: ricontrollo del dongle e della tabella prima di un singolo comando...");
        start_worker(self.hwnd, move || WorkerReply::Unpair(backend::unpair(r, target, verified)));
    }
    unsafe fn done(&mut self, msg: WorkerReply) {
        self.busy(false);
        match msg {
            WorkerReply::Scan(result) => match result {
                Ok(result) => {
                    self.receivers = result.receivers;
                    self.selected_receiver = None;
                    self.selected_slot = None;
                    clear_list(self.receiver_list);
                    clear_list(self.pairing_list);
                    for r in &self.receivers {
                        add_list(self.receiver_list, &r.name);
                    }
                    let text = result.details;
                    set_text(self.status, &text);
                    if self.receivers.is_empty() {
                        set_text(self.log, "Nessuna tabella associazioni compatibile. Verifica che i dongle siano collegati, i dispositivi svegli e Secretarybird chiuso.\r\nNessun comando di modifica inviato.");
                    } else {
                        set_text(self.log, "Scansione terminata, sola lettura. Seleziona un ricevitore e un dispositivo.\r\nLa compatibilita' del comando UNPAIR non e' generale: il caso 02C9/00EA e' quello verificato.");
                        SendMessageW(self.receiver_list, LB_SETCURSEL, 0, 0);
                        self.pick_receiver();
                    }
                }
                Err(err) => {
                    set_text(self.status, "Scansione fallita. Non e' stato inviato alcun comando di modifica.");
                    set_text(self.log, &err);
                }
            },
            WorkerReply::Unpair(result) => match result {
                Ok(out) => {
                    set_text(self.log, &out.message);
                    set_text(self.status, if out.verified {
                        "Associazione rimossa e verificata. Premi Rileva / aggiorna."
                    } else {
                        "Esito non confermato. NON ripetere l'operazione. Controlla il log."
                    });
                }
                Err(err) => {
                    set_text(self.log, &format!("Operazione bloccata prima del comando oppure interrotta.\r\n{err}"));
                    set_text(self.status, "Sicurezza: nessun comando inviato se il preflight e' fallito.");
                }
            }
        }
        set_text(self.confirmation, "");
        EnableWindow(self.remove_btn, 0);
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            let app = State::create(hwnd);
            STATE.with(|cell| *cell.borrow_mut() = Some(app));
            0
        }
        WM_COMMAND => {
            let id = (w & 0xFFFF) as u16;
            let notification = ((w >> 16) & 0xFFFF) as u16;
            with_state(|s| unsafe {
                match (id, notification as u32) {
                    (ID_SCAN, BN_CLICKED) => s.scan(),
                    (ID_RECEIVER, LBN_SELCHANGE) => s.pick_receiver(),
                    (ID_PAIRING, LBN_SELCHANGE) => s.pick_slot(),
                    (ID_CONFIRM, EN_CHANGE) => s.confirmation_changed(),
                    (ID_REMOVE, BN_CLICKED) => s.remove(),
                    _ => {}
                }
            });
            0
        }
        WORK_DONE => {
            if l != 0 {
                let result = Box::from_raw(l as *mut WorkerReply);
                with_state(|s| unsafe { s.done(*result) });
            }
            0
        }
        WM_DESTROY => {
            STATE.with(|cell| *cell.borrow_mut() = None);
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, w, l),
    }
}

pub fn run() {
    unsafe {
        let instance = GetModuleHandleW(null());
        let class = wide(CLASS_NAME);
        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: instance,
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground: (COLOR_WINDOW as usize + 1) as HBRUSH,
            lpszClassName: class.as_ptr(),
            ..zeroed()
        };
        if RegisterClassW(&wc) == 0 {
            return;
        }
        let title = wide("Secretarybird  |  HyperSpeed Unpair Utility");
        let hwnd = CreateWindowExW(0, class.as_ptr(), title.as_ptr(),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
            CW_USEDEFAULT, CW_USEDEFAULT, 923, 697,
            null_mut(), null_mut(), instance, null_mut());
        if hwnd.is_null() { return; }
        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
        let mut message: MSG = zeroed();
        while GetMessageW(&mut message, null_mut(), 0, 0) > 0 {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
}
