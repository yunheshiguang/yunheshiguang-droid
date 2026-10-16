use std::path::Path;
use std::time::Duration;

use crate::contract::yunheshiguang::workspace::v1::UiText;
use crate::vendor::DeviceIdentity;

pub const MAX_SHELL_COMMAND_BYTES: usize = 4000;

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ShellOutput {
    pub stdout: String,
}

impl ShellOutput {
    pub fn new(stdout: impl Into<String>) -> Self {
        Self {
            stdout: stdout.into(),
        }
    }
}

pub trait DeviceShell {
    fn identity(&self) -> Option<&DeviceIdentity>;

    fn shell(&self, args: &[&str]) -> Result<ShellOutput, UiText>;

    fn shell_with_timeout(&self, args: &[&str], timeout: Duration) -> Result<ShellOutput, UiText> {
        let _ = timeout;
        self.shell(args)
    }

    fn push(&self, local: &Path, remote: &str) -> Result<(), UiText>;
}
