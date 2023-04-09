use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use super::Status;

pub struct LCDStatusDisplay {
    wifi_connected: AtomicBool,
    barrier_connected: AtomicBool,
    activated: AtomicBool,
    clipboard_size: AtomicUsize,
}

impl LCDStatusDisplay {
    fn new() -> Self {
        Self {
            wifi_connected: AtomicBool::new(false),
            barrier_connected: AtomicBool::new(false),
            activated: AtomicBool::new(false),
            clipboard_size: AtomicUsize::new(0),
        }
    }

    pub fn set_status(&mut self, status: Status) {
        match status {
            Status::Start => {}
            Status::WifiConnected => {
                self.wifi_connected.store(true, Ordering::Relaxed);
            }
            Status::BarrierConnected => {
                self.wifi_connected.store(true, Ordering::Relaxed);
                self.barrier_connected.store(true, Ordering::Relaxed);
            }
            Status::Activated => {
                self.wifi_connected.store(true, Ordering::Relaxed);
                self.barrier_connected.store(true, Ordering::Relaxed);
                self.activated.store(true, Ordering::Relaxed);
            }
            Status::Deactivated => {
                self.wifi_connected.store(true, Ordering::Relaxed);
                self.barrier_connected.store(true, Ordering::Relaxed);
                self.activated.store(false, Ordering::Relaxed);
            }
            Status::ClipboardSize(size) => {
                self.clipboard_size.store(size, Ordering::Relaxed);
            }
        }
    }
}
