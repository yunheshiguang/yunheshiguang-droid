use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub struct CommandPolicy {
    pub timeout: Duration,
    pub max_output_bytes: usize,
}

impl CommandPolicy {
    pub const fn new(timeout: Duration, max_output_bytes: usize) -> Self {
        Self {
            timeout,
            max_output_bytes,
        }
    }
}
