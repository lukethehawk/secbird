//! Passive Spectrum restart scheduling for two devices. No HID polling.
//! The first input after an idle gap marks a wake; restart only when both
//! devices have produced input recently. Works whichever device wakes first.

use std::collections::HashMap;
use std::time::{Duration, Instant};

pub const BLACKWIDOW: u16 = 0x02C9;
pub const BASILISK: u16 = 0x00EA;
const WAKE_IDLE: Duration = Duration::from_secs(60);

#[derive(Default)]
pub struct WakeTracker {
    last_input: HashMap<u16, Instant>,
    pending_wake: bool,
    scheduled: bool,
}

impl WakeTracker {
    /// Returns true once after either device wakes AND both are active.
    /// All ordinary input is just timestamped. No device is queried here.
    pub fn input(&mut self, pid: u16, now: Instant) -> bool {
        if pid != BLACKWIDOW && pid != BASILISK {
            return false;
        }
        if self.last_input.get(&pid).is_none_or(|last| now.duration_since(*last) >= WAKE_IDLE) {
            self.pending_wake = true;
        }
        self.last_input.insert(pid, now);
        let both_active = [BLACKWIDOW, BASILISK].into_iter().all(|id| {
            self.last_input.get(&id).is_some_and(|last| now.duration_since(*last) < WAKE_IDLE)
        });
        if self.pending_wake && both_active && !self.scheduled {
            self.scheduled = true;
            true
        } else {
            false
        }
    }

    /// Even if a device is slow to accept the write, don't send a stream of
    /// retries on every mouse report. The next genuine wake can try again.
    pub fn finished(&mut self) {
        self.scheduled = false;
        self.pending_wake = false;
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    fn scenario(first: u16, second: u16) {
        let mut t = WakeTracker::default();
        let zero = Instant::now();
        assert!(!t.input(first, zero), "only first device active");
        assert!(t.input(second, zero + Duration::from_millis(100)), "second wake synchronizes both");
        t.finished();
        assert!(!t.input(first, zero + Duration::from_secs(1)), "no repeat on ordinary input");
        assert!(!t.input(second, zero + Duration::from_secs(2)), "no repeat on ordinary input");
        let later = zero + Duration::from_secs(90);
        assert!(!t.input(first, later), "other device still inactive");
        assert!(t.input(second, later + Duration::from_millis(100)), "second device re-woke");
    }

    #[test]
    fn mouse_then_keyboard() {
        scenario(BASILISK, BLACKWIDOW);
    }

    #[test]
    fn keyboard_then_mouse() {
        scenario(BLACKWIDOW, BASILISK);
    }

    #[test]
    fn does_not_handle_other_hid_devices() {
        let mut t = WakeTracker::default();
        let now = Instant::now();
        assert!(!t.input(0x02CC, now));
        assert!(!t.input(BASILISK, now));
        assert!(!t.input(0x1234, now));
        assert!(t.input(BLACKWIDOW, now));
    }
}
