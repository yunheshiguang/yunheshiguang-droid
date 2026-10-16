use std::ffi::OsString;
use std::time::Duration;

use crate::workspace::adb::{AdbClient, AdbCommandError, AdbErrorKind, CommandPolicy};

pub const TRANSFER_POLICY: CommandPolicy =
    CommandPolicy::new(Duration::from_secs(300), 4 * 1024 * 1024);

const MAX_FAILURE_CHARS: usize = 200;

#[derive(Clone, Debug)]
pub struct AdbCommand {
    args: Vec<OsString>,
    policy: CommandPolicy,
    success_line: Option<&'static str>,
}

impl AdbCommand {
    pub fn new(args: impl IntoIterator<Item: Into<OsString>>, policy: CommandPolicy) -> Self {
        Self {
            args: args.into_iter().map(Into::into).collect(),
            policy,
            success_line: None,
        }
    }

    pub fn expecting(mut self, line: &'static str) -> Self {
        self.success_line = Some(line);
        self
    }

    #[cfg(test)]
    pub fn args(&self) -> &[OsString] {
        &self.args
    }

    pub fn run(&self, client: &AdbClient, serial: &str) -> Result<(), AdbCommandError> {
        let output = client.run_serial_with_policy(serial, &self.args, self.policy)?;

        self.check(&String::from_utf8_lossy(&output.stdout))
    }

    pub(super) fn check(&self, stdout: &str) -> Result<(), AdbCommandError> {
        let Some(expected) = self.success_line else {
            return Ok(());
        };

        if stdout.lines().any(|line| line.trim() == expected) {
            Ok(())
        } else {
            let reported = stdout
                .trim()
                .chars()
                .take(MAX_FAILURE_CHARS)
                .collect::<String>();
            Err(AdbCommandError::new(AdbErrorKind::NonZeroExit, reported))
        }
    }
}
