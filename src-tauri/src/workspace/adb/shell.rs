use std::ffi::OsString;
use std::path::Path;
use std::time::Duration;

use yunheshiguang_sdk::device::{DeviceShell, MAX_SHELL_COMMAND_BYTES, ShellOutput};
use yunheshiguang_sdk::vendor::DeviceIdentity;

use crate::workspace::contract::UiText;
use crate::workspace::messages::*;
use crate::workspace::text::{ui_text, ui_text_with_params};

use super::{ACTION_POLICY, AdbClient, CommandPolicy};

const MAX_SHELL_TIMEOUT: Duration = Duration::from_secs(600);
const SHELL_OUTPUT_LIMIT: usize = 1024 * 1024;

fn is_plain(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || b"_./:=@%+,-".contains(&byte)
}

pub fn quote(argument: &str) -> String {
    if !argument.is_empty() && argument.bytes().all(is_plain) {
        argument.to_string()
    } else {
        format!("'{}'", argument.replace('\'', r"'\''"))
    }
}

pub fn command_line(args: &[&str]) -> Result<String, UiText> {
    if args.iter().any(|argument| argument.contains('\0')) {
        return Err(ui_text(DEVICE_SHELL_INVALID_KEY));
    }

    let command = args
        .iter()
        .map(|argument| quote(argument))
        .collect::<Vec<_>>()
        .join(" ");

    if command.len() > MAX_SHELL_COMMAND_BYTES {
        return Err(ui_text(DEVICE_SHELL_TOO_LONG_KEY));
    }

    Ok(command)
}

pub struct AdbDeviceShell<'a> {
    client: &'a AdbClient,
    serial: &'a str,
    identity: Option<&'a DeviceIdentity>,
}

impl<'a> AdbDeviceShell<'a> {
    pub fn new(
        client: &'a AdbClient,
        serial: &'a str,
        identity: Option<&'a DeviceIdentity>,
    ) -> Self {
        Self {
            client,
            serial,
            identity,
        }
    }

    fn failure(&self, message: &str) -> UiText {
        ui_text_with_params(
            DEVICE_SHELL_FAILED_KEY,
            "",
            [("message", message.replace(self.serial, "<device>"))],
        )
    }

    fn run(&self, args: &[&str], policy: CommandPolicy) -> Result<ShellOutput, UiText> {
        let command = command_line(args)?;

        self.client
            .run_serial_with_policy(self.serial, ["shell", command.as_str()], policy)
            .map(|output| ShellOutput::new(String::from_utf8_lossy(&output.stdout)))
            .map_err(|error| self.failure(&error.to_string()))
    }
}

impl DeviceShell for AdbDeviceShell<'_> {
    fn identity(&self) -> Option<&DeviceIdentity> {
        self.identity
    }

    fn shell(&self, args: &[&str]) -> Result<ShellOutput, UiText> {
        self.run(args, ACTION_POLICY)
    }

    fn shell_with_timeout(&self, args: &[&str], timeout: Duration) -> Result<ShellOutput, UiText> {
        self.run(
            args,
            CommandPolicy::new(timeout.min(MAX_SHELL_TIMEOUT), SHELL_OUTPUT_LIMIT),
        )
    }

    fn push(&self, local: &Path, remote: &str) -> Result<(), UiText> {
        let args = [
            OsString::from("push"),
            local.as_os_str().to_os_string(),
            OsString::from(remote),
        ];

        self.client
            .run_serial_with_policy(self.serial, args, ACTION_POLICY)
            .map(|_| ())
            .map_err(|error| self.failure(&error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes_every_argument_into_exactly_one_word() {
        assert_eq!(quote("-keyboard"), "-keyboard");
        assert_eq!(
            quote("-Djava.class.path=/data/local/tmp/x"),
            "-Djava.class.path=/data/local/tmp/x"
        );
        assert_eq!(quote("a b; rm -rf /"), "'a b; rm -rf /'");
        assert_eq!(quote("it's $(reboot)"), r"'it'\''s $(reboot)'");
        assert_eq!(quote("你好"), "'你好'");
        assert_eq!(quote(""), "''");
        assert_eq!(
            command_line(&["echo", "a b", "c"]).as_deref(),
            Ok("echo 'a b' c")
        );
    }

    #[test]
    fn refuses_commands_the_device_shell_cannot_take() {
        assert_eq!(
            command_line(&["echo", "a\0b"]),
            Err(ui_text(DEVICE_SHELL_INVALID_KEY))
        );
        assert_eq!(
            command_line(&["echo", &"字".repeat(1400)]),
            Err(ui_text(DEVICE_SHELL_TOO_LONG_KEY))
        );
    }
}
