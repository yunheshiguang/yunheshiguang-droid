use std::process::{Child, Command};
use std::thread;
use std::time::{Duration, Instant};

use crate::workspace::adb::error::{AdbCommandError, AdbErrorKind};

const POLL_INTERVAL: Duration = Duration::from_millis(10);

pub fn wait_for_exit(
    child: &mut Child,
    timeout: Duration,
) -> Result<std::process::ExitStatus, AdbCommandError> {
    let started = Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) if started.elapsed() >= timeout => {
                terminate_child(child);

                return Err(AdbCommandError::new(
                    AdbErrorKind::Timeout,
                    format!("command timed out after {} ms", timeout.as_millis()),
                ));
            }
            Ok(None) => thread::sleep(POLL_INTERVAL),
            Err(error) => {
                terminate_child(child);

                return Err(AdbCommandError::from_io(error));
            }
        }
    }
}

pub fn terminate_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(windows)]
pub fn configure_command(command: &mut Command) {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
pub fn configure_command(_command: &mut Command) {}
