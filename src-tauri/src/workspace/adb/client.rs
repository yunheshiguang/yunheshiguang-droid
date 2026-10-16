use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;

use super::device::{AdbDevice, parse_device_list};
use super::error::AdbCommandError;
use super::process::{CommandPolicy, ProcessExecutor, ProcessOutput};

pub const PROBE_POLICY: CommandPolicy = CommandPolicy::new(Duration::from_secs(5), 1024 * 1024);
pub const ACTION_POLICY: CommandPolicy = CommandPolicy::new(Duration::from_secs(30), 1024 * 1024);

#[derive(Clone, Debug)]
pub struct AdbClient {
    path: PathBuf,
    executor: ProcessExecutor,
}

impl AdbClient {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            executor: ProcessExecutor,
        }
    }

    pub fn version(&self) -> Result<String, AdbCommandError> {
        let output = self.run([OsStr::new("version")], PROBE_POLICY)?;
        let text = String::from_utf8_lossy(&output.stdout);
        let version = text
            .lines()
            .map(str::trim)
            .find_map(|line| line.strip_prefix("Version "))
            .or_else(|| text.lines().map(str::trim).find(|line| !line.is_empty()))
            .unwrap_or_default();

        if version.is_empty() {
            Err(super::error::AdbCommandError::new(
                super::AdbErrorKind::InvalidOutput,
                "adb version returned no version information",
            ))
        } else {
            Ok(version.to_string())
        }
    }

    pub fn devices(&self) -> Result<Vec<AdbDevice>, AdbCommandError> {
        let output = self.run([OsStr::new("devices"), OsStr::new("-l")], PROBE_POLICY)?;
        parse_device_list(&output.stdout).map_err(|message| {
            super::error::AdbCommandError::new(super::AdbErrorKind::InvalidOutput, message)
        })
    }

    #[cfg(windows)]
    pub fn kill_server(&self) -> Result<(), AdbCommandError> {
        self.run([OsStr::new("kill-server")], PROBE_POLICY)
            .map(|_| ())
    }

    pub fn run_serial<I, S>(&self, serial: &str, args: I) -> Result<ProcessOutput, AdbCommandError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut command_args = vec![OsString::from("-s"), OsString::from(serial)];
        command_args.extend(args.into_iter().map(|value| value.as_ref().to_os_string()));
        self.executor.run(&self.path, &command_args, ACTION_POLICY)
    }

    pub fn run_serial_with_policy<I, S>(
        &self,
        serial: &str,
        args: I,
        policy: CommandPolicy,
    ) -> Result<ProcessOutput, AdbCommandError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut command_args = vec![OsString::from("-s"), OsString::from(serial)];
        command_args.extend(args.into_iter().map(|value| value.as_ref().to_os_string()));
        self.executor.run(&self.path, &command_args, policy)
    }

    pub fn run_serial_to_file<I, S>(
        &self,
        serial: &str,
        args: I,
        policy: CommandPolicy,
        file: File,
    ) -> Result<(), AdbCommandError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut command_args = vec![OsString::from("-s"), OsString::from(serial)];
        command_args.extend(args.into_iter().map(|value| value.as_ref().to_os_string()));
        self.executor
            .run_to_file(&self.path, &command_args, policy, file)
    }

    pub fn run<I, S>(
        &self,
        args: I,
        policy: CommandPolicy,
    ) -> Result<ProcessOutput, AdbCommandError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let args = args
            .into_iter()
            .map(|value| value.as_ref().to_os_string())
            .collect::<Vec<_>>();
        self.executor.run(&self.path, &args, policy)
    }

    pub fn run_with_input(
        &self,
        args: &[&str],
        input: &str,
        policy: CommandPolicy,
    ) -> Result<ProcessOutput, AdbCommandError> {
        let args = args.iter().map(OsString::from).collect::<Vec<_>>();
        self.executor
            .run_with_input(&self.path, &args, policy, input.as_bytes())
    }
}
