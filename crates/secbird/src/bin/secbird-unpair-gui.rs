//! Standalone native Windows GUI for guarded Razer receiver unpairing.
//! Read-only until the user selects an explicit target and confirms it.
#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
#[path = "secbird-unpair-gui/backend.rs"]
mod backend;
#[cfg(windows)]
#[path = "secbird-unpair-gui/gui.rs"]
mod gui;

#[cfg(windows)]
fn main() {
    gui::run();
}

#[cfg(not(windows))]
fn main() {
    eprintln!("secbird-unpair-gui runs only on Windows");
}
