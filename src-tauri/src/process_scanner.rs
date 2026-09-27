// StreamShield Core - ProcessScanner
use std::collections::HashMap;

pub struct ProcessScanner {
    cache: HashMap<u32, String>,
}

impl ProcessScanner {
    pub fn new() -> Self {
        Self { cache: HashMap::new() }
    }

    pub fn resolve_name(&mut self, pid: u32) -> Option<&String> {
        self.cache.get(&pid)
    }

    pub fn insert_mapping(&mut self, pid: u32, name: String) {
        self.cache.insert(pid, name);
    }
}
