// StreamShield Core - Error Types
#[derive(Debug)]
pub enum ShieldError {
    InvalidWindowHandle(isize),
    AccessDenied(u32),
    ProcessNotFound(u32),
    AffinityFailed(i32),
}

impl std::fmt::Display for ShieldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ShieldError {}
