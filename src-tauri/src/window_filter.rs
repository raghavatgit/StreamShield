// StreamShield Core - WindowFilter
// Matches process names against exclusion lists

pub struct WindowFilter {
    target_processes: Vec<String>,
}

impl WindowFilter {
    pub fn new(targets: Vec<&str>) -> Self {
        Self {
            target_processes: targets.into_iter().map(|s| s.to_lowercase()).collect(),
        }
    }

    pub fn matches(&self, process_name: &str) -> bool {
        let lower = process_name.to_lowercase();
        self.target_processes.iter().any(|target| lower.contains(target))
    }
}
