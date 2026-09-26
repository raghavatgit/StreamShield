// StreamShield Core - ThreadSafeAffinityGuard
// Manages window affinity state transitions atomically

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct AffinityGuard {
    masked_hwnds: Arc<Mutex<HashSet<isize>>>,
}

impl AffinityGuard {
    pub fn new() -> Self {
        Self {
            masked_hwnds: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn register_exclusion(&self, hwnd: isize) -> bool {
        let mut set = self.masked_hwnds.lock().unwrap();
        set.insert(hwnd)
    }

    pub fn remove_exclusion(&self, hwnd: isize) -> bool {
        let mut set = self.masked_hwnds.lock().unwrap();
        set.remove(&hwnd)
    }

    pub fn is_excluded(&self, hwnd: isize) -> bool {
        let set = self.masked_hwnds.lock().unwrap();
        set.contains(&hwnd)
    }
}
